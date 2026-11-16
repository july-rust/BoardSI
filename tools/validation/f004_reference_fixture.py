#!/usr/bin/env python3
"""F004 independent Python/SQLite v1 Oracle. No Rust Writer is invoked.

This recipe is a checked-in, reviewable source for the immutable first Golden.
The frozen .bsi image is stored as zlib+Base85 to keep repository APIs text-only.
"""
from __future__ import annotations

import argparse
import hashlib
import sqlite3
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCHEMA = ROOT / "schemas/bsi/v1.sql"
UUID = "00000000-0000-4000-8000-000000000004"
POINTS = ((12345678901, -12345678901), (12345679001, -12345678801))
SOURCE_DIGEST = bytes([7]) * 32


def i128(n: int) -> bytes:
    return n.to_bytes(16, "little", signed=True)


def rational(n: int, d: int) -> bytes:
    return i128(n) + i128(d)


def field(fid: int, typ: int, value: bytes) -> bytes:
    return struct.pack("<HHI", fid, typ, len(value)) + value


def trace_blob() -> bytes:
    points = struct.pack("<I", len(POINTS))
    for x, y in POINTS:
        points += struct.pack("<Iqq", 0, x, y)
    payload = struct.pack("<HH", 1, 2)
    payload += field(1, 7, points)
    payload += field(2, 5, rational(1, 100000))
    return (b"BSIG" + struct.pack("<HHIQ", 1, 1, 0, len(payload))
            + hashlib.sha256(payload).digest() + payload)


def generate(output: Path) -> None:
    if output.exists():
        raise FileExistsError(f"never overwrite a previously generated Golden: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    try:
        with sqlite3.connect(output) as conn:
            conn.executescript(SCHEMA.read_text(encoding="utf-8"))
            conn.execute(
                "INSERT INTO file_meta VALUES(?,?,?,?,?,?,?,?,?,?)",
                (1, UUID, 1, 0, 1, 0, "Pcb", "F004 independent Python SQL",
                 "1", "2026-11-16T00:00:00Z"),
            )
            conn.execute(
                "INSERT INTO source_identity VALUES(?,?,?,?,?,?,?)",
                (1, "Synthetic", None, "1", "F004-oracle", "synthetic-fixture", "1"),
            )
            conn.execute(
                "INSERT INTO source_files VALUES(?,?,?,?,?)",
                (1, 0, "synthetic/golden-v1.asc", SOURCE_DIGEST, 0),
            )
            for dim in "GNPSRM":
                conn.execute("INSERT INTO import_capability VALUES(?,?,?)",
                             (1, dim, "Missing"))
            for idx, (kind, key) in enumerate(
                (("Frame", "f"), ("Layer", "L1"),
                 ("Net", "NET_A"), ("Trace", "T1")), 1
            ):
                conn.execute("INSERT INTO objects VALUES(?,?,?,?,?,?)",
                             (idx, 1, kind, key, "", "F004 independent fixture"))
            conn.execute(
                "INSERT INTO coordinate_frames VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)",
                (1, "um", i128(1), i128(1000000),
                 i128(0), i128(1), i128(0), i128(1),
                 "Orthogonal", 0, None, 0, 0),
            )
            conn.execute(
                "INSERT INTO layers VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)",
                (2, 0, "L1", "Copper", None, "Missing", None, None,
                 "Missing", None, None),
            )
            conn.execute("INSERT INTO nets VALUES(?,?,?)", (3, "NET_A", "Signal"))
            conn.execute(
                "INSERT INTO geometry(object_id,geometry_kind,codec_version,envelope,layer_id,net_id) "
                "VALUES(?,?,?,?,?,?)",
                (4, 1, 1, trace_blob(), 2, 3),
            )
            conn.execute("INSERT INTO geometry_refs VALUES(?,?,?,?)",
                         (4, 0, 1, "Frame"))
            conn.commit()
            assert conn.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
            assert conn.execute("PRAGMA foreign_key_check").fetchall() == []
        content = output.read_bytes()
        assert content[:16] == b"SQLite format 3\x00"
        assert len(content) <= 512 * 1024 * 1024
        print(f"F004_INDEPENDENT_ORACLE file={output.name} bytes={len(content)} "
              f"sha256={hashlib.sha256(content).hexdigest()} trace={POINTS}")
    except Exception:
        output.unlink(missing_ok=True)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    generate(parser.parse_args().output)
