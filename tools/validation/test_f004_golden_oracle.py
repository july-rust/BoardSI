#!/usr/bin/env python3
"""F_004 independent SQLite+BSIG v1 Golden oracle (no Rust Writer/Reader imports).

Checks fixed bytes, exact envelopes/TLV types, coordinate ticks, source
identities, all nine geometry kinds, reference indices and error vectors.
This is a structural oracle, not physical copper/contact correctness.
"""
from __future__ import annotations

from collections import Counter
import hashlib
import math
from pathlib import Path
import sqlite3
import struct

ROOT = Path(__file__).resolve().parents[2]
BSI_DIR = ROOT / "schemas" / "bsi"

TYPES = {
    1: ((1, 7), (2, 5)),
    2: ((1, 6), (2, 6), (3, 6), (4, 1), (5, 5)),
    3: ((1, 8), (2, 9)),
    4: ((1, 8),),
    5: ((1, 11),),
    6: ((1, 6), (2, 10)),
    7: ((1, 6), (2, 2), (3, 12), (4, 9)),
    8: ((1, 5), (2, 1), (3, 12)),
    9: ((1, 8), (2, 9)),
}


def u16(buf: bytes, at: int) -> int:
    return struct.unpack_from("<H", buf, at)[0]


def u32(buf: bytes, at: int) -> int:
    return struct.unpack_from("<I", buf, at)[0]


def u64(buf: bytes, at: int) -> int:
    return struct.unpack_from("<Q", buf, at)[0]


def rational(raw: bytes) -> tuple[int, int]:
    assert len(raw) == 32
    n = int.from_bytes(raw[:16], "little", signed=True)
    d = int.from_bytes(raw[16:], "little", signed=True)
    assert n != -(1 << 127) and d > 0 and math.gcd(n, d) == 1
    return (n, d)


def point(raw: bytes) -> tuple[int, int, int]:
    assert len(raw) == 20
    frame, x, y = struct.unpack("<Iqq", raw)
    return frame, x, y


def boundary(raw: bytes) -> int:
    assert len(raw) >= 4
    count = u32(raw, 0)
    assert 0 < count <= 1_000_000
    pos = 4
    segments = []
    for _ in range(count):
        tag = raw[pos]
        size = {1: 41, 2: 62}[tag]
        seg = raw[pos : pos + size]
        assert len(seg) == size
        if tag == 1:
            a, b = point(seg[1:21]), point(seg[21:41])
        else:
            point(seg[1:21])
            a, b = point(seg[21:41]), point(seg[41:61])
            assert seg[61] in (0, 1)
        segments.append((a, b))
        pos += size
    assert pos == len(raw)
    assert all(segments[i][1] == segments[(i + 1) % len(segments)][0]
               for i in range(len(segments)))
    return count


def references(raw: bytes) -> list[int]:
    assert len(raw) >= 4
    count = u32(raw, 0)
    assert count <= 65_535 and len(raw) == 4 + 4 * count
    return list(struct.unpack("<" + "I" * count, raw[4:]))


def shape_entries(raw: bytes) -> list[tuple[int, int, int]]:
    count = u32(raw, 0)
    assert 0 < count <= 65_535
    pos = 4
    result = []
    for _ in range(count):
        layer_ref, role, shape, reserved, size = struct.unpack_from("<IBBHI", raw, pos)
        assert role in range(3) and shape in range(5) and reserved == 0
        pos += 12
        data = raw[pos : pos + size]
        assert len(data) == size
        if shape == 0:
            rational(data)
        elif shape in (1, 2):
            assert len(data) == 64
            rational(data[:32]); rational(data[32:])
        elif shape == 3:
            assert len(data) == 103 and data[0] in (0, 1)
            for start in (1, 33, 65):
                rational(data[start : start + 32])
            assert u16(data, 97) < 360 and u32(data, 99) > 0
        else:
            boundary(data)
        pos += size
        result.append((layer_ref, role, shape))
    assert pos == len(raw)
    return result


def parse_envelope(raw: bytes, sql_kind: int) -> dict[int, tuple[int, bytes]]:
    assert len(raw) >= 52 and raw[:4] == b"BSIG"
    assert u16(raw, 4) == 1 and u16(raw, 6) == sql_kind and u32(raw, 8) == 0
    size = u64(raw, 12)
    assert size <= 64 * 1024 * 1024 and size == len(raw) - 52
    assert hashlib.sha256(raw[52:]).digest() == raw[20:52]
    payload = raw[52:]
    assert u16(payload, 0) == 1
    field_count = u16(payload, 2)
    assert 1 <= field_count <= 64
    pos = 4
    found = {}
    for expected_id, expected_type in TYPES[sql_kind]:
        assert pos + 8 <= len(payload)
        field_id, ty, length = struct.unpack_from("<HHI", payload, pos)
        assert (field_id, ty) == (expected_id, expected_type)
        pos += 8
        data = payload[pos : pos + length]
        assert len(data) == length
        pos += length
        found[field_id] = (ty, data)
    assert field_count == len(TYPES[sql_kind]) and pos == len(payload)
    return found


