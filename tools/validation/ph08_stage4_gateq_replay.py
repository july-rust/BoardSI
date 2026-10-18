#!/usr/bin/env python3
"""Reproduce the *historical FAIL* from a SHA-pinned private PH-08 Stage 4 archive.

This is provenance / numerical regression ONLY. A successful run means the
historical HMM Gate Q failure was reproduced, NOT that the HMM is qualified.
The archive is never extracted outside a private TemporaryDirectory, uploaded,
or written to a GitHub Actions log. Execute only on an approved isolated,
network-disabled POSIX host: subprocess limits are NOT a security sandbox.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import io
import json
import math
import os
import signal
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

from ph08_evidence_audit import inspect, load_unique_json

STAGE4_ZIP_SHA256 = '6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9'
ZIP_ROOT = 'boardsi-ph08-stage4-qualification/'
COMPARE_FIELDS = ('bulk_phi_rms', 'bulk_gradient_rms', 'linear_residual',
                  'interface_phi_rms', 'interface_gradient_rms',
                  'interface_normal_D_exact_error_rms_normalized',
                  'exact_data_max_error')
ABS_TOL = 2e-13
REL_TOL = 5e-10


def compare_rows(actual: dict, reference: dict) -> list[str]:
    errors = []
    if actual.get('status') != 'FAIL' or reference.get('status') != 'FAIL':
        errors.append('historical FAIL status not preserved')
    if actual.get('failures') != reference.get('failures'):
        errors.append('MMS gate failures differ')
    a_rows, b_rows = actual.get('rows', []), reference.get('rows', [])
    if len(a_rows) != 36 or len(b_rows) != 36:
        errors.append('expected 36 MMS rows')
        return errors
    for index, (a, b) in enumerate(zip(a_rows, b_rows)):
        for key in ('grid', 'kind', 'n'):
            if a.get(key) != b.get(key):
                errors.append(f'row {index}: {key} changed')
        for key in COMPARE_FIELDS:
            if (key in a) != (key in b):
                errors.append(f'row {index}: {key} missing')
            elif key in a:
                va, vb = a[key], b[key]
                if (not isinstance(va, (float, int)) or
                    not isinstance(vb, (float, int)) or
                    not math.isfinite(va) or not math.isfinite(vb) or
                    not math.isclose(va, vb, abs_tol=ABS_TOL, rel_tol=REL_TOL)):
                    errors.append(f'row {index}: {key} differs')
    return errors


def check_historical_gate(mms: dict) -> bool:
    """This reproducer *requires* the original fail; it cannot approve PH-08."""
    target = ('uniform', 'quadratic', 'bulk_gradient_rms', 'reduction')
    hits = [x for x in mms.get('failures', []) if tuple(x[:4]) == target]
    return (mms.get('status') == 'FAIL' and len(mms.get('failures', [])) == 1
            and len(hits) == 1 and len(hits[0][4]) == 2
            and all(isinstance(x, (float, int)) and math.isfinite(x) and x < 1.8 for x in hits[0][4]))



MAX_ARCHIVE_BYTES = 8 * 1024 * 1024
MAX_JSON_BYTES = 4 * 1024 * 1024
MAX_SOURCE_BYTES = 4 * 1024 * 1024
EXPECTED_RUNTIME = {'numpy': '2.3.5', 'scipy': '1.17.0'}


class ReplayNotVerified(Exception):
    """Only a fixed public reason code may escape to a report."""

    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def fail(code: str) -> dict:
    return {
        'status': 'REPLAY_NOT_VERIFIED',
        'reason_code': code,
        'original_gate_q': 'FAIL',
        'AC_P1': 'PENDING',
        'TR': 'PROPOSED_NOT_FROZEN',
        'contract': 'NOT_FROZEN',
        'rust_fem': 'NO_GO',
        'Gate_M': 'NOT_VERIFIED',
        'physical_error_bound': 'BOUND_NOT_CERTIFIED',
    }


def _approved_parent_runtime() -> bool:
    """Metadata is a preliminary check; actual child imports are checked separately."""
    if sys.version_info[:2] != (3, 13):
        return False
    for package, expected in EXPECTED_RUNTIME.items():
        try:
            if importlib.metadata.version(package) != expected:
                return False
        except Exception:
            return False
    return True


def _read_original_once(archive: Path) -> bytes:
    """One input handle, bounded memory, immutable in-memory bytes for all later steps."""
    parts = []
    size = 0
    try:
        with archive.open('rb') as source:
            while chunk := source.read(1024 * 1024):
                size += len(chunk)
                if size > MAX_ARCHIVE_BYTES:
                    raise ReplayNotVerified('ARCHIVE_SIZE_LIMIT')
                parts.append(chunk)
    except ReplayNotVerified:
        raise
    except OSError:
        raise ReplayNotVerified('MISSING_OR_UNREADABLE_PRIVATE_ARCHIVE') from None
    data = b''.join(parts)
    if hashlib.sha256(data).hexdigest() != STAGE4_ZIP_SHA256:
        raise ReplayNotVerified('ORIGINAL_ARCHIVE_SHA256_MISMATCH')
    return data


def _read_zip_member(z: zipfile.ZipFile, name: str, max_size: int) -> bytes:
    try:
        info = z.getinfo(name)
        if info.file_size > max_size:
            raise ReplayNotVerified('ARCHIVE_MEMBER_SIZE_LIMIT')
        with z.open(info) as stream:
            content = stream.read(max_size + 1)
        if len(content) > max_size:
            raise ReplayNotVerified('ARCHIVE_MEMBER_SIZE_LIMIT')
        return content
    except KeyError:
        raise ReplayNotVerified('REQUIRED_ARCHIVE_MEMBER_MISSING') from None


def _clean_env() -> dict:
    env = os.environ.copy()
    for key in ('PYTHONPATH', 'PYTHONHOME', 'PYTHONSTARTUP',
                'PYTHONINSPECT', 'PYTHONUSERBASE', 'PYTHONWARNINGS'):
        env.pop(key, None)
    env['PYTHONNOUSERSITE'] = '1'
    env['PYTHONDONTWRITEBYTECODE'] = '1'
    for key in ('OPENBLAS_NUM_THREADS', 'OMP_NUM_THREADS', 'MKL_NUM_THREADS',
                'NUMEXPR_NUM_THREADS'):
        env[key] = '1'
    return env


def _child_limits():
    """Per-process limits only; host isolation and aggregate quotas still required."""
    import resource
    for category, cap in ((resource.RLIMIT_CPU, 80),
                          (resource.RLIMIT_AS, 4 * 1024**3),
                          (resource.RLIMIT_FSIZE, MAX_JSON_BYTES)):
        soft, hard = resource.getrlimit(category)
        bounded = cap if hard == resource.RLIM_INFINITY else min(cap, hard)
        resource.setrlimit(category, (bounded, bounded))


def _run_private_child(argv: list[str], cwd: Path, env: dict,
                       stdout: Path, stderr: Path, timeout: int) -> int:
    child = None
    with stdout.open('xb') as out, stderr.open('xb') as err:
        try:
            child = subprocess.Popen(
                argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                stdout=out, stderr=err, start_new_session=True,
                preexec_fn=_child_limits,
            )
            try:
                return child.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                raise ReplayNotVerified('PRIVATE_CHILD_TIMEOUT') from None
        finally:
            if child is not None:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                try:
                    child.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    raise ReplayNotVerified('CHILD_TERMINATION_NOT_VERIFIED') from None


def _probe_actual_child_runtime(source_dir: Path, root: Path, env: dict) -> None:
    probe = (
        "import json,sys;sys.path.insert(0,sys.argv[1]);"
        "import numpy,scipy;"
        "print(json.dumps({'python':'.'.join(map(str,sys.version_info[:3])),"
        "'numpy':numpy.__version__,'scipy':scipy.__version__}))"
    )
    out, err = root / 'probe-private-output.json', root / 'probe-private-error.txt'
    code = _run_private_child(
        [sys.executable, '-I', '-c', probe, str(source_dir)],
        source_dir, env, out, err, timeout=30)
    if code != 0 or out.stat().st_size > 2048:
        raise ReplayNotVerified('CHILD_RUNTIME_PROBE_FAILED')
    observed = load_unique_json(out.read_bytes())
    if (not isinstance(observed, dict) or set(observed) != {'python', 'numpy', 'scipy'} or
            any(not isinstance(v, str) for v in observed.values()) or
            not observed['python'].startswith('3.13.') or
            any(observed[p] != v for p, v in EXPECTED_RUNTIME.items())):
        raise ReplayNotVerified('CHILD_RUNTIME_VERSION_UNAPPROVED')



def _verified_child_runtime(value: object) -> bool:
    """Validate the versions observed by the SAME process that ran mms_suite."""
    return (isinstance(value, dict) and
            set(value) == {'python', 'numpy', 'scipy'} and
            all(isinstance(v, str) for v in value.values()) and
            value['python'].startswith('3.13.') and
            all(value[k] == v for k, v in EXPECTED_RUNTIME.items()))


def rerun(archive: Path) -> dict:
    """Reproduce an archived FAIL in approved private isolation; never certify physics."""
    if os.name != 'posix':
        raise ReplayNotVerified('UNSUPPORTED_NONPOSIX_PRIVATE_REPLAY')
    if not _approved_parent_runtime():
        raise ReplayNotVerified('PARENT_RUNTIME_VERSION_UNAPPROVED')
    pinned_data = _read_original_once(archive)
    with tempfile.TemporaryDirectory(prefix='boardsi-ph08-stage4-private-') as temp:
        root = Path(temp)
        pinned = root / 'authenticated-private.zip'
        pinned.write_bytes(pinned_data)
        audit = inspect(pinned, expected_sha256=STAGE4_ZIP_SHA256,
                        expected_entries=338, expected_sources=20,
                        max_bytes=128 * 1024 * 1024)
        if (audit.get('result') != 'PASS' or
                audit.get('checksum_verified') != 338 or
                audit.get('source_hash_verified') != 20 or
                audit.get('unlisted_members')):
            raise ReplayNotVerified('ORIGINAL_ARCHIVE_INTEGRITY_NOT_VERIFIED')
        # Audit opens a copy on disk. Re-authenticate the same byte sequence
        # before parsing/executing the immutable memory copy.
        if hashlib.sha256(pinned.read_bytes()).hexdigest() != STAGE4_ZIP_SHA256:
            raise ReplayNotVerified('AUDITED_SNAPSHOT_CHANGED')
        with zipfile.ZipFile(io.BytesIO(pinned_data)) as z:
            saved_obj = load_unique_json(_read_zip_member(
                z, ZIP_ROOT + 'evidence/qualification.json', MAX_JSON_BYTES))
            if not isinstance(saved_obj, dict) or not isinstance(saved_obj.get('mms'), dict):
                raise ReplayNotVerified('ARCHIVED_MMS_SCHEMA_INVALID')
            saved = saved_obj['mms']
            manifest = load_unique_json(_read_zip_member(
                z, ZIP_ROOT + 'evidence/manifest.json', MAX_JSON_BYTES))
            inventory = manifest.get('sources') if isinstance(manifest, dict) else None
            if not isinstance(inventory, dict) or len(inventory) != 20:
                raise ReplayNotVerified('ORIGINAL_SOURCE_INVENTORY_INVALID')
            if 'm2_ph08_stage4_qualification.py' not in inventory:
                raise ReplayNotVerified('ORIGINAL_ENTRYPOINT_NOT_REGISTERED')
            source_dir = root / 'source_snapshot'
            source_dir.mkdir()
            for basename, digest in sorted(inventory.items()):
                if (not isinstance(basename, str) or not basename or
                        not basename.isascii() or
                        any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-'
                            for c in basename) or basename in ('.', '..') or
                        not basename.endswith(('.py', '.json', '.md')) or
                        not isinstance(digest, str) or len(digest) != 64):
                    raise ReplayNotVerified('UNSAFE_ORIGINAL_SOURCE_NAME')
                raw = _read_zip_member(
                    z, ZIP_ROOT + 'evidence/source_snapshot/' + basename, MAX_SOURCE_BYTES)
                if hashlib.sha256(raw).hexdigest() != digest.lower():
                    raise ReplayNotVerified('ORIGINAL_SOURCE_SHA256_MISMATCH')
                (source_dir / basename).write_bytes(raw)
        env = _clean_env()
        _probe_actual_child_runtime(source_dir, root, env)
        program = (
            "import json,sys,numpy,scipy\n"
            "runtime={'python':'.'.join(map(str,sys.version_info[:3])),"
            "'numpy':numpy.__version__,'scipy':scipy.__version__}\n"
            "if (not runtime['python'].startswith('3.13.') or "
            "runtime['numpy']!='2.3.5' or runtime['scipy']!='1.17.0'):\n"
            "    raise SystemExit(86)\n"
            "sys.path.insert(0,sys.argv[1])\n"
            "from m2_ph08_stage4_qualification import mms_suite\n"
            "print(json.dumps({'runtime':runtime,'mms':mms_suite()},allow_nan=False))"
        )
        out, err = root / 'original-private-output.json', root / 'original-private-error.txt'
        exit_code = _run_private_child(
            [sys.executable, '-I', '-c', program, str(source_dir)],
            source_dir, env, out, err, timeout=120)
        if exit_code != 0 or out.stat().st_size > MAX_JSON_BYTES:
            raise ReplayNotVerified('ORIGINAL_REPLAY_EXIT_OR_OUTPUT_INVALID')
        generated = load_unique_json(out.read_bytes())
        if (not isinstance(generated, dict) or
                set(generated) != {'runtime', 'mms'} or
                not _verified_child_runtime(generated['runtime']) or
                not isinstance(generated['mms'], dict)):
            raise ReplayNotVerified('REPLAY_OR_RUNTIME_SCHEMA_INVALID')
        replay = generated['mms']
    diffs = compare_rows(replay, saved)
    if not check_historical_gate(replay):
        diffs.append('original uniform quadratic gradient failure not reproduced')
    if diffs:
        raise ReplayNotVerified('HISTORICAL_FAIL_REPLAY_DIFFERENCE')
    return {
        'status': 'REPRODUCED_HISTORICAL_FAIL',
        'archive_sha256': STAGE4_ZIP_SHA256,
        'source_snapshots_verified': 20,
        'checksum_entries_verified': 338,
        'mms_rows_reexecuted': 36,
        'original_gate_q': 'FAIL',
        'AC_P1': 'PENDING',
        'TR': 'PROPOSED_NOT_FROZEN',
        'contract': 'NOT_FROZEN',
        'rust_fem': 'NO_GO',
        'Gate_M': 'NOT_VERIFIED',
        'physical_error_bound': 'BOUND_NOT_CERTIFIED',
    }


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('archive', type=Path)
    ap.add_argument('--output-json', type=Path)
    args = ap.parse_args(argv)
    try:
        result = rerun(args.archive)
    except ReplayNotVerified as exc:
        result = fail(exc.code)
    except Exception:
        # NEVER disclose exception text: private file paths, original stderr
        # and source operands may appear in underlying exceptions.
        result = fail('PRIVATE_REPLAY_INTERNAL_ERROR')
    output = json.dumps(result, indent=2, allow_nan=False) + '\n'
    if args.output_json:
        try:
            args.output_json.write_text(output, encoding='utf-8')
        except Exception:
            result = fail('SANITIZED_REPORT_WRITE_FAILED')
            output = json.dumps(result, indent=2, allow_nan=False) + '\n'
    print(output)
    return 0 if result['status'] == 'REPRODUCED_HISTORICAL_FAIL' else 2


if __name__ == '__main__':
    raise SystemExit(main())
