#!/usr/bin/env python3
"""PRIVATE original Stage 5B HMM/MMS historical FAIL replay (never a physics PASS).

Run only in an approved isolated private environment with SHA-pinned archive.
Public GitHub Actions execute synthetic tests of this file, NEVER the private ZIP.
This subprocess executes reviewed original source; it is NOT an OS security sandbox.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import io
import json
import os
from pathlib import Path
import re
import resource
import signal
import shutil
import subprocess
import sys
import tempfile
import zipfile

from ph08_evidence_audit import load_unique_json
from ph08_stage5b_preflight import (
    MAX_OUTER_ARCHIVE_COMPRESSED, STAGE5B_SHA256, STAGE4_SHA256, preflight,
)

SNAP_PREFIX = "stage5b/source_snapshot/"
STAGE4_QUAL = "stage4/boardsi-ph08-stage4-qualification/evidence/qualification.json"
NESTED_STAGE4 = "inputs/boardsi-ph08-stage4-qualification.zip"
ENTRYPOINT = "m2_ph08_stage5b_qualification.py"
MAX_JSON_BYTES = 4 * 1024 * 1024
EXPECTED_ROWS = 12
EXPECTED_NUMERIC = 1761
EXPECTED_TIMINGS = 12


def fail(code: str) -> dict:
    return {
        "status": "REPLAY_NOT_VERIFIED", "reason_code": code,
        "stage5b_archive_integrity": "NOT_VERIFIED",
        "original_stage5b_replay": "NOT_VERIFIED",
        "archived_gate_q": "FAIL", "physical_gate_q": "FAIL",
        "physical_solves": "NOT_VERIFIED",
        "AC_P1": "PENDING", "contract": "NOT_FROZEN",
        "rust_fem": "NO_GO", "Gate_M": "NOT_VERIFIED",
        "physical_error_bound": "BOUND_NOT_CERTIFIED",
    }


def require_qualification_object(value: object) -> dict:
    """Fail closed on syntactically valid but non-object generated JSON."""
    if not isinstance(value, dict):
        raise ValueError("qualification JSON root must be an object")
    return value


def compare_qualification(recorded: object, actual: object) -> dict:
    """Exact deterministic comparison with ONLY known timing fields excluded.

    Arbitrary nonnumeric strings/dictionaries, including changed fail status,
    are NEVER ignored. Count non-boolean numeric scalars once per pair.
    """
    numeric = 0
    timings = 0
    mismatches = 0
    def walk(a, b, path):
        nonlocal numeric, timings, mismatches
        if type(a) is not type(b):
            mismatches += 1
        elif isinstance(a, dict):
            if a.keys() != b.keys():
                mismatches += 1
            else:
                for k in a:
                    walk(a[k], b[k], path + (k,))
        elif isinstance(a, list):
            if len(a) != len(b):
                mismatches += 1
            else:
                for i, (x, y) in enumerate(zip(a,b)):
                    walk(x, y, path + (i,))
        elif type(a) in (int, float):
            if (len(path) == 4 and path[0] == "rows" and
                    type(path[1]) is int and path[2:] == ("linear", "seconds")):
                timings += 1
            else:
                numeric += 1
                if a != b:
                    mismatches += 1
        elif a != b:
            mismatches += 1
    walk(recorded, actual, ())
    return {"numeric_scalars_compared": numeric,
            "timing_scalars_excluded": timings,
            "mismatches": mismatches}


def _safe_sources(z: zipfile.ZipFile) -> dict[str, bytes]:
    pin = load_unique_json(z.read("stage5b/preregistration.json"))
    registered = pin.get("sources") if isinstance(pin, dict) else None
    if not isinstance(registered, dict) or len(registered) != 13:
        raise ValueError("source registration missing")
    discovered = {
        n[len(SNAP_PREFIX):] for n in z.namelist()
        if n.startswith(SNAP_PREFIX) and not n.endswith("/")
    }
    if discovered != set(registered) or ENTRYPOINT not in discovered:
        raise ValueError("registered source inventory mismatch")
    safe = {}
    for name, digest in registered.items():
        if (not isinstance(name, str) or
                re.fullmatch(r"[A-Za-z0-9_.-]+", name) is None or
                name in (".", "..") or not isinstance(digest, str) or
                re.fullmatch(r"[a-fA-F0-9]{64}", digest) is None):
            raise ValueError("invalid source name or digest")
        raw = z.read(SNAP_PREFIX + name)
        if hashlib.sha256(raw).hexdigest() != digest.lower():
            raise ValueError("source snapshot digest mismatch")
        safe[name] = raw
    return safe


def _approved_runtime() -> tuple[bool, dict]:
    """Pinned reproducibility contract, not a claim about interpreter security."""
    runtime = {"python": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"}
    for package in ("numpy", "scipy"):
        try:
            runtime[package] = importlib.metadata.version(package)
        except importlib.metadata.PackageNotFoundError:
            runtime[package] = "NOT_INSTALLED"
        except Exception:
            # Broken/unreadable installed distribution metadata is untrusted
            # environment input. Never leak a traceback or approve runtime.
            runtime[package] = "METADATA_NOT_VERIFIED"
    approved = (sys.version_info[:2] == (3, 13) and
                runtime["numpy"] == "2.3.5" and
                runtime["scipy"] == "1.17.0")
    return approved, runtime


def _limits():
    """Never request an invalid soft>hard limit on constrained private hosts."""
    for category, cap in ((resource.RLIMIT_CPU, 80),
                          (resource.RLIMIT_AS, 2 * 1024**3),
                          (resource.RLIMIT_FSIZE, 2 * 1024**2)):
        soft, hard = resource.getrlimit(category)
        bounded = cap if hard == resource.RLIM_INFINITY else min(cap, hard)
        resource.setrlimit(category, (bounded, hard))


def _kill_replay_group(pid: int):
    """Kill descendants even when the original Python child has exited."""
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass



def replay(path: Path) -> dict:
    out = fail("UNINITIALIZED")
    if os.name != "posix":
        return fail("UNSUPPORTED_NONPOSIX_PRIVATE_REPLAY")
    if not path.is_file():
        return fail("MISSING_PRIVATE_ARCHIVE")
    approved, runtime = _approved_runtime()
    if not approved:
        return fail("RUNTIME_VERSION_UNAPPROVED")
    try:
        with tempfile.TemporaryDirectory(prefix="boardsi-stage5b-private-replay-") as tmp:
            root = Path(tmp)
            snap = root / "original-pinned.zip"
            copied = 0
            h = hashlib.sha256()
            # Never re-open or execute a path that can change between checks.
            with path.open("rb") as source, snap.open("xb") as dst:
                while block := source.read(1024 * 1024):
                    copied += len(block)
                    if copied > MAX_OUTER_ARCHIVE_COMPRESSED:
                        return fail("ARCHIVE_SIZE_LIMIT")
                    h.update(block)
                    dst.write(block)
            if h.hexdigest() != STAGE5B_SHA256:
                return fail("ORIGINAL_ARCHIVE_SHA256_MISMATCH")
            checked = preflight(snap)
            if checked["status"] != "PASS_INTEGRITY_ONLY_NO_REPLAY":
                return fail("STAGE5B_OR_STAGE4_INTEGRITY_NOT_VERIFIED")
            # The auditor internally validates its own copy. Re-authenticate
            # the exact immutable in-memory bytes that will be parsed/executed.
            # Never reopen mutable snap after this point.
            pinned_bytes = snap.read_bytes()
            if hashlib.sha256(pinned_bytes).hexdigest() != STAGE5B_SHA256:
                return fail("AUTHENTICATED_SNAPSHOT_CHANGED")
            with zipfile.ZipFile(io.BytesIO(pinned_bytes)) as z:
                sources = _safe_sources(z)
                if z.getinfo("stage5b/qualification.json").file_size > MAX_JSON_BYTES:
                    return fail("SAVED_JSON_TOO_LARGE")
                saved = load_unique_json(z.read("stage5b/qualification.json"))
                if (not isinstance(saved, dict) or saved.get("Gate_Q") != "FAIL"
                        or saved.get("certified") is not False
                        or saved.get("physical_solves") != 0
                        or saved.get("pilot") != "NOT RUN"
                        or len(saved.get("rows", [])) != EXPECTED_ROWS):
                    return fail("ARCHIVED_FAILURE_SCHEMA_MISMATCH")
                # The original source reads both its preregistration file and
                # selected historical Stage 4 qualification by relative path.
                dst_root = root / "stage5b"
                source_dir = dst_root / "source_snapshot"
                source_dir.mkdir(parents=True)
                for name, raw in sources.items():
                    (source_dir / name).write_bytes(raw)
                (dst_root / "preregistration.json").write_bytes(
                    z.read("stage5b/preregistration.json"))
                stage4 = root / STAGE4_QUAL
                stage4.parent.mkdir(parents=True)
                stage4.write_bytes(z.read(STAGE4_QUAL))
                with zipfile.ZipFile(io.BytesIO(z.read(NESTED_STAGE4))) as nested:
                    nested_qual = nested.read(
                        "boardsi-ph08-stage4-qualification/evidence/qualification.json")
                if stage4.read_bytes() != nested_qual:
                    return fail("STAGE4_HISTORICAL_BYTES_MISMATCH")
            env = os.environ.copy()
            env.pop("PYTHONPATH", None)
            env["PYTHONNOUSERSITE"] = "1"
            for key in ("OPENBLAS_NUM_THREADS","OMP_NUM_THREADS",
                        "MKL_NUM_THREADS","NUMEXPR_NUM_THREADS"):
                env[key] = "1"
            # Both streams go to bounded PRIVATE files; never return raw logs.
            with (root / "private-stdout.log").open("wb") as stdout, (
                    root / "private-stderr.log").open("wb") as stderr:
                child = None
                try:
                    child = subprocess.Popen(
                        [sys.executable, str(source_dir / ENTRYPOINT)],
                        cwd=dst_root, stdin=subprocess.DEVNULL,
                        stdout=stdout, stderr=stderr, env=env,
                        preexec_fn=_limits, start_new_session=True)
                    try:
                        exit_code = child.wait(timeout=90)
                    except subprocess.TimeoutExpired:
                        return fail("PRIVATE_REPLAY_TIMEOUT")
                finally:
                    if child is not None:
                        # The original code is trusted by SHA, but child/spawned
                        # descendants must not outlive this private tempdir.
                        _kill_replay_group(child.pid)
                        try:
                            child.wait(timeout=10)
                        except subprocess.TimeoutExpired:
                            return fail("PRIVATE_REPLAY_TERMINATION_FAILED")
            generated = dst_root / "qualification.json"
            if exit_code != 1 or not generated.is_file():
                return fail("ORIGINAL_EXIT_OR_OUTPUT_MISMATCH")
            if generated.stat().st_size > MAX_JSON_BYTES:
                return fail("GENERATED_JSON_TOO_LARGE")
            actual = require_qualification_object(load_unique_json(generated.read_bytes()))
            result = compare_qualification(saved, actual)
            if (result["numeric_scalars_compared"] != EXPECTED_NUMERIC or
                    result["timing_scalars_excluded"] != EXPECTED_TIMINGS or
                    result["mismatches"] != 0 or
                    actual.get("Gate_Q") != "FAIL" or
                    actual.get("certified") is not False or
                    actual.get("physical_solves") != 0 or
                    actual.get("pilot") != "NOT RUN"):
                return fail("HISTORICAL_FAIL_REPLAY_DIFFERENCE")
            return {
                "status": "REPRODUCED_HISTORICAL_GATE_Q_FAIL",
                "stage5b_archive_integrity": "MATCH",
                "approved_runtime": runtime,
                "stage4_nested_sha256": STAGE4_SHA256,
                "source_sha256_verified": 13,
                "mms_rows": EXPECTED_ROWS,
                **result,
                "original_stage5b_replay": "REEXECUTED_HISTORICAL_FAIL",
                "archived_gate_q": "FAIL", "replayed_gate_q": "FAIL",
                "physical_gate_q": "FAIL", "physical_solves": 0,
                "pilot": "NOT RUN", "certified": False,
                "AC_P1": "PENDING", "contract": "NOT_FROZEN",
                "rust_fem": "NO_GO", "Gate_M": "NOT_VERIFIED",
                "physical_error_bound": "BOUND_NOT_CERTIFIED",
            }
    except (OSError, ValueError, RuntimeError, zipfile.BadZipFile, KeyError,
            TypeError, subprocess.SubprocessError,
            json.JSONDecodeError, EOFError):
        return fail("ARCHIVE_SCHEMA_OR_IO_FAILURE")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("private_archive", type=Path)
    ap.add_argument("--sanitized-json", type=Path)
    args = ap.parse_args(argv)
    result = replay(args.private_archive)
    data = json.dumps(result, sort_keys=True, allow_nan=False, indent=2) + "\n"
    if args.sanitized_json:
        args.sanitized_json.write_text(data, encoding="utf-8")
    print(data)
    return 0 if result["status"] == "REPRODUCED_HISTORICAL_GATE_Q_FAIL" else 2


if __name__ == "__main__":
    raise SystemExit(main())
