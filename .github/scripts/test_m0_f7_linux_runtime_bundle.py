"""Fail-closed unit tests for restricted Linux SDK-free M0 runtime packaging."""
import tempfile
import unittest
from pathlib import Path
from m0_f7_linux_runtime_bundle import read_dependencies, contained

class TestElFDependencies(unittest.TestCase):
    def test_linux_loader_and_qt_lines(self):
        got = read_dependencies("""linux-vdso.so.1 (0x01)
            libQt6Core.so.6 => /tmp/qt/lib/libQt6Core.so.6 (0x02)
            libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x03)
            /lib64/ld-linux-x86-64.so.2 (0x04)
        """)
        self.assertEqual(got["libQt6Core.so.6"], Path("/tmp/qt/lib/libQt6Core.so.6"))
        self.assertEqual(len(got), 2)

    def test_missing_dependency_refused(self):
        with self.assertRaisesRegex(ValueError, "Unresolved"):
            read_dependencies("libQt6Gui.so.6 => not found")

    def test_conflicting_soname_refused(self):
        with self.assertRaisesRegex(ValueError, "Conflicting"):
            read_dependencies("libA.so => /lib/a (0x01)\nlibA.so => /lib/b (0x02)")

    def test_nonabsolute_dependency_refused(self):
        with self.assertRaisesRegex(ValueError, "Nonabsolute"):
            read_dependencies("libQt6Gui.so.6 => ../../lib/fake (0x01)")

    def test_escape_via_link_refused(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d) / "sdk"
            root.mkdir()
            inside = root / "libQt6Core.so.6"
            inside.write_bytes(b"safe")
            escape = root / "link"
            escape.symlink_to("../outside")
            (Path(d) / "outside").write_bytes(b"not sdk")
            self.assertTrue(contained(inside, root))
            self.assertFalse(contained(escape, root))

if __name__ == "__main__":
    unittest.main()
