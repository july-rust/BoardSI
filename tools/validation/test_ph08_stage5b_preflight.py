"""Public synthetic-only tests for PH-08 Stage 5B integrity preflight."""
from __future__ import annotations

from io import BytesIO
import hashlib
import json
from pathlib import Path
import stat
import tempfile
import unittest
import warnings
import zipfile

from ph08_stage5b_preflight import preflight, STAGE5B_SHA256


def sha(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def nested_fixture(*, bad_source=False) -> bytes:
    src = b"# artificial sample only\n"
    manifest = json.dumps({"sources": {"source.py": sha(src)}}).encode()
    items = {
        "boardsi-ph08-stage4-qualification/evidence/source_snapshot/source.py":
            b"tampered" if bad_source else src,
        "boardsi-ph08-stage4-qualification/evidence/manifest.json": manifest,
    }
    sums = "".join(
        f"{sha(value)}  {name.rsplit('boardsi-ph08-stage4-qualification/',1)[1]}\n"
        for name, value in items.items()
    ).encode()
    # When bad_source is true, independently recorded checksums must continue
    # to bind the original bytes, so nested digest verification rejects it.
    if bad_source:
        sums = "".join(
            f"{sha(src if name.endswith('source.py') else value)}  "
            f"{name.rsplit('boardsi-ph08-stage4-qualification/',1)[1]}\n"
            for name, value in items.items()
        ).encode()
    items["boardsi-ph08-stage4-qualification/SHA256SUMS"] = sums
    f = BytesIO()
    with zipfile.ZipFile(f, "w", compression=zipfile.ZIP_DEFLATED) as z:
        for name, value in items.items():
            z.writestr(name, value)
    return f.getvalue()


def make_fixture(path: Path, *, corrupt_checksum=False, uncovered_source=False,
                 stage4=None, wrong_extra=False, qualification=None,
                 bad_member=None, duplicate=False, symlink=False,
                 zipbomb=False) -> tuple[str, str]:
    stage4 = nested_fixture() if stage4 is None else stage4
    inputs = {
        "README-ko.md": b"public artificial archive",
        "inputs/boardsi-ph08-stage4-qualification.zip": stage4,
        "inputs/provenance.json": b"{}",
        "inputs/metadata.txt": b"synthetic",
        "stage4/historical.json": b"{}",
    }
    if wrong_extra:
        inputs["wrong-readme.txt"] = inputs.pop("README-ko.md")
    inside = {"stage5b/qualification.json":
              b'{"Gate_Q":"FAIL"}' if qualification is None else qualification}
    for i in range(13):
        inside[f"stage5b/source_snapshot/sample_{i:02d}.py"] = b"# synthetic source\n"
    for i in range(22):
        inside[f"stage5b/aux/filler_{i:02d}.txt"] = str(i).encode()
    checks = dict(inside)
    if uncovered_source:
        removed = "stage5b/source_snapshot/sample_00.py"
        del checks[removed]
        checks["stage5b/aux/extra_coverage.txt"] = b"extra"
        inside["stage5b/aux/extra_coverage.txt"] = b"extra"
    sums = "".join(
        f"{sha(v)}  {name.removeprefix('stage5b/')}\n"
        for name, v in checks.items()
    ).encode()
    inside["stage5b/SHA256SUMS"] = sums
    if corrupt_checksum:
        inside["stage5b/source_snapshot/sample_00.py"] = b"unchecksummed mutation"
    contents = {**inside, **inputs}
    if bad_member:
        contents[bad_member] = b"malicious"
    if zipbomb:
        contents["stage5b/aux/filler_00.txt"] = b"x" * (2 * 1024 * 1024)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as z:
        for name, value in contents.items():
            z.writestr(name, value)
        if duplicate:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore", UserWarning)
                z.writestr("stage5b/qualification.json", b"{}")
        if symlink:
            inf = zipfile.ZipInfo("unexpected-symlink")
            inf.create_system = 3
            inf.external_attr = (stat.S_IFLNK | 0o777) << 16
            z.writestr(inf, "/private")
    return sha(path.read_bytes()), sha(stage4)


class Stage5BPreflight(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.path = Path(self.tmp.name) / "synthetic-stage5b.zip"

    def run_fixture(self, **changes):
        h5, h4 = make_fixture(self.path, **changes)
        return preflight(self.path, expected_sha256=h5,
                         nested_sha256=h4 if changes.get("stage4") is None else
                         sha(nested_fixture()),
                         expected_entries=36, expected_sources=13,
                         nested_entries=2, nested_sources=1)

    def test_valid_synthetic_integrity_but_never_physical_pass(self):
        r = self.run_fixture()
        self.assertEqual(r["status"], "PASS_INTEGRITY_ONLY_NO_REPLAY")
        self.assertEqual(r["scope"], "SYNTHETIC_TEST_EXPECTATIONS")
        self.assertEqual(r["source_snapshots_covered"], 13)
        self.assertEqual(r["internal_checksum_status"], "PASS")
        self.assertEqual(r["external_only_members"], "5_COVERED_BY_OUTER_SHA_ONLY")
        self.assertEqual(r["stage5b_qualification_schema"],
                         "STRUCTURAL_JSON_ONLY_NOT_VERIFIED")
        self.assertEqual(r["original_stage5b_replay"], "NOT_RUN")
        self.assertEqual(r["physical_error_bound"], "BOUND_NOT_CERTIFIED")
        self.assertEqual(r["historical_stage4_gate_q"], "FAIL")
        self.assertEqual(r["rust_fem"], "NO_GO")
        self.assertEqual(r["nested_stage4_integrity"], "PASS_INTEGRITY_ONLY")

    def test_hardcoded_historical_pin_cannot_admit_synthetic_zip(self):
        make_fixture(self.path)
        r = preflight(self.path)
        self.assertEqual(r["status"], "FAIL_CLOSED")
        self.assertIn("EXTERNAL_SHA256_MISMATCH", r["failure_codes"])
        self.assertEqual(STAGE5B_SHA256,
            "f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36")

    def test_no_original_file(self):
        r = preflight(self.path)
        self.assertIn("ARCHIVE_UNAVAILABLE", r["failure_codes"])
        self.assertEqual(r["original_stage5b_replay"], "NOT_RUN")

    def test_inner_checksum_tamper(self):
        self.assertIn("OUTER_ARCHIVE_AUDIT_FAILED",
                      self.run_fixture(corrupt_checksum=True)["failure_codes"])

    def test_source_missing_from_sha256s(self):
        self.assertIn("SOURCE_SNAPSHOT_NOT_COVERED",
                      self.run_fixture(uncovered_source=True)["failure_codes"])

    def test_unlisted_member_layout_rejected(self):
        self.assertIn("UNLISTED_MEMBER_LAYOUT_UNRECOGNIZED",
                      self.run_fixture(wrong_extra=True)["failure_codes"])

    def test_unknown_path_rejected(self):
        self.assertIn("OUTER_ARCHIVE_AUDIT_FAILED",
                      self.run_fixture(bad_member="../private/source.py")["failure_codes"])

    def test_duplicate_member_rejected(self):
        self.assertIn("OUTER_ARCHIVE_AUDIT_FAILED",
                      self.run_fixture(duplicate=True)["failure_codes"])

    def test_symlink_member_rejected(self):
        self.assertIn("OUTER_ARCHIVE_AUDIT_FAILED",
                      self.run_fixture(symlink=True)["failure_codes"])

    def test_large_compression_ratio_rejected(self):
        self.assertIn("OUTER_ARCHIVE_AUDIT_FAILED",
                      self.run_fixture(zipbomb=True)["failure_codes"])

    def test_invalid_qualification_json(self):
        self.assertIn("QUALIFICATION_JSON_INVALID",
                      self.run_fixture(qualification=b"{invalid")["failure_codes"])

    def test_duplicate_qualification_keys(self):
        self.assertIn("QUALIFICATION_JSON_INVALID",
                      self.run_fixture(qualification=b'{"Gate_Q":"FAIL","Gate_Q":"PASS"}')["failure_codes"])

    def test_nested_sha_mismatch(self):
        altered = nested_fixture() + b"trailer"
        self.assertIn("NESTED_STAGE4_SHA256_MISMATCH",
                      self.run_fixture(stage4=altered)["failure_codes"])

    def test_nested_stage4_member_tamper(self):
        altered = nested_fixture(bad_source=True)
        # Pin modified fixture digest so that actual inner audit is reached.
        h5, h4 = make_fixture(self.path, stage4=altered)
        r = preflight(self.path, expected_sha256=h5, nested_sha256=h4,
                      expected_entries=36, expected_sources=13,
                      nested_entries=2, nested_sources=1)
        self.assertIn("NESTED_STAGE4_AUDIT_FAILED", r["failure_codes"])

    def test_expected_source_count_change_rejected(self):
        h5, h4 = make_fixture(self.path)
        r = preflight(self.path, expected_sha256=h5, nested_sha256=h4,
                      expected_entries=36, expected_sources=12,
                      nested_entries=2, nested_sources=1)
        self.assertIn("SOURCE_SNAPSHOT_NOT_COVERED", r["failure_codes"])

    def test_no_private_paths_or_source_in_result(self):
        r = self.run_fixture()
        serialized = json.dumps(r)
        self.assertNotIn(str(self.path), serialized)
        self.assertNotIn("source.py", serialized)
        self.assertNotIn("stage5b/aux/", serialized)


if __name__ == "__main__":
    unittest.main(verbosity=2)
