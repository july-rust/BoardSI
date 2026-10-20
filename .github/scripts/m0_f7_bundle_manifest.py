#!/usr/bin/env python3
"""Deterministic inventory of relocated M0 Qt/Worker binaries.

An integrity/containment inventory, NOT Qt license clearance, clean-OS proof,
authenticode/notarization, or shipping approval.
"""
from __future__ import annotations

import hashlib
import os
from pathlib import Path

MAX_FILES = 20000
MAX_BYTES = 3 * 1024 * 1024 * 1024


def inside(path: Path, root: Path) -> bool:
    try:
        path.resolve(strict=True).relative_to(root.resolve(strict=True))
        return True
    except (ValueError, OSError, RuntimeError):
        return False


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as reader:
        for chunk in iter(lambda: reader.read(1048576), b""):
            h.update(chunk)
    return h.hexdigest()


def inventory(package: Path, executable: Path, worker: Path,
              platform_plugin: Path) -> dict:
    """Reject external links, missing components and non-contained entrypoints."""
    package = package.resolve(strict=True)
    if not package.is_dir():
        raise ValueError("Bundle root is not a directory")
    roots = {"qt_app": executable, "rust_worker": worker,
             "qt_platform_plugin": platform_plugin}
    for role, item in roots.items():
        if not item.is_file() or not inside(item, package):
            raise ValueError(f"{role} missing or outside bundle")
    found = []
    total = 0
    count = 0
    for item in sorted(package.rglob("*")):
        count += 1
        if count > MAX_FILES:
            raise ValueError("Bundle has too many filesystem entries")
        if item.is_symlink() and not inside(item, package):
            raise ValueError("Bundle symlink escapes root: " + str(item))
        if not item.is_file():
            continue
        if not inside(item, package):
            raise ValueError("Bundle binary resolves outside root")
        n = item.stat().st_size
        total += n
        if total > MAX_BYTES:
            raise ValueError("Bundle size budget exceeded")
        rel = item.relative_to(package).as_posix()
        found.append({"path": rel, "bytes": n, "sha256": digest(item)})
    if not found:
        raise ValueError("Empty bundle")
    entries = {e["path"]: e for e in found}
    role_map = {}
    for role, item in roots.items():
        key = item.relative_to(package).as_posix()
        if key not in entries:
            raise ValueError(role + " not present in inventory")
        role_map[role] = key
    # A manifest can authenticate the tested copy, not upstream licensing.
    return {
        "schema": "boardsi.m0.f7.bundle-integrity.v1",
        "status": "PASS_INTEGRITY_ONLY",
        "root_role": "relocated M0 Qt application directory",
        "files": found,
        "entrypoints": role_map,
        "file_count": len(found),
        "total_regular_file_bytes": total,
        "sha256_algorithm": "SHA-256",
        "license_verification": "NOT_PERFORMED",
        "legal_redistribution_approval": "NOT_VERIFIED",
        "clean_os_installation": "NOT_RUN",
        "commercial_release_signing": "NOT_RUN",
    }
