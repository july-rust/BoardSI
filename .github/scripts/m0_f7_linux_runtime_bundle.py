#!/usr/bin/env python3
"""Build an SDK-free Linux M0 Qt/Worker runtime bundle.

This is a limited Linux container runtime proof, NOT a release, full clean OS
installation, license approval or Windows/macOS packaging certification.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

LINK = re.compile(r"^\s*(\S+)\s+=>\s+(\S+)(?:\s+\(|$)")
REQUIRED_QT = {"libQt6Core.so.6", "libQt6Gui.so.6", "libQt6Widgets.so.6"}
MAX_LIBS = 128
MAX_TOTAL_BYTES = 800_000_000


def read_dependencies(raw: str) -> dict[str, Path]:
    result: dict[str, Path] = {}
    for line in raw.splitlines():
        if "not found" in line:
            raise ValueError("Unresolved ELF dependency: " + line.strip())
        match = LINK.match(line)
        if not match:
            continue
        soname, target = match.groups()
        if not target.startswith("/"):
            raise ValueError("Nonabsolute ELF dependency: " + soname)
        p = Path(target)
        if soname in result and result[soname] != p:
            raise ValueError("Conflicting SONAME resolution: " + soname)
        result[soname] = p
    return result


def contained(candidate: Path, root: Path) -> bool:
    try:
        candidate.resolve(strict=True).relative_to(root.resolve(strict=True))
        return True
    except (ValueError, OSError, RuntimeError):
        return False


def fingerprint(path: Path) -> str:
    sha = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            sha.update(chunk)
    return sha.hexdigest()


def build(qt_prefix: Path, qt_exe: Path, worker: Path,
          platform_plugin: Path, out: Path) -> dict:
    qt_prefix = qt_prefix.resolve(strict=True)
    inputs = (qt_exe, worker, platform_plugin)
    if not all(p.is_file() for p in inputs):
        raise ValueError("Missing M0 runtime entrypoint or plugin")
    if not contained(platform_plugin, qt_prefix):
        raise ValueError("Qt platform plugin must originate in pinned SDK")
    if not qt_exe.name == "boardsi-qt-model-v2":
        raise ValueError("Unexpected M0 GUI entrypoint name")
    if out.exists() and any(out.iterdir()):
        raise ValueError("Refusing to reuse a nonempty output bundle")
    dest_bin = out / "bin"
    dest_lib = out / "lib"
    dest_plugins = out / "plugins" / "platforms"
    for item in (dest_bin, dest_lib, dest_plugins):
        item.mkdir(parents=True, exist_ok=True)
    shutil.copy2(qt_exe, dest_bin / qt_exe.name)
    shutil.copy2(worker, dest_bin / "boardsi-worker")
    shutil.copy2(platform_plugin, dest_plugins / "libqoffscreen.so")
    resolved: dict[str, Path] = {}
    for input_file in (qt_exe, platform_plugin):
        output = subprocess.run(["ldd", str(input_file)], check=True,
                                capture_output=True, text=True).stdout
        for soname, path in read_dependencies(output).items():
            if soname in resolved and resolved[soname].resolve() != path.resolve():
                raise ValueError("Different target libraries for the same SONAME: " + soname)
            resolved[soname] = path
    if not REQUIRED_QT.issubset(resolved):
        raise ValueError("Required Qt Core/Gui/Widgets dependencies are missing")
    for soname in REQUIRED_QT:
        if not contained(resolved[soname], qt_prefix):
            raise ValueError("Required Qt library unexpectedly resolved outside build SDK: " + soname)
    sdk_libs = [(soname, path) for soname, path in sorted(resolved.items())
                if contained(path, qt_prefix)]
    if not sdk_libs or len(sdk_libs) > MAX_LIBS:
        raise ValueError("No SDK libraries or unexpectedly large dependency graph")
    total = 0
    for soname, path in sdk_libs:
        if "/" in soname or soname in {".", ".."}:
            raise ValueError("Unsafe SONAME")
        size = path.stat().st_size
        total += size
        if total > MAX_TOTAL_BYTES:
            raise ValueError("Runtime dependency budget exceeded")
        shutil.copy2(path, dest_lib / soname)
    if not any((dest_lib / p).is_file() for p in REQUIRED_QT):
        raise ValueError("Core Qt runtime library not copied")
    items = sorted(p for p in out.rglob("*") if p.is_file())
    if len(items) > MAX_LIBS + 4:
        raise ValueError("Unbounded package")
    report = {
        "schema": "boardsi.m0.f7.linux-sdk-free.v1",
        "scope": "Linux isolated container; synthetic Qt model / independent Rust Worker",
        "f7_technical_step": "BUNDLE_PREPARED_ONLY",
        "overall_m0": "BLOCKED",
        "build_sdk_directory_mounted_in_container": False,
        "requires_container_execution": True,
        "external_clean_os_installation": "NOT_RUN",
        "license_and_redistribution_approval": "NOT_VERIFIED",
        "other_target_os": ["Windows x64 NOT_COVERED", "Windows ARM64 NOT_COVERED", "macOS ARM64 NOT_COVERED"],
        "file_count": len(items),
        "sdk_shared_libraries_copied": len(sdk_libs),
        "files": [{"relative_path": p.relative_to(out).as_posix(),
                   "bytes": p.stat().st_size,
                   "sha256": fingerprint(p)} for p in items],
    }
    (out.parent / "sdk-free-package-manifest.json").write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return report


def main() -> int:
    if len(sys.argv) != 6:
        print("usage: bundler QT_PREFIX QT_APP WORKER OFFSCREEN_PLUGIN DEST", file=sys.stderr)
        return 2
    info = build(*(Path(s) for s in sys.argv[1:]))
    print("SDK-free M0 runtime package constructed; files:", info["file_count"])
    print("F7 legal and other OS certification: NOT_VERIFIED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
