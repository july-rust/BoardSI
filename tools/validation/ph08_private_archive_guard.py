#!/usr/bin/env python3
"""Required-CI guard for known SHA-pinned original PRIVATE PH-08 ZIP objects.

Checks stage-0 Git INDEX blobs, not filter-modified checkout files. This
detects exact SHA-256 bytes under any Git path and known private basenames
in any tracked directory. Not comprehensive DLP against transformed, embedded,
unpacked or recompressed private material. No private ZIP data is in tests.
"""
from __future__ import annotations

import hashlib
import re
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
OID_PATTERN = re.compile(r"[0-9a-f]{40}|[0-9a-f]{64}")


def tracked_index(root: Path) -> list[tuple[str, str, str]]:
    p = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--stage", "--full-name", "-z"],
        check=True, capture_output=True, timeout=30,
    )
    records = []
    for raw in p.stdout.split(b"\0"):
        if not raw:
            continue
        try:
            prefix, raw_path = raw.split(b"\t", 1)
            mode, oid, stage = prefix.decode("ascii").split(" ")
            rel = raw_path.decode("utf-8", "surrogateescape")
        except (ValueError, UnicodeDecodeError) as e:
            raise ValueError("invalid Git index record") from e
        if (stage != "0" or mode not in ("100644", "100755", "120000", "160000")
                or not OID_PATTERN.fullmatch(oid) or not rel):
            raise ValueError("unsupported Git index record")
        records.append((rel, mode, oid))
    return records


def _git_blob_digest(root: Path, oid: str) -> str:
    # Explicitly read the immutable Git object; checkout filters/.gitattributes
    # can otherwise change bytes before a worktree path is hashed.
    p = subprocess.Popen(
        ["git", "-C", str(root), "cat-file", "blob", oid],
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
    )
    h = hashlib.sha256()
    try:
        assert p.stdout is not None
        for chunk in iter(lambda: p.stdout.read(1024 * 1024), b""):
            h.update(chunk)
        if p.wait(timeout=20) != 0:
            raise ValueError("Git blob could not be verified")
        return h.hexdigest()
    finally:
        if p.poll() is None:
            p.kill()
            p.wait(timeout=5)
        if p.stdout:
            p.stdout.close()


def scan(root: Path, records, digests=PRIVATE_DIGESTS):
    """Fail closed on index gaps; never echo secret filenames/contents."""
    checked = 0
    forbidden = 0
    for rel, mode, oid in records:
        if PurePosixPath(rel).name.lower() in PRIVATE_NAMES:
            forbidden += 1
            continue
        # Gitlink is a referenced external COMMIT, not an in-tree blob.
        # Do not treat it as a scanned file or make DLP claims for its content.
        if mode == "160000":
            continue
        if mode not in ("100644", "100755", "120000") or not OID_PATTERN.fullmatch(oid):
            raise ValueError("invalid index blob metadata")
        checked += 1
        if _git_blob_digest(root, oid) in digests:
            forbidden += 1
    return {"tracked_git_blobs_hashed": checked,
            "forbidden_known_private_objects": forbidden}


def main():
    root = Path(__file__).resolve().parents[2]
    try:
        report = scan(root, tracked_index(root))
    except (OSError, subprocess.SubprocessError, ValueError):
        print("PH08_PRIVATE_GIT_OBJECT_CHECK_NOT_VERIFIED", file=sys.stderr)
        return 2
    if report["forbidden_known_private_objects"]:
        print("PH08_KNOWN_PRIVATE_ARCHIVE_GIT_OBJECT_DETECTED", file=sys.stderr)
        return 1
    print("PH08_KNOWN_PRIVATE_GIT_BLOB_CHECK_PASS "
          f"tracked_blobs={report['tracked_git_blobs_hashed']} "
          "scope=known_names_and_exact_stage0_object_digests_only")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
