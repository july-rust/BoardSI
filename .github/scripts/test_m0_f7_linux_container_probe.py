"""Adversarial unit tests for independent M0 F7 in-container verifier."""
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from m0_f7_linux_container_probe import (
    MANDATORY, checksum, verify_bundle, verify_qt_linkage
)


def valid_fixture(root: Path) -> dict:
    entries = []
    for rel in sorted(MANDATORY):
        f = root / rel
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_bytes(("content:" + rel).encode("utf-8"))
        entries.append({
            "relative_path": rel,
            "bytes": f.stat().st_size,
            "sha256": checksum(f),
        })
    return {
        "schema": "boardsi.m0.f7.linux-sdk-free.v1",
        "file_count": len(entries),
        "files": entries,
    }


class TestIndependentBundleInventory(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / "runtime"
        self.root.mkdir()
        self.manifest = valid_fixture(self.root)

    def tearDown(self):
        self.temp.cleanup()

    def test_exact_full_tree_pass(self):
        self.assertEqual(verify_bundle(self.root, self.manifest), len(MANDATORY))

    def test_manifest_omission_fails(self):
        self.manifest["files"].pop()
        self.manifest["file_count"] -= 1
        with self.assertRaisesRegex(ValueError, "complete runtime tree"):
            verify_bundle(self.root, self.manifest)

    def test_duplicate_entry_fails(self):
        self.manifest["files"].append(dict(self.manifest["files"][0]))
        self.manifest["file_count"] += 1
        with self.assertRaisesRegex(ValueError, "duplicate"):
            verify_bundle(self.root, self.manifest)

    def test_extra_unvalidated_file_fails(self):
        (self.root / "lib" / "libQt6Evil.so.6").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "complete runtime tree"):
            verify_bundle(self.root, self.manifest)

    def test_symlink_inside_runtime_fails(self):
        existing = self.root / "lib" / "libQt6Core.so.6"
        spare = existing.with_name("real-copy")
        existing.rename(spare)
        existing.symlink_to(spare.name)
        with self.assertRaisesRegex(ValueError, "symlink"):
            verify_bundle(self.root, self.manifest)

    def test_changed_payload_fails(self):
        f = self.root / "lib" / "libQt6Core.so.6"
        f.write_bytes(b"wrong")
        with self.assertRaisesRegex(ValueError, "hash/size mismatch"):
            verify_bundle(self.root, self.manifest)

    def test_traversal_fails(self):
        self.manifest["files"][0]["relative_path"] = "../escape"
        with self.assertRaisesRegex(ValueError, "traversal"):
            verify_bundle(self.root, self.manifest)

    def test_absolute_path_fails(self):
        self.manifest["files"][0]["relative_path"] = "/etc/passwd"
        with self.assertRaisesRegex(ValueError, "traversal"):
            verify_bundle(self.root, self.manifest)


class TestQtLibraryProvenance(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / "runtime"
        (self.root / "lib").mkdir(parents=True)
        (self.root / "lib" / "libQt6Core.so.6").write_bytes(b"qt")

    def tearDown(self):
        self.temp.cleanup()

    def test_entire_qt_map_inside_package(self):
        lib = self.root / "lib" / "libQt6Core.so.6"
        result = verify_qt_linkage(
            "libQt6Core.so.6 => " + str(lib) + " (0xabc)\n", self.root)
        self.assertEqual(result["libQt6Core.so.6"], str(lib))

    def test_qt_secondary_lib_from_system_fails(self):
        with tempfile.TemporaryDirectory() as d:
            external = Path(d) / "libQt6Gui.so.6"
            external.write_bytes(b"other qt")
            lib = self.root / "lib" / "libQt6Core.so.6"
            data = (f"libQt6Core.so.6 => {lib} (0x01)\n"
                    f"libQt6Gui.so.6 => {external} (0x02)\n")
            with self.assertRaisesRegex(ValueError, "outside validated runtime"):
                verify_qt_linkage(data, self.root)

    def test_unparsed_qt_mapping_fails(self):
        with self.assertRaisesRegex(ValueError, "Unparseable"):
            verify_qt_linkage("libQt6Core.so.6 @unknown", self.root)

    def test_missing_qt_mapping_fails(self):
        with self.assertRaisesRegex(ValueError, "Qt Core"):
            verify_qt_linkage("libc.so.6 => /lib/libc.so.6 (0x1)", self.root)


if __name__ == "__main__":
    unittest.main()
