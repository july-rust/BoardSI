#!/usr/bin/env python3
"""Synthetic tests: private originals are never used or copied into CI."""
import hashlib
from pathlib import Path
import tempfile
import unittest

from ph08_private_archive_guard import scan


class PrivateArchiveGuardTests(unittest.TestCase):
    def test_nested_known_name_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "nested").mkdir()
            (root / "nested/boardsi-ph08-stage4-qualification.zip").write_bytes(b"FAKE")
            result = scan(root, ["nested/boardsi-ph08-stage4-qualification.zip"])
            self.assertEqual(result["forbidden_or_missing_files"], 1)

    def test_other_nested_known_name_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "subdir").mkdir()
            (root / "subdir/boardsi-ph08-stage5b-evidence.zip").write_bytes(b"FAKE")
            self.assertEqual(scan(root, ["subdir/boardsi-ph08-stage5b-evidence.zip"])
                             ["forbidden_or_missing_files"], 1)

    def test_renamed_exact_digest_detected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            payload = b"SYNTHETIC-NOT-AN-ORIGINAL-ARCHIVE"
            (root / "renamed.dat").write_bytes(payload)
            fake_hashes = frozenset({hashlib.sha256(payload).hexdigest()})
            result = scan(root, ["renamed.dat"], digests=fake_hashes)
            self.assertEqual(result["forbidden_or_missing_files"], 1)

    def test_unrelated_zip_is_allowed(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "synthetic.zip").write_bytes(b"SYNTHETIC")
            result = scan(root, ["synthetic.zip"])
            self.assertEqual(result["forbidden_or_missing_files"], 0)

    def test_missing_tracked_file_fails_closed(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.assertEqual(scan(root, ["gone.zip"])
                             ["forbidden_or_missing_files"], 1)

    def test_nested_symlink_not_followed(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "good.txt").write_bytes(b"synthetic")
            (root / "alias.dat").symlink_to(root / "good.txt")
            result = scan(root, ["alias.dat", "good.txt"])
            self.assertEqual(result["forbidden_or_missing_files"], 0)
            self.assertEqual(result["tracked_files_hashed"], 1)


if __name__ == "__main__":
    unittest.main()
