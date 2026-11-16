#!/usr/bin/env python3
"""Verify the immutable F004 v1 SQLite Golden, independent of Rust."""
from __future__ import annotations

import argparse
import base64
import hashlib
import sqlite3
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FROZEN = ROOT / "fixtures/bsi/v1/reference-v1.bsi.b85"
SHA256 = "e43f30933c3640141ad583d1d3b91b8416e099bfed089be2fa9671db3bf4c2d7"
LENGTH = 163840


def frozen() -> bytes:
    text = "".join(line.strip() for line in FROZEN.read_text(encoding="ascii").splitlines()
                   if line.strip() and not line.startswith("#"))
    binary = zlib.decompress(base64.b85decode(text))
    assert len(binary) == LENGTH, f"Golden length changed: {len(binary)}"
    assert hashlib.sha256(binary).hexdigest() == SHA256, "Golden SHA-256 mismatch"
    assert binary[:16] == b"SQLite format 3\x00", "not native SQLite"
    assert int.from_bytes(binary[68:72], "big") == 0x42534931, "wrong app_id"
    return binary


def verify(path: Path) -> None:
    known = frozen()
    assert path.read_bytes() == known, "independent Oracle diverged from immutable Golden"
    with sqlite3.connect(f"file:{path}?mode=ro", uri=True) as conn:
        assert conn.execute("PRAGMA user_version").fetchone()[0] == 1
        assert conn.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
        assert conn.execute("PRAGMA foreign_key_check").fetchall() == []
        assert conn.execute("SELECT COUNT(*) FROM geometry").fetchone()[0] == 1
        assert conn.execute("SELECT COUNT(*) FROM objects").fetchone()[0] == 4
        assert conn.execute("SELECT COUNT(*) FROM extensions").fetchone()[0] == 0
    print(f"F004_FROZEN_PASS bytes={LENGTH} sha256={SHA256}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--compare", type=Path, required=True)
    verify(parser.parse_args().compare)
