#!/usr/bin/env python3
"""Strict in-container M0 Linux Qt/Worker validation without the build SDK.

The independent verifier rejects incomplete/duplicate manifests, changed files,
symlinks, Qt libraries from non-bundle paths, and missing resource restrictions.
Pass is Linux synthetic execution evidence only, NOT F7/M0 release clearance.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import subprocess
import sys

RUNTIME = Path("/runtime")
EVIDENCE = Path("/evidence")
MAX_MEMORY = str(8 * 1024**3)
MAX_FILES = 132
QT_LINE = re.compile(r"^\s*(libQt6\S+)\s+=>\s+(\S+)(?:\s+\(|$)")
MANDATORY = {
    "bin/boardsi-qt-model-v2", "bin/boardsi-worker",
    "plugins/platforms/libqoffscreen.so",
    "lib/libQt6Core.so.6", "lib/libQt6Gui.so.6", "lib/libQt6Widgets.so.6",
}


def checksum(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for data in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(data)
    return digest.hexdigest()


def verify_bundle(root: Path, manifest: dict) -> int:
    if manifest.get("schema") != "boardsi.m0.f7.linux-sdk-free.v1":
        raise ValueError("Unexpected SDK-free manifest schema")
    records = manifest.get("files")
    if not isinstance(records, list) or not 0 < len(records) <= MAX_FILES:
        raise ValueError("Empty or oversized runtime manifest")
    if not root.is_dir() or root.is_symlink():
        raise ValueError("Runtime root absent or a symlink")
    regular = set()
    for candidate in root.rglob("*"):
        if candidate.is_symlink():
            raise ValueError("Bundle symlink forbidden: " + str(candidate))
        if candidate.is_file():
            regular.add(candidate.relative_to(root).as_posix())
        elif not candidate.is_dir():
            raise ValueError("Unsupported bundle filesystem entry")
    claimed = set()
    for entry in records:
        if not isinstance(entry, dict) or set(entry) != {"relative_path", "bytes", "sha256"}:
            raise ValueError("Manifest record shape invalid")
        relative = entry["relative_path"]
        if not isinstance(relative, str) or not relative or "\\" in relative:
            raise ValueError("Unsafe manifest path")
        parts = PurePosixPath(relative)
        if parts.is_absolute() or ".." in parts.parts or "." in parts.parts:
            raise ValueError("Manifest traversal or absolute path")
        if parts.as_posix() != relative or relative in claimed:
            raise ValueError("Noncanonical or duplicate manifest path")
        claimed.add(relative)
        path = root / relative
        if not path.is_file() or path.is_symlink() or not path.resolve(strict=True).is_relative_to(root.resolve(strict=True)):
            raise ValueError("Missing or external manifest file: " + relative)
        expected_size = entry["bytes"]
        expected_hash = entry["sha256"]
        if (type(expected_size) is not int or expected_size < 0 or
                not isinstance(expected_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", expected_hash)):
            raise ValueError("Invalid file size or hash in manifest")
        if path.stat().st_size != expected_size or checksum(path) != expected_hash:
            raise ValueError("Runtime file hash/size mismatch: " + relative)
    if claimed != regular:
        raise ValueError("Manifest fails to cover complete runtime tree")
    if not MANDATORY.issubset(claimed):
        raise ValueError("Required Qt libraries/app/Worker/plugin missing")
    if manifest.get("file_count") != len(claimed):
        raise ValueError("Manifest file-count mismatch")
    return len(regular)


def verify_qt_linkage(ldd_stdout: str, root: Path) -> dict[str, str]:
    """Verify ALL resolved Qt6 SONAMEs, not merely one safe Qt library."""
    if "not found" in ldd_stdout:
        raise ValueError("Unresolved dependency")
    mappings: dict[str, str] = {}
    for line in ldd_stdout.splitlines():
        match = QT_LINE.match(line)
        if not match:
            if "libQt6" in line:
                raise ValueError("Unparseable Qt library resolution: " + line.strip())
            continue
        soname, location = match.groups()
        target = Path(location)
        if not target.is_absolute() or not target.is_file():
            raise ValueError("Missing/nonabsolute Qt dependency: " + soname)
        allowed = root / "lib"
        if not target.resolve(strict=True).is_relative_to(allowed.resolve(strict=True)):
            raise ValueError("Qt resolved from outside validated runtime: " + soname)
        if soname in mappings and mappings[soname] != str(target):
            raise ValueError("Conflicting Qt dependency mapping: " + soname)
        mappings[soname] = str(target)
    if not mappings or "libQt6Core.so.6" not in mappings:
        raise ValueError("Qt Core not mapped to runtime package")
    return mappings


def main() -> int:
    manifest = json.loads((EVIDENCE / "sdk-free-package-manifest.json").read_text(encoding="utf-8"))
    validated = verify_bundle(RUNTIME, manifest)
    if Path("/qt").exists() or Path("/qt-sdk").exists() or Path("/usr/local/Qt").exists():
        raise ValueError("Unexpected build SDK mount in test container")
    if platform.machine() not in {"x86_64", "amd64"}:
        raise ValueError("Linux target is not x86_64")
    memory = Path("/sys/fs/cgroup/memory.max").read_text().strip()
    if memory != MAX_MEMORY:
        raise ValueError("No real enforced 8 GiB memory cgroup")
    cpu = Path("/sys/fs/cgroup/cpu.max").read_text().strip()
    quota, period = cpu.split()
    if quota == "max" or int(quota) > 2 * int(period):
        raise ValueError("CPU limit not enforced to <= 2")
    env = dict(os.environ)
    env.update(QT_QPA_PLATFORM="offscreen", QT_PLUGIN_PATH="/runtime/plugins",
               QT_QPA_PLATFORM_PLUGIN_PATH="/runtime/plugins/platforms",
               LD_LIBRARY_PATH="/runtime/lib")
    executable = RUNTIME / "bin" / "boardsi-qt-model-v2"
    worker = RUNTIME / "bin" / "boardsi-worker"
    plugin = RUNTIME / "plugins" / "platforms" / "libqoffscreen.so"
    qt_paths: dict[str, dict[str, str]] = {}
    for program in (executable, plugin):
        checked = subprocess.run(["ldd", str(program)], env=env,
                                 capture_output=True, text=True, timeout=30)
        if checked.returncode:
            raise ValueError("Linux dynamic loader inspection failed")
        qt_paths[program.relative_to(RUNTIME).as_posix()] = verify_qt_linkage(
            checked.stdout + "\n" + checked.stderr, RUNTIME)
    # The actual executable must use the complete expected Qt Widgets stack.
    app_libraries = qt_paths["bin/boardsi-qt-model-v2"]
    for mandatory in ("libQt6Core.so.6", "libQt6Gui.so.6", "libQt6Widgets.so.6"):
        if mandatory not in app_libraries:
            raise ValueError("Qt Widgets runtime mapping missing: " + mandatory)
    run = subprocess.run([str(executable), "--selftest", str(worker),
                          str(EVIDENCE / "a.frame"), str(EVIDENCE / "b.frame")],
                         cwd=str(EVIDENCE), env=env, capture_output=True,
                         text=True, errors="replace", timeout=90)
    report = {
        "schema": "boardsi.m0.f7.sdk-free-container-evidence.v1",
        "limited_linux_result": "PASS" if run.returncode == 0 else "FAIL",
        "overall_f7": "BLOCKED",
        "overall_m0": "BLOCKED",
        "source_head_sha": os.environ.get("SOURCE_HEAD_SHA", "UNKNOWN"),
        "architecture": platform.machine(),
        "os_release": Path("/etc/os-release").read_text()[:300],
        "memory_max_bytes": int(memory),
        "cpu_max": cpu,
        "sdk_mount_present": False,
        "network_isolated_by_docker_workflow": True,
        "bundle_files_sha256_validated": validated,
        "qt_library_resolution": qt_paths,
        "selftest_exit": run.returncode,
        "selftest_stdout_tail": run.stdout[-900:],
        "selftest_stderr_tail": run.stderr[-900:],
        "not_proven": [
            "Windows x64/ARM64 or macOS ARM64 clean-host installs",
            "commercial installer or OS signing",
            "complete Qt third-party licensing or legal distribution",
            "actual ~8 GiB reference PC/SSD and real CAD/SI workloads",
        ],
    }
    (EVIDENCE / "m0-f7-sdk-free-linux-result.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if run.returncode == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
