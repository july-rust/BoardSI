#!/usr/bin/env python3
"""Strict in-container probe for a Linux Qt/Worker bundle without the build SDK.

Run ONLY inside an isolated, network-disabled Linux runtime image. A pass
does not qualify Windows/macOS, third-party licenses or consumer installers.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

RUNTIME = Path("/runtime")
EVIDENCE = Path("/evidence")
MAX_MEMORY = str(8 * 1024**3)


def checksum(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for data in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(data)
    return digest.hexdigest()


def main() -> int:
    manifest_file = EVIDENCE / "sdk-free-package-manifest.json"
    manifest = json.loads(manifest_file.read_text(encoding="utf-8"))
    if manifest.get("schema") != "boardsi.m0.f7.linux-sdk-free.v1":
        raise ValueError("Unexpected SDK-free manifest schema")
    for item in manifest["files"]:
        path = (RUNTIME / item["relative_path"])
        if not path.is_file() or not path.resolve().is_relative_to(RUNTIME.resolve()):
            raise ValueError("Bundle file missing or escaping container: " + item["relative_path"])
        if checksum(path) != item["sha256"] or path.stat().st_size != item["bytes"]:
            raise ValueError("Bundled ELF/plugin hash or size changed")
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
    for program in (executable, plugin):
        checked = subprocess.run(["ldd", str(program)], env=env,
                                 capture_output=True, text=True, timeout=30)
        if checked.returncode or "not found" in checked.stdout or "not found" in checked.stderr:
            raise ValueError("Unresolved Linux runtime dependency: " + checked.stdout[-2000:])
        if "/runtime/lib/libQt6" not in checked.stdout:
            raise ValueError("Qt SDK-free library resolution not evidenced")
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
        "bundle_files_sha256_validated": len(manifest["files"]),
        "qt_library_resolution": "ONLY /runtime/lib for pinned Qt6",
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
