#!/usr/bin/env python3
"""F7: relocated native Qt/Worker probe. Does not certify clean OS or licensing."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

def sha(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for buf in iter(lambda: f.read(1048576), b""):
            h.update(buf)
    return h.hexdigest()

def main():
    if len(sys.argv) != 5:
        print("usage: probe.py RELOCATED_MODEL RELOCATED_WORKER A.frame B.frame")
        return 2
    model, worker, a, b = [Path(v).resolve() for v in sys.argv[1:]]
    if not all(p.is_file() for p in (model, worker, a, b)):
        print("missing required executable or synthetic model frame", file=sys.stderr)
        return 2
    env = dict(os.environ)
    qt_prefixes = [env.get("QT_PREFIX", ""), env.get("QT_HOST_PREFIX", "")]
    for key in list(env):
        if key.startswith("QT_") or key.startswith("QTDIR") or key in (
            "DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH", "CMAKE_PREFIX_PATH"
        ):
            env.pop(key, None)
    def normalized(p):
        return os.path.normcase(os.path.normpath(p.strip('"')))
    roots = [normalized(q) for q in qt_prefixes if q]
    def is_sdk_path(p):
        item = normalized(p)
        return any(item == root or item.startswith(root + os.sep) for root in roots)
    env["PATH"] = os.pathsep.join(
        p for p in env.get("PATH", "").split(os.pathsep)
        if p and not is_sdk_path(p)
    )
    if any(is_sdk_path(p) for p in env["PATH"].split(os.pathsep) if p):
        raise RuntimeError("Qt SDK path still present in child PATH")
    if sys.platform == "darwin":
        backend = "cocoa"
        plugin_dir = model.parent.parent / "PlugIns" / "platforms"
    elif os.name == "nt":
        backend = "windows"
        plugin_dir = model.parent / "platforms"
    else:
        raise RuntimeError("Windows/macOS only")
    env["QT_QPA_PLATFORM"] = backend
    # macdeployqt may leave an absolute SDK framework or LC_RPATH even when
    # Qt environment variables and PATH are stripped. Fail closed if any
    # shipped Mach-O executable/library/plugin refers to the build SDK.
    if sys.platform == "darwin":
        app_root = model.parents[2]
        qt_prefixes = [q for q in qt_prefixes if q]
        sigs = (bytes.fromhex("cffaedfe"), bytes.fromhex("feedfacf"),
                bytes.fromhex("cafebabe"), bytes.fromhex("bebafeca"))
        checked = 0
        for candidate in app_root.rglob("*"):
            if not candidate.is_file() or candidate.is_symlink():
                continue
            with candidate.open("rb") as source:
                magic = source.read(4)
            if magic not in sigs:
                continue
            checked += 1
            for args in (["otool", "-L", str(candidate)],
                         ["otool", "-l", str(candidate)]):
                info = subprocess.run(args, capture_output=True, text=True, check=True)
                for prefix in qt_prefixes:
                    if prefix in info.stdout:
                        raise RuntimeError(
                            "Absolute SDK linkage/rpath in bundled Mach-O: "
                            + str(candidate.relative_to(app_root))
                        )
        if checked < 2:
            raise RuntimeError("Packaged macOS Qt Mach-O dependencies not inspected")
    if not plugin_dir.is_dir() or not any(plugin_dir.iterdir()):
        print("Relocated Qt platform plugin missing", plugin_dir, file=sys.stderr)
        return 1
    result = subprocess.run(
        [str(model), "--selftest", str(worker), str(a), str(b)],
        cwd=model.parent, env=env, capture_output=True, text=True,
        errors="replace", timeout=90, check=False
    )
    evidence = {
        "scope": "M0 F7 relocatable Qt synthetic model/Worker only",
        "overall_f7": "BLOCKED",
        "not_proven": ["actual clean OS", "licensing", "redistribution", "installer"],
        "os": sys.platform, "qt_backend": backend,
        "model_sha256": sha(model), "worker_sha256": sha(worker),
        "platform_plugin_present": True,
        "qt_sdk_environment_paths_removed": True,
        "selftest_exit_code": result.returncode,
        "selftest_pass": result.returncode == 0
    }
    out = Path(os.environ.get("RUNNER_TEMP", ".")) / "m0-f7-relocation-result.json"
    out.write_text(json.dumps(evidence, indent=2) + "\n")
    print("F7 relocated selftest exit:", result.returncode)
    print(result.stderr[-1200:])
    print("F7 remains BLOCKED until a real clean OS and licensing review.")
    return 0 if result.returncode == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
