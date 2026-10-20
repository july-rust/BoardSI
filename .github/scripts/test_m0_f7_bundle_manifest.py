"""No-network adversarial tests for F7 deployed bundle evidence."""
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import m0_f7_bundle_manifest as manifest


class BundleIntegrityTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / "package"
        self.root.mkdir()
        (self.root / "platforms").mkdir()
        self.app = self.root / "boardsi-model"
        self.worker = self.root / "boardsi-worker"
        self.plugin = self.root / "platforms" / "qwindows.dll"
        self.app.write_bytes(b"test Qt app")
        self.worker.write_bytes(b"test Rust Worker")
        self.plugin.write_bytes(b"test Qt platform plugin")

    def test_exact_bundle_sha_and_roles(self):
        from hashlib import sha256
        result = manifest.inventory(self.root, self.app, self.worker, self.plugin)
        self.assertEqual(result["status"], "PASS_INTEGRITY_ONLY")
        self.assertEqual(result["file_count"], 3)
        self.assertEqual(result["entrypoints"]["qt_platform_plugin"], "platforms/qwindows.dll")
        self.assertEqual(
            next(f for f in result["files"] if f["path"] == "boardsi-worker")["sha256"],
            sha256(b"test Rust Worker").hexdigest())
        self.assertEqual(result["license_verification"], "NOT_PERFORMED")
        self.assertEqual(result["clean_os_installation"], "NOT_RUN")

    def test_mutation_changes_hash(self):
        first = manifest.inventory(self.root, self.app, self.worker, self.plugin)
        self.worker.write_bytes(b"changed Worker")
        second = manifest.inventory(self.root, self.app, self.worker, self.plugin)
        self.assertNotEqual(first["files"], second["files"])

    def test_reject_deleted_plugin(self):
        self.plugin.unlink()
        with self.assertRaisesRegex(ValueError, "missing"):
            manifest.inventory(self.root, self.app, self.worker, self.plugin)

    def test_reject_worker_outside_bundle(self):
        outsider = Path(self.tmp.name) / "external-worker"
        outsider.write_bytes(b"no")
        with self.assertRaisesRegex(ValueError, "outside bundle"):
            manifest.inventory(self.root, self.app, outsider, self.plugin)

    def test_symlink_escape_rejected(self):
        outsider = Path(self.tmp.name) / "external"
        outsider.write_bytes(b"unsafe")
        alias = self.root / "escape"
        try:
            alias.symlink_to(outsider)
        except (OSError, NotImplementedError) as exc:
            self.skipTest(f"Runner cannot create symlinks: {exc}")
        with self.assertRaisesRegex(ValueError, "escapes root"):
            manifest.inventory(self.root, self.app, self.worker, self.plugin)

    def test_internal_symlink_retarget_changes_manifest(self):
        versions = self.root / "framework" / "Versions"
        (versions / "A").mkdir(parents=True)
        (versions / "B").mkdir(parents=True)
        (versions / "A" / "core").write_bytes(b"VERSION A")
        (versions / "B" / "core").write_bytes(b"VERSION B")
        link = versions / "Current"
        try:
            link.symlink_to("A", target_is_directory=True)
        except (OSError, NotImplementedError) as exc:
            self.skipTest(f"Runner cannot create directory symlink: {exc}")
        first = manifest.inventory(self.root, self.app, self.worker, self.plugin)
        link.unlink()
        link.symlink_to("B", target_is_directory=True)
        second = manifest.inventory(self.root, self.app, self.worker, self.plugin)
        self.assertEqual(first["files"], second["files"])
        self.assertNotEqual(first["symlinks"], second["symlinks"])
        self.assertEqual(first["symlinks"][0]["raw_target"], "A")
        self.assertEqual(second["symlinks"][0]["raw_target"], "B")
        self.assertEqual(first["symlink_count"], 1)

    def test_budgets_fail_closed(self):
        with mock.patch.object(manifest, "MAX_BYTES", 3):
            with self.assertRaisesRegex(ValueError, "budget"):
                manifest.inventory(self.root, self.app, self.worker, self.plugin)
        with mock.patch.object(manifest, "MAX_FILES", 2):
            with self.assertRaisesRegex(ValueError, "many"):
                manifest.inventory(self.root, self.app, self.worker, self.plugin)


if __name__ == "__main__":
    unittest.main()
