#!/usr/bin/env python3
"""Fail-closed integrity/provenance auditor for BoardSI PH-08 evidence ZIPs.

Independent preflight only. This tool does NOT validate electrostatic fields,
energy certification, the original HMM Gate Q, or Rust FEM.

Usage:
  python ph08_evidence_audit.py evidence.zip --expected-sha256 HEX \
      --expected-entries 338 --expected-sources 20 --json result.json

The member paths in SHA256SUMS are interpreted relative to its directory.
Any unsupported manifest schema is reported NOT_VERIFIED, never passed.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import sys
import zipfile
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from zoneinfo import ZoneInfo

HEX64 = re.compile(r"^[0-9a-fA-F]{64}$")
SUM_ROW = re.compile(r"^([0-9a-fA-F]{64}) (?: |\*)(.+)$")
CHUNK_SIZE = 1024 * 1024


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while part := f.read(CHUNK_SIZE):
            h.update(part)
    return h.hexdigest()


def sha256_member(z: zipfile.ZipFile, member: zipfile.ZipInfo) -> str:
    h = hashlib.sha256()
    with z.open(member) as f:
        while part := f.read(CHUNK_SIZE):
            h.update(part)
    return h.hexdigest()


def clean_path(path: str) -> str | None:
    """Reject noncanonical archive paths, including traversal and OS ambiguity."""
    if (not path or "\\" in path or "\x00" in path or
            path.startswith("/") or ":" in path or path.endswith("/")):
        return None
    bits = path.split("/")
    if any(part in ("", ".", "..") for part in bits):
        return None
    if str(PurePosixPath(path)) != path:
        return None
    return path


def parse_sum_lines(content: bytes) -> tuple[dict[str, str], list[str]]:
    errors, recorded = [], {}
    try:
        lines = content.decode("utf-8").splitlines()
    except UnicodeDecodeError:
        return {}, ["SHA256SUMS is not UTF-8"]
    for line_no, line in enumerate(lines, 1):
        if not line.strip():
            continue
        m = SUM_ROW.fullmatch(line)
        if not m:
            errors.append(f"SHA256SUMS line {line_no}: malformed digest line")
            continue
        digest, member = m.group(1).lower(), m.group(2)
        if clean_path(member) is None:
            errors.append(f"SHA256SUMS line {line_no}: unsafe path")
        elif member in recorded:
            errors.append(f"SHA256SUMS line {line_no}: duplicate member {member}")
        else:
            recorded[member] = digest
    return recorded, errors


def source_records(manifest: object) -> tuple[list[tuple[str, str | None]] | None, str]:
    """Recognize only explicit source inventory schemas; never guess keys."""
    if not isinstance(manifest, dict):
        return None, "manifest root is not a JSON object"
    keys = [k for k in ("sources", "source_files", "source") if k in manifest]
    if len(keys) != 1:
        return None, "no unique recognized sources/source_files/source key"
    obj = manifest[keys[0]]
    if not isinstance(obj, list):
        return None, f"manifest {keys[0]} is not a list"
    records = []
    for i, record in enumerate(obj):
        if isinstance(record, str):
            path, digest = record, None
        elif isinstance(record, dict) and isinstance(record.get("path"), str):
            path, digest = record["path"], record.get("sha256")
            if digest is not None and (not isinstance(digest, str) or not HEX64.fullmatch(digest)):
                return None, f"manifest source[{i}] has invalid sha256"
        else:
            return None, f"manifest source[{i}] schema unsupported"
        if clean_path(path) is None:
            return None, f"manifest source[{i}] invalid path"
        records.append((path, digest.lower() if digest is not None else None))
    if len({p for p, _ in records}) != len(records):
        return None, "manifest contains duplicate source paths"
    return records, "recognized"


def inspect(path: Path, expected_sha256: str | None = None,
            expected_entries: int | None = None, expected_sources: int | None = None,
            max_bytes: int = 2_000_000_000, max_ratio: float = 1000.0,
            require_coverage: bool = True) -> dict:
    fail: list[str] = []
    not_verified: list[str] = []
    warning: list[str] = []
    now = datetime.now(timezone.utc)
    out = {
        "tool": "BoardSI PH-08 independent ZIP preflight v1.0",
        "scope": "Archive integrity, SHA256SUMS completeness and explicitly recognized source inventory only",
        "excludes": ["Original HMM mathematical or numerical Gate Q", "PDE solve",
                     "Floating-point-certified energy bounds", "Rust FEM"],
        "archive": str(path),
        "timestamp_utc": now.isoformat(),
        "timestamp_kst": now.astimezone(ZoneInfo("Asia/Seoul")).isoformat(),
        "archive_sha256": None,
        "expected_sha256": expected_sha256,
        "member_count": 0,
        "checksum_entries": 0,
        "checksum_verified": 0,
        "source_entries": None,
        "source_hash_verified": 0,
        "unlisted_members": [],
        "failures": fail,
        "not_verified": not_verified,
        "warnings": warning,
        "historical_original_gate_q": "FAIL (unchanged)",
        "AC_P1": "PENDING",
        "ph08_contract": "NOT_FROZEN",
        "ph08_rust_fem": "NO_GO",
    }
    if not path.is_file():
        fail.append("input ZIP does not exist")
        out["result"] = "FAIL"
        return out
    out["archive_sha256"] = sha256_file(path)
    if expected_sha256 is None:
        not_verified.append("external known-good archive SHA-256 not supplied")
    elif not HEX64.fullmatch(expected_sha256):
        fail.append("invalid expected SHA-256")
    elif out["archive_sha256"].lower() != expected_sha256.lower():
        fail.append("archive SHA-256 does not match expected")
    try:
        with zipfile.ZipFile(path, "r") as z:
            infos = z.infolist()
            out["member_count"] = len(infos)
            members: dict[str, zipfile.ZipInfo] = {}
            total = 0
            for info in infos:
                if info.is_dir():
                    if clean_path(info.filename[:-1]) is None:
                        fail.append(f"unsafe ZIP directory path: {info.filename!r}")
                    continue
                name = clean_path(info.filename)
                if name is None:
                    fail.append(f"unsafe ZIP member path: {info.filename!r}")
                    continue
                if name in members:
                    fail.append(f"duplicate ZIP member: {name}")
                    continue
                mode = info.external_attr >> 16
                if stat.S_IFMT(mode) == stat.S_IFLNK:
                    fail.append(f"symlink ZIP member: {name}")
                    continue
                if info.flag_bits & 0x1:
                    fail.append(f"encrypted ZIP member: {name}")
                    continue
                total += info.file_size
                if info.file_size > max_bytes or total > max_bytes:
                    fail.append("ZIP expansion byte limit exceeded")
                if info.file_size and info.file_size / max(1, info.compress_size) > max_ratio:
                    fail.append(f"ZIP compression ratio limit exceeded: {name}")
                members[name] = info
            if fail:
                out["result"] = "FAIL"
                return out
            sum_names = [name for name in members if PurePosixPath(name).name == "SHA256SUMS"]
            if len(sum_names) == 0:
                # Older Chat-only Stage 5C/5D archives used a different manifest.
                # This is a separate integrity check, NEVER an HMM evidence audit.
                manifests = [n for n in members if PurePosixPath(n).name in
                             ("boardsi-ph08-stage5c-manifest.json", "boardsi-ph08-stage5d-manifest.json")]
                if len(manifests) != 1:
                    fail.append("no SHA256SUMS and no uniquely recognizable Chat-only manifest")
                    out["result"] = "FAIL"
                    return out
                name = manifests[0]
                if members[name].file_size > 16_000_000:
                    fail.append("Chat-only manifest size exceeds limit")
                    out["result"] = "FAIL"
                    return out
                out["format"] = "Chat-only Stage 5C/5D bundle (not original HMM)"
                try:
                    data = json.loads(z.read(members[name]))
                except (ValueError, UnicodeDecodeError):
                    fail.append("invalid Chat-only manifest JSON")
                    out["result"] = "FAIL"
                    return out
                inventory = data.get("files_sha256", data.get("sha256")) if isinstance(data, dict) else None
                if not isinstance(inventory, dict) or not inventory:
                    fail.append("unrecognized Chat-only file digest inventory")
                    out["result"] = "FAIL"
                    return out
                parent = str(PurePosixPath(name).parent)
                prefix = "" if parent == "." else parent + "/"
                known = set()
                for relative, digest in inventory.items():
                    if not isinstance(relative, str) or clean_path(relative) is None or not isinstance(digest, str) or not HEX64.fullmatch(digest):
                        fail.append("invalid Chat-only manifest entry")
                        continue
                    full = prefix + relative
                    if full in known:
                        fail.append("duplicate Chat-only inventory path")
                        continue
                    known.add(full)
                    if full not in members:
                        fail.append(f"Chat-only manifest references missing member: {full}")
                    elif sha256_member(z, members[full]) != digest.lower():
                        fail.append(f"Chat-only SHA-256 mismatch: {full}")
                    else:
                        out["checksum_verified"] += 1
                out["checksum_entries"] = len(inventory)
                out["unlisted_members"] = sorted(set(members) - known - {name})
                if out["unlisted_members"]:
                    not_verified.append("Chat-only files omitted from manifest")
                if expected_entries is not None or expected_sources is not None:
                    not_verified.append("PH-08 original expected entry/source counts not applicable to Chat-only bundle")
                out["result"] = "FAIL" if fail else "NOT_VERIFIED" if not_verified else "PASS"
                return out
            if len(sum_names) != 1:
                fail.append(f"expected exactly one SHA256SUMS, found {len(sum_names)}")
                out["result"] = "FAIL"
                return out
            out["format"] = "PH-08 SHA256SUMS-based evidence archive"
            sums_name = sum_names[0]
            if members[sums_name].file_size > 16_000_000:
                fail.append("SHA256SUMS size exceeds limit")
                out["result"] = "FAIL"
                return out
            prefix = str(PurePosixPath(sums_name).parent)
            prefix = "" if prefix == "." else prefix + "/"
            checksum, parse_errors = parse_sum_lines(z.read(members[sums_name]))
            fail.extend(parse_errors)
            out["checksum_entries"] = len(checksum)
            if expected_entries is not None and len(checksum) != expected_entries:
                fail.append(f"checksum count {len(checksum)} != expected {expected_entries}")
            checked = set()
            for relative, expected in checksum.items():
                name = prefix + relative
                if name not in members:
                    fail.append(f"SHA256SUMS references missing member: {name}")
                    continue
                actual = sha256_member(z, members[name])
                if actual != expected:
                    fail.append(f"SHA256SUMS digest mismatch: {name}")
                else:
                    out["checksum_verified"] += 1
                checked.add(name)
            unlisted = sorted(set(members) - checked - {sums_name})
            out["unlisted_members"] = unlisted
            if unlisted and require_coverage:
                not_verified.append(f"{len(unlisted)} non-checksummed ZIP member(s)")
            elif unlisted:
                warning.append(f"{len(unlisted)} non-checksummed ZIP member(s)")
            manifest_names = [p for p in members if p == prefix + "evidence/manifest.json"]
            if len(manifest_names) == 1 and members[manifest_names[0]].file_size > 16_000_000:
                fail.append("source manifest size exceeds limit")
            elif len(manifest_names) != 1:
                not_verified.append("evidence/manifest.json absent from expected SHA256SUMS-root layout")
            else:
                try:
                    manifest = json.loads(z.read(members[manifest_names[0]]))
                except (ValueError, UnicodeDecodeError):
                    fail.append("invalid JSON in evidence/manifest.json")
                else:
                    sources, why = source_records(manifest)
                    if sources is None:
                        not_verified.append("source inventory unsupported: " + why)
                    else:
                        out["source_entries"] = len(sources)
                        if expected_sources is not None and len(sources) != expected_sources:
                            fail.append(f"source count {len(sources)} != expected {expected_sources}")
                        for relative, digest in sources:
                            name = prefix + relative
                            if name not in members:
                                fail.append(f"source inventory references missing member: {name}")
                            elif name not in checked:
                                not_verified.append(f"source missing from SHA256SUMS: {name}")
                            elif digest is not None and sha256_member(z, members[name]) != digest:
                                fail.append(f"source inventory sha256 mismatch: {name}")
                            elif name in checked and (prefix + relative) in members:
                                out["source_hash_verified"] += 1
            if expected_sources is not None and out["source_entries"] is None:
                not_verified.append("expected source count could not be checked")
    except (zipfile.BadZipFile, EOFError, RuntimeError, OSError, ValueError) as exc:
        fail.append(f"ZIP read/decompression error: {type(exc).__name__}: {exc}")
    out["result"] = "FAIL" if fail else "NOT_VERIFIED" if not_verified else "PASS"
    return out


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("--expected-sha256")
    parser.add_argument("--expected-entries", type=int)
    parser.add_argument("--expected-sources", type=int)
    parser.add_argument("--max-bytes", type=int, default=2_000_000_000)
    parser.add_argument("--json", type=Path)
    args = parser.parse_args(argv)
    result = inspect(args.archive, args.expected_sha256, args.expected_entries,
                     args.expected_sources, args.max_bytes)
    if args.json:
        args.json.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return {"PASS": 0, "NOT_VERIFIED": 1, "FAIL": 2}[result["result"]]


if __name__ == "__main__":
    sys.exit(main())
