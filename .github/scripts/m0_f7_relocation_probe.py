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
        # GitHub's Windows target and GITHUB_PATH may use different separators.
        # os.path.normcase on Windows canonicalizes slash direction and case.
        return os.path.normcase(os.path.normpath(p.strip('"')))
    roots = [normalized(q) for q in qt_prefixes if q]
    def is_sdk_path(p):
        item = normalized(p)
        return any(item == root or item.startswith(root + os.sep) for root in roots)
    if os.name == "nt":
        # Exercise the exact mixed-slash scenario from GitHub's GITHUB_ENV vs
        # GITHUB_PATH: qt-native prefixes may use / while PATH uses \\.
        for root in roots:
            if not is_sdk_path(root.replace(os.sep, "/") + "/bin"):
                raise RuntimeError("Windows mixed-slash Qt SDK PATH scrub failed")
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
    if os.name == "nt":
        # Refuse a deceptively passing smoke test with a host-x64 Qt DLL inside
        # a Windows ARM64 bundle (or the reverse). Parse the PE header directly.
        import struct
        required = ("Qt6Core.dll", "Qt6Gui.dll", "Qt6Widgets.dll")
        for filename in required:
            if not (model.parent / filename).is_file():
                raise RuntimeError("Missing relocated Qt runtime: " + filename)
        if not (plugin_dir / "qwindows.dll").is_file():
            raise RuntimeError("Missing relocated Windows QPA plugin")
        expected = {"X64": 0x8664, "ARM64": 0xAA64}.get(
            os.environ.get("RUNNER_ARCH", "").upper()
        )
        if expected is None:
            raise RuntimeError("Unknown native Windows runner architecture")
        def machine(path):
            with path.open("rb") as source:
                dos = source.read(64)
                if len(dos) != 64 or dos[:2] != b"MZ":
                    raise RuntimeError("Not a Windows PE image: " + str(path))
                pe_off = struct.unpack_from("<I", dos, 60)[0]
                if pe_off < 64 or pe_off > 1048576:
                    raise RuntimeError("Invalid PE header offset: " + str(path))
                source.seek(pe_off)
                header = source.read(6)
                if len(header) != 6 or header[:4] != bytes((80, 69, 0, 0)):
                    raise RuntimeError("Invalid PE signature: " + str(path))
                return struct.unpack_from("<H", header, 4)[0]
        # windeployqt may place DLLs in nested plugin subdirectories.
        binaries = [model, worker, *model.parent.rglob("*.dll")]
        for binary in binaries:
            actual = machine(binary)
            if actual != expected:
                raise RuntimeError(
                    f"Non-target PE machine 0x{actual:04x} for {binary.name}; "
                    f"expected 0x{expected:04x}"
                )
    env["QT_QPA_PLATFORM"] = backend
    # Enforce bundled-or-Apple-system resolution for *every* shipped Mach-O
    # install name and LC_RPATH. Testing only the original SDK prefix can miss
    # /opt/homebrew, /usr/local, and other hosted-runner dylib fallback paths.
    if sys.platform == "darwin":
        import re
        app_root = model.parents[2].resolve()
        exe_dir = model.parent.resolve()
        sigs = (bytes.fromhex("cffaedfe"), bytes.fromhex("feedfacf"),
                bytes.fromhex("cafebabe"), bytes.fromhex("bebafeca"))
        binaries = []
        for candidate in app_root.rglob("*"):
            if not candidate.is_file() or candidate.is_symlink():
                continue
            with candidate.open("rb") as source:
                magic = source.read(4)
            if magic in sigs:
                binaries.append(candidate)
        if len(binaries) < 2:
            raise RuntimeError("Packaged macOS Qt Mach-O dependencies not inspected")

        def otool(*args):
            return subprocess.run(["otool", *args], capture_output=True,
                                  text=True, check=True).stdout

        def contained(p):
            try:
                p.resolve().relative_to(app_root)
                return True
            except ValueError:
                return False

        def expand_location(path, loader):
            if path == "@loader_path":
                return loader.parent
            if path.startswith("@loader_path/"):
                return loader.parent / path[len("@loader_path/"):]
            if path == "@executable_path":
                return exe_dir
            if path.startswith("@executable_path/"):
                return exe_dir / path[len("@executable_path/"):]
            if path.startswith("/"):
                return Path(path)
            raise RuntimeError("Unsupported Mach-O dependency location: " + path)

        def approved_system(path):
            # Explicit Apple system locations only. Third-party system-wide
            # frameworks, Homebrew and /usr/local are NOT approved.
            return path.startswith("/usr/lib/") or path.startswith("/System/Library/")

        rpaths = {}
        for binary in binaries:
            resolved = []
            for command in otool("-l", str(binary)).split("Load command "):
                if not re.search(r"(?m)^\s*cmd LC_RPATH\s*$", command):
                    continue
                match = re.search(r"(?m)^\s*path (.+?) \(offset \d+\)", command)
                if not match:
                    raise RuntimeError("Unparseable LC_RPATH: " + str(binary))
                loc = expand_location(match.group(1), binary)
                if not contained(loc):
                    raise RuntimeError("External LC_RPATH: " + str(loc))
                resolved.append(loc.resolve())
            rpaths[binary.resolve()] = resolved

        # A library can inherit runpaths from its loading chain; checking all
        # *bundled* runpath roots is conservative for presence, while refusing
        # every external runpath. It does not claim a clean OS installation.
        all_bundle_roots = list({str(root): root
                                 for roots in rpaths.values()
                                 for root in roots}.values())
        dependency_count = 0
        for binary in binaries:
            listing = otool("-L", str(binary)).splitlines()
            if len(listing) < 2:
                raise RuntimeError("Missing Mach-O dependency listing: " + str(binary))
            for line in listing[1:]:
                dep = line.strip().split(" (", 1)[0]
                if not dep:
                    continue
                dependency_count += 1
                if dep.startswith("@rpath/"):
                    suffix = dep[len("@rpath/"):]
                    targets = [root / suffix for root in all_bundle_roots]
                    if not targets:
                        raise RuntimeError("Unresolved @rpath: " + dep)
                    if not any(contained(target) and target.is_file()
                               for target in targets):
                        raise RuntimeError("Unbundled @rpath dependency: " + dep)
                elif dep.startswith("@loader_path") or dep.startswith("@executable_path"):
                    target = expand_location(dep, binary)
                    if not contained(target) or not target.is_file():
                        raise RuntimeError("Unbundled relative dependency: " + dep)
                elif dep.startswith("/"):
                    if not approved_system(dep):
                        target = Path(dep)
                        if not contained(target) or not target.is_file():
                            raise RuntimeError("Unapproved external dylib: " + dep)
                else:
                    raise RuntimeError("Unexpected Mach-O dependency: " + dep)
        if dependency_count == 0:
            raise RuntimeError("No macOS Mach-O dependencies were audited")
        print("Audited macOS Mach-O images:", len(binaries),
              "dependency references:", dependency_count,
              "bundle-only rpaths:", len(all_bundle_roots))
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
