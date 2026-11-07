#!/usr/bin/env python3
"""Fail-closed, source-tree-aware classification of documentation-only Git changes.

Used by .github/actions/classify-docs-only/action.yml. A successful return
means the named commit comparison changed *only* regular documentation files.
Do not inspect working-tree files or trust GitHub's event filename summaries.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

SHA = re.compile(r"[0-9a-f]{40}\Z")
ROOT_DOCS = frozenset(("README.md", "CONTRIBUTING.md", "SECURITY.md"))
DOC_EXTENSIONS = frozenset((".md", ".mdx", ".rst"))
RAW = re.compile(
    rb":(000000|100644) (000000|100644) [0-9a-f]{40} [0-9a-f]{40} [AMD]\Z"
)


def allowed_document(path: str) -> bool:
    """All and only explicitly permitted documentation paths."""
    if path in ROOT_DOCS:
        return True
    parts = path.split("/")
    if (
        not parts
        or any(part in ("", ".", "..") for part in parts)
        or any(ord(char) < 32 or ord(char) == 127 for char in path)
    ):
        return False
    if parts[0] == "docs" and len(parts) >= 2:
        return Path(parts[-1]).suffix in DOC_EXTENSIONS
    # A terminal component README is documentation, but arbitrary .md files
    # or source/JSON/fixture files inside apps/ or crates/ are not.
    if parts[0] in ("apps", "crates") and len(parts) >= 3:
        return parts[-1] == "README.md"
    return False


def _git(*args: str, cwd: str | Path | None = None) -> bytes | None:
    try:
        process = subprocess.run(
            ("git", *args),
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    return process.stdout if process.returncode == 0 else None


def classify_changes(base: str, head: str, cwd: str | Path | None = None) -> bool:
    """Return True only for verified, non-renamed, normal-file docs changes."""
    if not SHA.fullmatch(base) or not SHA.fullmatch(head):
        return False
    if base == "0" * 40 or head == "0" * 40 or base == head:
        return False

    # A rename of any file, including docs -> docs, is deliberately not
    # certified as a documentation-only change. Do not use --no-renames here.
    renames = _git(
        "diff", "--name-only", "-z", "--diff-filter=RC",
        "--find-renames=1%", "--find-copies",
        base, head, cwd=cwd,
    )
    if renames is None or renames:
        return False

    # --no-renames below sees all original deletes/adds independently.
    # Reject symlinks, mode changes, submodules and unexpected raw statuses.
    raw = _git("diff", "--no-renames", "--raw", "-z", base, head, cwd=cwd)
    if raw is None or not raw:
        return False
    records = raw.split(b"\0")
    if records.pop() != b"" or len(records) % 2:
        return False

    for index in range(0, len(records), 2):
        metadata, name = records[index : index + 2]
        if not RAW.fullmatch(metadata):
            return False
        try:
            path = name.decode("utf-8", errors="strict")
        except UnicodeDecodeError:
            return False
        if not allowed_document(path):
            return False
    return True


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print("Usage: docs_only_classifier.py BASE_SHA HEAD_SHA", file=sys.stderr)
        return 2
    if classify_changes(argv[1], argv[2]):
        print("Verified documentation-only Git diff: allow native-job skip.")
        return 0
    print("Cannot prove docs-only Git diff: retain full validation.")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
