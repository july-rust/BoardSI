#!/usr/bin/env python3
"""Always-on public checkout guard for two SHA-pinned PRIVATE PH-08 archives.

Detects known exact raw ZIP bytes even if renamed or placed in a subdirectory;
also rejects the two known original basenames anywhere among tracked files.
This is NOT comprehensive DLP against recompression, obfuscation or extracted
sources. Run it as part of the EXISTING required docs job on every PR/push.
"""
from __future__ import annotations

import hashlib
from pathlib import Path, PurePosixPath
import subprocess
import sys

PRIVATE_NAMES = frozenset({
    "boardsi-ph08-stage4-qualification.zip",
    "boardsi-ph08-stage5b-evidence.zip",
})
PRIVATE_DIGESTS = frozenset({
    "6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9",
    "f4431dc1a13e356675bb0d8baf6d4f2be4bb7a93f46008ab4547c6b5beaacd36",
})


def tracked_paths(root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--cached", "--full-name", "-z"],
        check=True, capture_output=True, timeout=30,
    )
    return [p.decode("utf-8", errors="surrogateescape")
            for p in result.stdout.split(b"\0") if p]


def _digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def scan(root: Path, paths: list[str], digests=PRIVATE_DIGESTS):
    """Return counts only; do not echo paths potentially containing secrets."""
    checked = 0
    forbidden = 0
    for rel in paths:
        # Git tracked names are the authority, including nested and renamed
        # paths; hashes also catch exact protected ZIP contents under any name.
        if PurePosixPath(rel).name.lower() in PRIVATE_NAMES:
            forbidden += 1
            continue
        p = root / rel
        if p.is_symlink():
            # Do not follow tracked links outside the checkout.
            continue
        if not p.is_file():
            # Missing checkout files must not silently pass.
            forbidden += 1
            continue
        checked += 1
        if _digest(p) in digests:
            forbidden += 1
    return {"tracked_files_hashed": checked,
            "forbidden_or_missing_files": forbidden}


def main():
    root = Path(__file__).resolve().parents[2]
    try:
        report = scan(root, tracked_paths(root))
    except (OSError, subprocess.SubprocessError, ValueError):
        print("PH08_PRIVATE_CHECKOUT_GUARD_NOT_VERIFIED", file=sys.stderr)
        return 2
    if report["forbidden_or_missing_files"]:
        print("PH08_PRIVATE_ARCHIVE_OR_MISSING_TRACKED_FILE_DETECTED",
              file=sys.stderr)
        return 1
    print("PH08_KNOWN_ORIGINAL_ZIP_CHECK_PASS "
          f"tracked_regular_files={report['tracked_files_hashed']} "
          "scope=known_names_and_exact_raw_digests_only")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
