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
    # Audit exactly the native ARM64 slice. Universal Mach-O images may have
    # different load commands/dependencies in other architecture slices.
    # Resolve @rpath only against the executable -> dependency loading chain,
    # or the application's actual plugin entry chain. Never union rpaths
    # globally across unrelated bundled files.
    if sys.platform == "darwin":
        import re
        app_root = model.parents[2].resolve()
        exe_dir = model.parent.resolve()
        plugin_root = (app_root / "Contents" / "PlugIns").resolve()
        sigs = (bytes.fromhex("cffaedfe"), bytes.fromhex("feedfacf"),
                bytes.fromhex("cafebabe"), bytes.fromhex("bebafeca"))
        binaries = []
        for candidate in app_root.rglob("*"):
            if candidate.is_symlink() or not candidate.is_file():
                continue
            with candidate.open("rb") as source:
                if source.read(4) in sigs:
                    binaries.append(candidate)
        if len(binaries) < 2:
            raise RuntimeError("Packaged macOS Qt Mach-O dependencies not inspected")
        images = {candidate.resolve(): candidate for candidate in binaries}

        def otool(flag, binary):
            return subprocess.run(["otool", "-arch", "arm64", flag, str(binary)],
                                  capture_output=True, text=True, check=True).stdout

        def contained(path):
            try:
                path.resolve().relative_to(app_root)
                return True
            except ValueError:
                return False

        def expands(path, loader):
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
            # These two Apple OS locations are an explicit allowlist, not a
            # wildcard for Homebrew, /usr/local, or SDK frameworks.
            return path.startswith("/usr/lib/") or path.startswith("/System/Library/")

        own_rpaths = {}
        deps = {}
        for binary in binaries:
            resolved = []
            for command in otool("-l", binary).split("Load command "):
                if not re.search(r"(?m)^\s*cmd LC_RPATH\s*$", command):
                    continue
                match = re.search(r"(?m)^\s*path (.+?) \(offset \d+\)", command)
                if not match:
                    raise RuntimeError("Unparseable LC_RPATH: " + str(binary))
                root = expands(match.group(1), binary)
                if not contained(root):
                    raise RuntimeError("External LC_RPATH: " + str(root))
                resolved.append(root.resolve())
            own_rpaths[binary.resolve()] = resolved

            refs = []
            for line in otool("-L", binary).splitlines():
                # Ignore every architecture/file header, not only the first;
                # -arch arm64 selects the native slice for all load commands.
                if not line.strip() or not line[0].isspace():
                    continue
                dep = line.strip().split(" (", 1)[0]
                if not dep:
                    raise RuntimeError("Empty Mach-O install name: " + str(binary))
                refs.append(dep)
            if not refs:
                raise RuntimeError("Missing Mach-O dependency listing: " + str(binary))
            deps[binary.resolve()] = refs

        seen = set()
        dependency_count = 0
        def check_chain(binary, inherited=()):
            nonlocal dependency_count
            actual = binary.resolve()
            if actual not in images:
                raise RuntimeError("Uninspected bundled dylib: " + str(binary))
            roots = tuple(own_rpaths[actual]) + inherited
            marker = (actual, roots)
            if marker in seen:
                return
            seen.add(marker)
            for dep in deps[actual]:
                dependency_count += 1
                if dep.startswith("@rpath/"):
                    suffix = dep[len("@rpath/"):]
                    # Match dyld's ordered load-chain lookup, not some unrelated
                    # framework/plugin's runpath. The first existing root wins.
                    possible = [root / suffix for root in roots]
                    target = next((p for p in possible if p.is_file()), None)
                    if target is None or not contained(target):
                        raise RuntimeError("Unbundled load-chain @rpath: " + dep)
                elif dep.startswith("@loader_path") or dep.startswith("@executable_path"):
                    target = expands(dep, binary)
                    if not target.is_file() or not contained(target):
                        raise RuntimeError("Unbundled relative dependency: " + dep)
                elif dep.startswith("/"):
                    if approved_system(dep):
                        continue
                    target = Path(dep)
                    if not target.is_file() or not contained(target):
                        raise RuntimeError("External dylib: " + dep)
                else:
                    raise RuntimeError("Unsupported Mach-O dependency: " + dep)
                check_chain(target, roots)

        # Real launch entrypoints must resolve independently; Qt plugin
        # entrypoints inherit only the main executable's loader-chain roots.
        check_chain(model)
        check_chain(worker)
        for candidate in binaries:
            resolved = candidate.resolve()
            if resolved in {item[0] for item in seen}:
                continue
            if candidate.resolve().is_relative_to(plugin_root):
                check_chain(candidate, tuple(own_rpaths[model.resolve()]))
            else:
                check_chain(candidate)
        print("Audited ARM64 Mach-O images:", len(binaries),
              "load-chain dependency references:", dependency_count)
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