def check_geometry(kind: int, fields: dict, refs: set[int]) -> None:
    def field(n: int) -> bytes:
        return fields[n][1]

    def require_index(idx: int) -> None:
        assert idx in refs, "missing geometry_refs index"

    def require_point(raw: bytes) -> tuple[int, int, int]:
        p = point(raw)
        require_index(p[0])
        return p

    if kind == 1:
        data = field(1)
        assert len(data) == 4 + u32(data, 0) * 20 and u32(data, 0) >= 2
        coords = [require_point(data[4+i*20 : 24+i*20])
                  for i in range(u32(data, 0))]
        assert [p[1:] for p in coords] == [(0, 0), (10000, 0)]
        assert rational(field(2)) == (1, 100000)
    elif kind == 2:
        assert [require_point(field(i))[1:] for i in (1, 2, 3)] == [
            (3000, 3000), (4000, 3000), (3000, 4000)
        ]
        assert field(4) == b"\x00" and rational(field(5)) == (1, 100000)
    elif kind in (3, 4, 9):
        assert boundary(field(1)) == 4
        if kind != 4:
            for idx in references(field(2)):
                require_index(idx)
    elif kind == 5:
        entries = shape_entries(field(1))
        assert [(role, shape) for _, role, shape in entries] == [
            (0, 0), (1, 1), (0, 2), (2, 3), (1, 4)
        ]
        for idx, _, _ in entries:
            require_index(idx)
    elif kind == 6:
        assert require_point(field(1))[1:] == (0, 0)
        orientation = field(2)
        assert len(orientation) == 12
        assert orientation[:4] == bytes((1, 1, 255, 0))
        assert struct.unpack("<d", orientation[4:])[0] == 0.37
    elif kind == 7:
        assert require_point(field(1))[1:] == (8000, 8000)
        assert field(2) == b"\x00"
        assert len(field(3)) == 8
        a, b = struct.unpack("<II", field(3))
        assert a != b
        require_index(a); require_index(b)
        layers = references(field(4))
        assert len(layers) == 2
        for idx in layers:
            require_index(idx)
    elif kind == 8:
        assert rational(field(1)) == (1, 20000)
        assert field(2) == b"\x01"
        a, b = struct.unpack("<II", field(3))
        assert a != b
        require_index(a); require_index(b)


def verify_golden(stem: str, rich: bool) -> None:
    path = BSI_DIR / (stem + ".bsi")
    expected_digest = (BSI_DIR / (stem + ".sha256")).read_text().strip()
    raw = path.read_bytes()
    actual_digest = hashlib.sha256(raw).hexdigest()
    assert actual_digest == expected_digest, (stem, actual_digest, expected_digest)
    assert raw[:16] == b"SQLite format 3\x00"
    assert int.from_bytes(raw[68:72], "big") == 0x42534931
    assert raw[18:20] == b"\x01\x01", "no live WAL snapshots"
    con = sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)
    try:
        assert con.execute("PRAGMA application_id").fetchone()[0] == 0x42534931
        assert con.execute("PRAGMA user_version").fetchone()[0] == 1
        assert con.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
        assert con.execute("PRAGMA foreign_key_check").fetchall() == []
        assert con.execute("SELECT format_major,format_minor,min_reader_major,min_reader_minor FROM file_meta").fetchall() == [(1, 0, 1, 0)]
        assert con.execute("SELECT COUNT(*) FROM features").fetchone()[0] == 0
        assert con.execute("SELECT COUNT(*) FROM extensions").fetchone()[0] == 0
        rows = con.execute(
            "SELECT g.object_id,g.geometry_kind,g.codec_version,g.envelope,o.source_object "
            "FROM geometry g JOIN objects o ON o.id=g.object_id ORDER BY g.object_id"
        ).fetchall()
        if rich:
            kinds = Counter(row[1] for row in rows)
            assert kinds == Counter({1: 1, 2: 1, 3: 1, 4: 2, 5: 1, 6: 1, 7: 1, 8: 1, 9: 1})
            assert {"T1", "ARC1", "AREA1", "PAD1", "VIA1", "DRILL1", "BOARD1"}.issubset(
                {row[4] for row in rows}
            )
        else:
            assert rows == []
        for obj_id, kind, codec, envelope, object_name in rows:
            assert codec == 1 and kind in TYPES and object_name
            ref_rows = con.execute(
                "SELECT ref_index,target_object_id FROM geometry_refs WHERE geometry_id=? ORDER BY ref_index",
                (obj_id,),
            ).fetchall()
            assert [r[0] for r in ref_rows] == list(range(len(ref_rows)))
            for _, target in ref_rows:
                assert con.execute("SELECT 1 FROM objects WHERE id=?", (target,)).fetchone()
            fields = parse_envelope(envelope, kind)
            check_geometry(kind, fields, {r[0] for r in ref_rows})
            damaged = bytearray(envelope)
            damaged[-1] ^= 0x01
            try:
                parse_envelope(damaged, kind)
            except AssertionError:
                pass
            else:
                raise AssertionError("mutated binary wrongly passed the independent oracle")
    finally:
        con.close()
    assert hashlib.sha256(path.read_bytes()).hexdigest() == expected_digest
    print(f"F004_ORACLE PASS {stem} sha256={expected_digest} geometries={len(rows)}")


if __name__ == "__main__":
    verify_golden("golden-v1-empty", rich=False)
    verify_golden("golden-v1-nine-geometry", rich=True)
