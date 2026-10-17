#!/usr/bin/env python3
"""Synthetic Git blob tests; never use or publish private original archives."""
import hashlib
from pathlib import Path
import subprocess
import tempfile
import unittest

from ph08_private_archive_guard import scan, tracked_index


def git(root, *args, data=None):
    result = subprocess.run(["git", "-C", str(root), *args],
                            input=data, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, check=True)
    return result.stdout.decode().strip()


class PrivateGitObjectGuardTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        git(self.root, "init", "-q")

    def add(self, rel, content):
        p = self.root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(content)
        git(self.root, "add", "--", rel)

    def test_nested_original_stage4_basename_blocked(self):
        self.add("nested/boardsi-ph08-stage4-qualification.zip", b"SYNTHETIC")
        r = scan(self.root, tracked_index(self.root))
        self.assertEqual(r["forbidden_known_private_objects"], 1)

    def test_nested_original_stage5b_basename_blocked(self):
        self.add("arbitrary/sub/boardsi-ph08-stage5b-evidence.zip", b"SYNTHETIC")
        self.assertEqual(scan(self.root, tracked_index(self.root))
                         ["forbidden_known_private_objects"], 1)

    def test_renamed_exact_git_blob_blocked(self):
        payload = b"SYNTHETIC-ORIGINAL-HASH-STANDIN"
        self.add("renamed.dat", payload)
        digest = hashlib.sha256(payload).hexdigest()
        self.assertEqual(scan(self.root, tracked_index(self.root),
                              digests=frozenset({digest}))
                         ["forbidden_known_private_objects"], 1)

    def test_git_blob_used_not_filtered_worktree_bytes(self):
        # Simulates .gitattributes eol rewriting on checkout: Git's stage-0
        # bytes differ from a filtered worktree copy. No private bytes involved.
        original_blob = b"SYNTHETIC-INDEX-CRLF\r\n"
        oid = git(self.root, "hash-object", "-w", "--stdin", data=original_blob)
        (self.root / "filtered.txt").write_bytes(b"SYNTHETIC-WORKTREE-LF\n")
        git(self.root, "update-index", "--add", "--cacheinfo",
            f"100644,{oid},filtered.txt")
        expected = hashlib.sha256(original_blob).hexdigest()
        self.assertEqual(scan(self.root, tracked_index(self.root),
                              digests=frozenset({expected}))
                         ["forbidden_known_private_objects"], 1)

    def test_unrelated_zip_allowed(self):
        self.add("synthetic.zip", b"PUBLIC-SYNTHETIC")
        self.assertEqual(scan(self.root, tracked_index(self.root))
                         ["forbidden_known_private_objects"], 0)

    def test_removed_worktree_cannot_hide_tracked_blob(self):
        payload = b"PUBLIC-SYNTHETIC-OBJECT"
        self.add("removed_from_worktree.txt", payload)
        (self.root / "removed_from_worktree.txt").unlink()
        digest = hashlib.sha256(payload).hexdigest()
        self.assertEqual(scan(self.root, tracked_index(self.root),
                              digests=frozenset({digest}))
                         ["forbidden_known_private_objects"], 1)

    def test_symlink_object_scanned_without_following_target(self):
        self.add("good.txt", b"PUBLIC-SYNTHETIC")
        (self.root / "alias").symlink_to(self.root / "good.txt")
        git(self.root, "add", "--", "alias")
        r = scan(self.root, tracked_index(self.root))
        self.assertEqual(r["forbidden_known_private_objects"], 0)
        self.assertEqual(r["tracked_git_blobs_hashed"], 2)

    def test_invalid_blob_oid_rejected(self):
        with self.assertRaises(ValueError):
            scan(self.root, [("bad", "100644", "not-a-git-object")])


if __name__ == "__main__":
    unittest.main()
