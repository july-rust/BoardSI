#!/usr/bin/env python3
"""Stage 5B private-archive integrity preflight. Never executes original code.

This is not a physics replay. Only a separately reviewed, source-specific
runner in an authorized private environment may execute archived HMM source.
Public CI runs synthetic archives and negative controls only.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import tempfile
import zipfile

from ph08_evidence_audit import (
    clean_path, inspect, load_unique_json, parse_sum_lines, sha256_file,
)

STAGE5B_SHA256 = "f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36"
STAGE4_SHA256 = "6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9"
SUMS = "stage5b/SHA256SUMS"
QUALIFICATION = "stage5b/qualification.json"
NESTED_STAGE4 = "inputs/boardsi-ph08-stage4-qualification.zip"
MAX_OUTER_UNCOMPRESSED = 128 * 1024 * 1024
MAX_OUTER_ARCHIVE_COMPRESSED = 16 * 1024 * 1024
MAX_NESTED_COMPRESSED = 8 * 1024 * 1024
MAX_QUALIFICATION = 4 * 1024 * 1024


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def _status(*, scope: str) -> dict:
    return {
        "scope": scope,
        "status": "FAIL_CLOSED",
        "outer_integrity": "NOT_VERIFIED",
        "internal_checksum_status": "NOT_VERIFIED",
        "source_snapshot_coverage": "NOT_VERIFIED",
        "external_only_members": "NOT_VERIFIED",
        "nested_stage4_integrity": "NOT_VERIFIED",
        "stage5b_qualification_schema": "NOT_VERIFIED",
        "original_stage5b_replay": "NOT_RUN",
        "original_reduction_provenance": "NOT_VERIFIED",
        "historical_stage4_gate_q": "FAIL",
        "archived_stage5b_gate_q": "FAIL_RECORDED_NOT_REEXECUTED",
        "physical_gate_q": "FAIL",
        "physical_error_bound": "BOUND_NOT_CERTIFIED",
        "AC_P1": "PENDING",
        "contract": "NOT_FROZEN",
        "rust_fem": "NO_GO",
        "Gate_M": "NOT_VERIFIED",
        "failure_codes": [],
    }


def _fail(report: dict, code: str) -> dict:
    report["failure_codes"].append(code)
    return report


def _external_only_expected(paths: list[str]) -> bool:
    """Recognize *documented* five-member layout, not checksum coverage."""
    if len(paths) != 5 or NESTED_STAGE4 not in paths or "README-ko.md" not in paths:
        return False
    inputs = [p for p in paths if p.startswith("inputs/")]
    stage4 = [p for p in paths if p.startswith("stage4/")]
    return len(inputs) == 3 and len(stage4) == 1


def _preflight_stable_snapshot(
    path: Path,
    *,
    expected_sha256: str = STAGE5B_SHA256,
    nested_sha256: str = STAGE4_SHA256,
    expected_entries: int = 36,
    expected_sources: int = 13,
    nested_entries: int = 338,
    nested_sources: int = 20,
) -> dict:
    """Inspect authenticated archive bytes; never import or execute ZIP source.

    Overridable expectations exist solely for PUBLIC synthetic unit fixtures.
    Such overrides never produce original-physics certification.
    """
    report = _status(scope="ORIGINAL_SHA_PIN" if expected_sha256 == STAGE5B_SHA256
                     else "SYNTHETIC_TEST_EXPECTATIONS")
    if not path.is_file():
        return _fail(report, "ARCHIVE_UNAVAILABLE")
    if (len(expected_sha256) != 64 or len(nested_sha256) != 64
            or any(c not in "0123456789abcdef" for c in
                   (expected_sha256 + nested_sha256).lower())):
        return _fail(report, "INVALID_EXPECTED_DIGEST")
    if sha256_file(path) != expected_sha256.lower():
        return _fail(report, "EXTERNAL_SHA256_MISMATCH")

    # Reuse the published auditor's traversal, symlink, duplicate, zip-bomb,
    # checksum and format guards, but do not reinterpret unsupported Stage5B
    # manifest and its externally-pinned five unlisted files as an internal PASS.
    audit = inspect(path, expected_sha256=expected_sha256,
                    expected_entries=expected_entries,
                    max_bytes=MAX_OUTER_UNCOMPRESSED,
                    require_coverage=False)
    if audit["result"] == "FAIL":
        return _fail(report, "OUTER_ARCHIVE_AUDIT_FAILED")
    if audit["checksum_verified"] != expected_entries:
        return _fail(report, "OUTER_CHECKSUM_INCOMPLETE")
    report["outer_integrity"] = "EXTERNAL_SHA256_MATCH"
    report["internal_checksum_status"] = "PASS"
    report["checksum_entries_verified"] = expected_entries

    try:
        with zipfile.ZipFile(path) as z:
            names = {i.filename: i for i in z.infolist() if not i.is_dir()}
            if SUMS not in names or QUALIFICATION not in names or NESTED_STAGE4 not in names:
                return _fail(report, "STAGE5B_LAYOUT_UNRECOGNIZED")
            if names[SUMS].file_size > 1_000_000:
                return _fail(report, "CHECKSUM_LIST_TOO_LARGE")
            checksums, issues = parse_sum_lines(z.read(SUMS))
            if issues or len(checksums) != expected_entries:
                return _fail(report, "STAGE5B_CHECKSUM_FORMAT")
            covered = {"stage5b/" + rel for rel in checksums}
            if any(clean_path(rel) is None for rel in checksums):
                return _fail(report, "UNSAFE_SOURCE_PATH")
            sources = {p for p in names if p.startswith("stage5b/source_snapshot/")}
            if (len(sources) != expected_sources or
                    not sources.issubset(covered)):
                return _fail(report, "SOURCE_SNAPSHOT_NOT_COVERED")
            report["source_snapshot_coverage"] = "CHECKSUM_COVERED_NOT_SOURCE_ATTESTED"
            report["source_snapshots_covered"] = len(sources)
            extra = sorted(set(names) - covered - {SUMS})
            if not _external_only_expected(extra):
                return _fail(report, "UNLISTED_MEMBER_LAYOUT_UNRECOGNIZED")
            report["external_only_members"] = "5_COVERED_BY_OUTER_SHA_ONLY"
            # Qualification JSON is untrusted input. Parsing without duplicate
            # keys is NOT a semantic or physical Gate Q replay.
            if names[QUALIFICATION].file_size > MAX_QUALIFICATION:
                return _fail(report, "QUALIFICATION_TOO_LARGE")
            try:
                qualification = load_unique_json(z.read(QUALIFICATION))
            except (ValueError, UnicodeError, json.JSONDecodeError):
                return _fail(report, "QUALIFICATION_JSON_INVALID")
            if not isinstance(qualification, dict):
                return _fail(report, "QUALIFICATION_SCHEMA_UNRECOGNIZED")
            report["stage5b_qualification_schema"] = "STRUCTURAL_JSON_ONLY_NOT_VERIFIED"
            if names[NESTED_STAGE4].file_size > MAX_NESTED_COMPRESSED:
                return _fail(report, "NESTED_ZIP_TOO_LARGE")
            nested_bytes = z.read(NESTED_STAGE4)
            if digest(nested_bytes) != nested_sha256.lower():
                return _fail(report, "NESTED_STAGE4_SHA256_MISMATCH")
            with tempfile.TemporaryDirectory(prefix="ph08-private-stage5b-audit-") as tmp:
                stage4_file = Path(tmp) / "pinned-stage4.zip"
                stage4_file.write_bytes(nested_bytes)
                nested_audit = inspect(
                    stage4_file, expected_sha256=nested_sha256,
                    expected_entries=nested_entries, expected_sources=nested_sources,
                    max_bytes=MAX_OUTER_UNCOMPRESSED, require_coverage=True,
                )
                if (nested_audit["result"] != "PASS" or
                        nested_audit["checksum_verified"] != nested_entries or
                        nested_audit["source_hash_verified"] != nested_sources):
                    return _fail(report, "NESTED_STAGE4_AUDIT_FAILED")
            # Even a checksummed extra .py under source_snapshot is not
            # source-attested if omitted from the authoritative manifest.
            with zipfile.ZipFile(stage4_file) as inner:
                manifest_path = (
                    "boardsi-ph08-stage4-qualification/evidence/manifest.json"
                )
                snap_prefix = (
                    "boardsi-ph08-stage4-qualification/evidence/source_snapshot/"
                )
                manifest = load_unique_json(inner.read(manifest_path))
                registered = manifest.get("sources") if isinstance(manifest, dict) else None
                if (not isinstance(registered, dict) or
                        len(registered) != nested_sources or
                        any(not isinstance(name, str) or
                            clean_path(name) is None or "/" in name
                            for name in registered)):
                    return _fail(report, "NESTED_STAGE4_MANIFEST_UNRECOGNIZED")
                actual_snapshots = {
                    name for name in inner.namelist()
                    if name.startswith(snap_prefix) and not name.endswith("/")
                }
                recorded_snapshots = {
                    snap_prefix + name for name in registered
                }
                if actual_snapshots != recorded_snapshots:
                    return _fail(
                        report, "NESTED_STAGE4_SOURCE_INVENTORY_INCOMPLETE"
                    )
            report["nested_stage4_integrity"] = "PASS_INTEGRITY_ONLY"
            report["nested_checksums_verified"] = nested_entries
            report["nested_sources_verified"] = nested_sources
    except (OSError, ValueError, RuntimeError, zipfile.BadZipFile, EOFError):
        return _fail(report, "ZIP_FORMAT_OR_READ_FAILURE")

    report["status"] = "PASS_INTEGRITY_ONLY_NO_REPLAY"
    return report



def preflight(
    path: Path,
    *,
    expected_sha256: str = STAGE5B_SHA256,
    nested_sha256: str = STAGE4_SHA256,
    expected_entries: int = 36,
    expected_sources: int = 13,
    nested_entries: int = 338,
    nested_sources: int = 20,
) -> dict:
    """Audit a private authenticated copy; never reopen the mutable source.

    The stable snapshot is necessary: a mutable path or a re-targeted symlink
    otherwise permits a second ZIP open to analyze unpinned bytes (TOCTOU).
    The private snapshot has a bound on copied bytes and is destroyed on exit.
    """
    report = _status(scope="ORIGINAL_SHA_PIN" if expected_sha256 == STAGE5B_SHA256
                     else "SYNTHETIC_TEST_EXPECTATIONS")
    if not path.is_file():
        return _fail(report, "ARCHIVE_UNAVAILABLE")
    if (len(expected_sha256) != 64 or
            any(ch not in "0123456789abcdef" for ch in expected_sha256.lower())):
        return _fail(report, "INVALID_EXPECTED_DIGEST")
    try:
        with tempfile.TemporaryDirectory(prefix="ph08-authenticated-snapshot-") as tmp:
            snapshot = Path(tmp) / "staged-private-archive.zip"
            copied = 0
            h = hashlib.sha256()
            with path.open("rb") as source, snapshot.open("xb") as dest:
                while block := source.read(1024 * 1024):
                    copied += len(block)
                    if copied > MAX_OUTER_ARCHIVE_COMPRESSED:
                        return _fail(report, "ARCHIVE_COMPRESSED_SIZE_LIMIT")
                    h.update(block)
                    dest.write(block)
            if h.hexdigest() != expected_sha256.lower():
                return _fail(report, "EXTERNAL_SHA256_MISMATCH")
            return _preflight_stable_snapshot(
                snapshot, expected_sha256=expected_sha256,
                nested_sha256=nested_sha256,
                expected_entries=expected_entries,
                expected_sources=expected_sources,
                nested_entries=nested_entries,
                nested_sources=nested_sources,
            )
    except (OSError, RuntimeError, ValueError):
        return _fail(report, "PRIVATE_SNAPSHOT_FAILED")


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("archive", type=Path, help="private archive path, never uploaded")
    ap.add_argument("--output-json", type=Path, help="private sanitized result file")
    args = ap.parse_args(argv)
    try:
        result = preflight(args.archive)
    except (OSError, RuntimeError, ValueError):
        result = _fail(_status(scope="ORIGINAL_SHA_PIN"), "PREFLIGHT_EXCEPTION")
    output = json.dumps(result, sort_keys=True, indent=2, allow_nan=False) + "\n"
    if args.output_json:
        args.output_json.write_text(output, encoding="utf-8")
    print(output)
    return 0 if result["status"] == "PASS_INTEGRITY_ONLY_NO_REPLAY" else 2


if __name__ == "__main__":
    raise SystemExit(main())
