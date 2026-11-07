"""Negative/adversarial tests for actual SHA-bound docs-only Git classifier."""
from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from docs_only_classifier import allowed_document, classify_changes


class DocsOnlyClassifierTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix="boardsi-docs-only-")
        self.addCleanup(self.directory.cleanup)
        self.repo = Path(self.directory.name)
        self.git("init", "-q")
        self.git("config", "user.email", "ci-fixture@example.invalid")
        self.git("config", "user.name", "BoardSI test fixture")
        self.base = self.commit("empty baseline")

    def git(self, *args: str) -> str:
        return subprocess.check_output(
            ("git", *args), cwd=self.repo, stderr=subprocess.STDOUT, text=True
        ).strip()

    def write(self, name: str, text: str = "public synthetic fixture\n") -> None:
        path = self.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def commit(self, label: str) -> str:
        self.git("add", "-A")
        self.git("commit", "-q", "--allow-empty", "-m", label)
        return self.git("rev-parse", "HEAD")

    def reset(self) -> None:
        self.git("reset", "--hard", self.base)
        self.git("clean", "-fd")

    def assert_git(self, changes: dict[str, str], expected: bool) -> None:
        self.reset()
        for name, content in changes.items():
            self.write(name, content)
        head = self.commit("fixture")
        self.assertEqual(
            classify_changes(self.base, head, cwd=self.repo),
            expected,
            f"changed={sorted(changes)}",
        )

    def test_document_allowlist_is_exact_and_scoped(self) -> None:
        allowed = (
            "README.md",
            "CONTRIBUTING.md",
            "SECURITY.md",
            "docs/overview.md",
            "docs/a/b/readme.mdx",
            "docs/a/b/notes.rst",
            "apps/boardsi-qt-prototype/README.md",
            "crates/boardsi-import/README.md",
            "apps/a/b/README.md",
            "crates/a/b/README.md",
        )
        rejected = (
            ".github/workflows/qt-runtime.yml",
            ".github/actions/classify-docs-only/action.yml",
            "Cargo.toml",
            "Cargo.lock",
            "src/lib.rs",
            "tests/fixtures/example.asc",
            "docs/project-plan.json",
            "docs/README.md.rs",
            "docs/subdir/README.yaml",
            "apps/boardsi-qt-prototype/README.md.bak",
            "apps/a/src/lib.rs",
            "apps/a/docs/notes.md",
            "crates/boardsi-import/Cargo.toml",
            "other/README.md",
            "docs/../src/lib.rs",
            "docs//foo.md",
            "docs/a\nb.md",
            "apps/a/README.md\nsrc/main.rs",
        )
        for name in allowed:
            with self.subTest(allowed=name):
                self.assertTrue(allowed_document(name))
        for name in rejected:
            with self.subTest(rejected=name):
                self.assertFalse(allowed_document(name))

    def test_git_change_matrix(self) -> None:
        allowed = (
            "README.md",
            "CONTRIBUTING.md",
            "SECURITY.md",
            "docs/nested/status.md",
            "docs/api/notes.mdx",
            "docs/architecture/review.rst",
            "apps/boardsi-qt-prototype/README.md",
            "crates/boardsi-import/README.md",
            "apps/some/extra/README.md",
        )
        rejected = (
            ".github/workflows/qt-runtime.yml",
            ".github/actions/classify-docs-only/action.yml",
            "tools/validation/docs_only_classifier.py",
            "src/lib.rs",
            "Cargo.lock",
            "docs/project-plan.json",
            "crates/boardsi-import/src/lib.rs",
            "crates/boardsi-import/tests/fixtures/test.asc",
            "apps/boardsi-qt-prototype/CMakeLists.txt",
            "apps/boardsi-qt-prototype/README.md.bak",
        )
        for file in allowed:
            with self.subTest(allowed=file):
                self.assert_git({file: "hello\n"}, True)
        for file in rejected:
            with self.subTest(rejected=file):
                self.assert_git({file: "hello\n"}, False)
        self.assert_git(
            {"docs/changes.md": "text\n", "apps/x/src/main.rs": "fn main() {}\n"},
            False,
        )
        self.assert_git(
            {
                "docs/changes.md": "text\n",
                "apps/x/README.md": "text\n",
                "crates/y/README.md": "text\n",
            },
            True,
        )

    def test_rename_between_document_paths_is_not_docs_only(self) -> None:
        self.write("docs/old.md")
        self.base = self.commit("source docs")
        self.git("mv", "docs/old.md", "docs/new.md")
        head = self.commit("renamed docs")
        self.assertFalse(classify_changes(self.base, head, cwd=self.repo))

    def test_source_to_readme_rename_is_not_docs_only(self) -> None:
        self.write("apps/pkg/src/module.rs")
        self.base = self.commit("source")
        self.git("mv", "apps/pkg/src/module.rs", "apps/pkg/README.md")
        head = self.commit("source renamed to README")
        self.assertFalse(classify_changes(self.base, head, cwd=self.repo))

    def test_missing_zero_invalid_and_identical_shas_fail_closed(self) -> None:
        self.write("README.md")
        head = self.commit("doc")
        for base, tip in (
            ("", head),
            ("0" * 40, head),
            ("1" * 40, head),
            (self.base[:-1], head),
            (self.base, "0" * 40),
            (self.base, self.base),
        ):
            with self.subTest(base=base, head=tip):
                self.assertFalse(classify_changes(base, tip, cwd=self.repo))

    def test_non_regular_symlink_readme_is_rejected(self) -> None:
        target = self.repo / "apps" / "pkg" / "README.md"
        target.parent.mkdir(parents=True, exist_ok=True)
        try:
            target.symlink_to("../../src/lib.rs")
        except (NotImplementedError, OSError):
            self.skipTest("no symlink support on test platform")
        head = self.commit("symlink README")
        self.assertFalse(classify_changes(self.base, head, cwd=self.repo))

    def test_executable_bit_change_is_rejected(self) -> None:
        self.write("docs/help.md")
        self.base = self.commit("normal document")
        self.git("update-index", "--chmod=+x", "docs/help.md")
        head = self.commit("mode changed")
        self.assertFalse(classify_changes(self.base, head, cwd=self.repo))

    def test_deleting_old_document_is_docs_only(self) -> None:
        self.write("docs/old.md")
        self.base = self.commit("baseline")
        self.git("rm", "docs/old.md")
        head = self.commit("remove obsolete doc")
        self.assertTrue(classify_changes(self.base, head, cwd=self.repo))


if __name__ == "__main__":
    unittest.main()
