#!/usr/bin/env python3
"""Verify the immutable F004 v1 SQLite Goldens, independent of Rust Reader."""
from __future__ import annotations

import argparse
import base64
import hashlib
import sqlite3
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GOLDEN = ROOT / "fixtures/bsi/v1/reference-v1.bsi.b85"
GOLDEN_SHA = "e43f30933c3640141ad583d1d3b91b8416e099bfed089be2fa9671db3bf4c2d7"
NINE = ROOT / "fixtures/bsi/v1/all-nine-v1.bsi.b85"
NINE_SHA = "6fcda969a94766b1f25d0f171c1d986965c9d3efef0fdd1950ac093a134eccc9"
LENGTH = 163840


def decode(path: Path, sha: str) -> bytes:
    text = "".join(line.strip() for line in path.read_text(encoding="ascii").splitlines()
                   if line.strip() and not line.startswith("#"))
    binary = zlib.decompress(base64.b85decode(text))
    assert len(binary) == LENGTH, f"Golden length changed: {len(binary)}"
    assert hashlib.sha256(binary).hexdigest() == sha, "Golden SHA-256 mismatch"
    assert binary[:16] == b"SQLite format 3\x00", "not native SQLite"
    assert int.from_bytes(binary[68:72], "big") == 0x42534931, "wrong app_id"
    return binary


def verify(path: Path, source: Path, sha: str, nine: bool) -> None:
    known = decode(source, sha)
    assert path.read_bytes() == known, "generator output diverged from frozen Golden"
    with sqlite3.connect(f"file:{path}?mode=ro", uri=True) as conn:
        assert conn.execute("PRAGMA user_version").fetchone()[0] == 1
        assert conn.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
        assert conn.execute("PRAGMA foreign_key_check").fetchall() == []
        if nine:
            kinds = {kind for (kind,) in conn.execute("SELECT geometry_kind FROM geometry")}
            assert kinds == set(range(1, 10)), f"missing geometry kind(s): {kinds}"
            assert conn.execute("SELECT COUNT(*) FROM geometry").fetchone()[0] >= 9
        else:
            assert conn.execute("SELECT COUNT(*) FROM geometry").fetchone()[0] == 1
            assert conn.execute("SELECT COUNT(*) FROM objects").fetchone()[0] == 4
            assert conn.execute("SELECT COUNT(*) FROM extensions").fetchone()[0] == 0
    print(f"F004_FROZEN_PASS fixture={path.name} bytes={LENGTH} sha256={sha}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--compare", type=Path, required=True)
    parser.add_argument("--compare-full", type=Path)
    args = parser.parse_args()
    verify(args.compare, GOLDEN, GOLDEN_SHA, False)
    if args.compare_full is not None:
        verify(args.compare_full, NINE, NINE_SHA, True)
