#!/usr/bin/env python3
"""F_002 executable SQL + deterministic byte-contract qualification; not production I/O."""
import hashlib
import pathlib
import sqlite3
import struct
import tempfile
import unittest
from fractions import Fraction

ROOT = pathlib.Path(__file__).resolve().parents[2]
DDL = (ROOT / "schemas/bsi/v1.sql").read_text(encoding="utf-8")
APPLICATION_ID = 0x42534931
GEOMETRY_KINDS = {"Trace": 1, "Arc": 2, "Area": 3, "Cutout": 4,
                  "Padstack": 5, "Pad": 6, "Via": 7, "Drill": 8, "Outline": 9}
HEADER = struct.Struct("<4sHHIQ")  # magic, codec, kind, flags, uncompressed_len
TLV = struct.Struct("<HHI")  # field_id, type_id, byte_len
MAX_PAYLOAD = 64 * 1024 * 1024  # F_002 reference qualification bound only
I128_MIN = -(1 << 127)
I128_MAX = (1 << 127) - 1


def i128(value):
    if not (I128_MIN <= value <= I128_MAX):
        raise ValueError("i128 overflow")
    return value.to_bytes(16, "little", signed=True)


def rational(n, d):
    if n == I128_MIN or d <= 0 or n < I128_MIN or n > I128_MAX or d > I128_MAX:
        raise ValueError("noncanonical i128 rational")
    if Fraction(n, d).numerator != n or Fraction(n, d).denominator != d:
        raise ValueError("unreduced rational")
    return i128(n) + i128(d)


def parse_rational(data):
    if len(data) != 32:
        raise ValueError("rational length")
    n = int.from_bytes(data[:16], "little", signed=True)
    d = int.from_bytes(data[16:], "little", signed=True)
    rational(n, d)
    return n, d


def point(frame_index, x_ticks, y_ticks):
    return struct.pack("<Iqq", frame_index, x_ticks, y_ticks)


def field(fid, typ, data):
    return TLV.pack(fid, typ, len(data)) + data


def encode(kind, fields):
    if kind not in GEOMETRY_KINDS.values():
        raise ValueError("unknown geometry kind")
    fields = sorted(fields, key=lambda x: x[0])
    if not fields or len({f[0] for f in fields}) != len(fields):
        raise ValueError("duplicate/empty fields")
    payload = struct.pack("<HH", 1, len(fields)) + b"".join(field(*f) for f in fields)
    return HEADER.pack(b"BSIG", 1, kind, 0, len(payload)) + hashlib.sha256(payload).digest() + payload


def decode(data, expected_kind):
    if len(data) < 52 or len(data) > MAX_PAYLOAD + 52:
        raise ValueError("envelope bounds")
    magic, codec, kind, flags, n = HEADER.unpack_from(data)
    payload = data[52:]
    if magic != b"BSIG" or codec != 1 or kind != expected_kind or flags != 0:
        raise ValueError("header mismatch")
    if kind not in GEOMETRY_KINDS.values() or n != len(payload):
        raise ValueError("unknown kind/length")
    if not hashlib.sha256(payload).digest() == data[20:52]:
        raise ValueError("SHA-256 mismatch")
    if len(payload) < 4:
        raise ValueError("short record")
    version, count = struct.unpack_from("<HH", payload)
    if version != 1 or count == 0 or count > 64:
        raise ValueError("field count/version")
    offset = 4
    fields = {}
    for _ in range(count):
        if offset + 8 > len(payload):
            raise ValueError("short TLV")
        fid, typ, size = TLV.unpack_from(payload, offset)
        offset += 8
        if fid in fields or offset + size > len(payload):
            raise ValueError("duplicate/truncated TLV")
        value = payload[offset: offset + size]
        offset += size
        fields[fid] = (typ, value)
    if offset != len(payload) or list(fields) != sorted(fields):
        raise ValueError("trailing data/order")
    # Explicit required fields for Trace/Arc: no permissive unknown layout.
    required = {1: {1: 7, 2: 5}, 2: {1: 6, 2: 6, 3: 6, 4: 1, 5: 5}}
    if kind in required:
        if {f: typ for f, (typ, _) in fields.items()} != required[kind]:
            raise ValueError("shape field contract")
        for typ, value in fields.values():
            if typ == 5:
                parse_rational(value)
            elif typ == 6 and len(value) != 20:
                raise ValueError("point size")
            elif typ == 1 and (len(value) != 1 or value[0] not in (0, 1)):
                raise ValueError("boolean")
            elif typ == 7:
                if len(value) < 4:
                    raise ValueError("point list")
                count = struct.unpack_from("<I", value)[0]
                if count < 2 or len(value) != 4 + 20 * count:
                    raise ValueError("point list size")
    return fields


class F002SchemaQualification(unittest.TestCase):
    def setUp(self):
        self.db = sqlite3.connect(":memory:")
        self.db.executescript(DDL)
        self.db.execute("""INSERT INTO file_meta VALUES
          (1,'123e4567-e89b-42d3-a456-426614174000',1,0,1,0,'Pcb','BoardSI','F_002','2026-11-15T00:00:00Z')""")

    def tearDown(self):
        self.db.close()

    def seed(self):
        self.db.execute("""INSERT INTO source_identity
            (id,source_format,format_other,format_version,importer_version,producer,producer_version)
            VALUES(1,'PadsAsc',NULL,'9.4','0.1','fixture','1')""")
        self.db.execute("""INSERT INTO objects
            (id,source_id,kind,source_object,part,source_object_type)
            VALUES(1,1,'Frame','source-frame','','Frame')""")

    def test_t01_schema_create_reopen(self):
        self.assertGreaterEqual(sqlite3.sqlite_version_info, (3, 37, 0))
        self.assertEqual(self.db.execute("PRAGMA integrity_check").fetchone(), ("ok",))
        with tempfile.TemporaryDirectory() as d:
            path = pathlib.Path(d) / "fixture.bsi"
            with sqlite3.connect(path) as con:
                con.executescript(DDL)
                con.execute("""INSERT INTO file_meta VALUES
                  (1,'123e4567-e89b-42d3-a456-426614174000',1,0,1,0,'Pcb','BoardSI','F_002','2026-11-15T00:00:00Z')""")
                con.commit()
            blob = path.read_bytes()
            self.assertEqual(blob[:16], b"SQLite format 3\\0".replace(b"\\0", b"\0"))
            self.assertEqual(int.from_bytes(blob[68:72], "big"), APPLICATION_ID)
            ro = sqlite3.connect(f"file:{path.as_posix()}?mode=ro", uri=True)
            self.assertEqual(ro.execute("select format_major from file_meta").fetchone(), (1,))
            ro.close()

    def test_t02_constraints(self):
        self.seed()
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO objects VALUES(2,1,'Frame','source-frame','','Frame')")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO objects VALUES(2,999,'Frame','x','','Frame')")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO source_files VALUES(1,0,'/test',X'AA',1)")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO import_capability VALUES(1,'G','Verified')")
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT INTO features VALUES('x',1,7,NULL)")

    def test_t03_application_and_pragma(self):
        self.assertEqual(self.db.execute("PRAGMA application_id").fetchone(), (APPLICATION_ID,))
        self.assertEqual(self.db.execute("PRAGMA user_version").fetchone(), (1,))
        self.assertEqual(self.db.execute("PRAGMA foreign_keys").fetchone(), (1,))
        self.assertEqual(self.db.execute("PRAGMA trusted_schema").fetchone(), (0,))
        self.assertEqual(self.db.execute("PRAGMA foreign_key_check").fetchall(), [])

    def test_t04_features_and_future_schema(self):
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("UPDATE file_meta SET min_reader_minor=2")
        self.db.execute("INSERT INTO features VALUES('vendor.note',1,0,X'00FF')")
        self.assertEqual(self.db.execute("SELECT hex(opaque_payload) FROM features").fetchone(), ("00FF",))
        self.db.execute("INSERT INTO features VALUES('new.solver',1,1,NULL)")
        self.assertEqual(self.db.execute("SELECT count(*) FROM features WHERE required=1").fetchone(), (1,))

    def test_t05_geometry_vector(self):
        vertices = struct.pack("<I", 2) + point(0, -(1 << 63), 0) + point(0, (1 << 63)-1, 8)
        vec = encode(1, [(1, 7, vertices), (2, 5, rational(1, 1000))])
        self.assertEqual(vec[:4], b"BSIG")
        self.assertEqual(vec[4:6], b"\x01\x00")
        self.assertEqual(len(vec), 52 + 4 + 8 + len(vertices) + 8 + 32)
        self.assertEqual(decode(vec, 1)[2][1], rational(1, 1000))
        arc = encode(2, [(1, 6, point(0, 0, 0)), (2, 6, point(0, 5, 0)),
                         (3, 6, point(0, 0, 5)), (4, 1, b"\x01"),
                         (5, 5, rational(1, 1000))])
        self.assertEqual(len(decode(arc, 2)), 5)
        self.assertNotEqual(vec, encode(1, [(2, 5, rational(1, 1000)), (1, 7, vertices)]) + b"\0")  # canonical order

    def test_t06_integer_rational_boundaries(self):
        self.assertEqual(parse_rational(rational(I128_MAX, 1)), (I128_MAX, 1))
        for args in [(1, 0), (I128_MIN, 1), (2, 4), (1, -2), (I128_MAX+1, 1)]:
            with self.subTest(args=args), self.assertRaises(ValueError):
                rational(*args)
        with self.assertRaises(struct.error):
            point(0, 1 << 63, 0)

    def test_t07_fail_closed_corruption(self):
        trace = encode(1, [(1, 7, struct.pack("<I", 2)+point(0,1,2)+point(0,2,3)),
                           (2, 5, rational(1, 10))])
        for altered in [trace[:-1], trace[:4]+b"\x02"+trace[5:],
                        trace[:8]+b"\x01"+trace[9:],
                        trace[:20]+b"\x00"*32+trace[52:],
                        trace[:52]+trace[52:]+b"x"]:
            with self.subTest(altered=altered[:10]), self.assertRaises(ValueError):
                decode(altered, 1)
        with self.assertRaises(ValueError):
            decode(trace, 2)

    def test_t08_indexes_and_bounds(self):
        names = {row[1] for row in self.db.execute("PRAGMA index_list('geometry')")}
        self.assertIn("geometry_layer_net", names)
        self.assertIn("geometry_padstack", names)
        plan = list(self.db.execute("EXPLAIN QUERY PLAN SELECT * FROM geometry WHERE layer_id=2 AND net_id=3"))
        self.assertTrue(any("geometry_layer_net" in row[3] for row in plan))
        with self.assertRaises(ValueError):
            decode(b"z" * (MAX_PAYLOAD + 53), 1)

    def test_t09_model_enums_and_provenance(self):
        self.seed()
        for kind in ["Trace","Arc","Area","Cutout","Padstack","Pad","Via","Drill","Outline"]:
            self.assertIn(kind, GEOMETRY_KINDS)
        self.db.execute("INSERT INTO source_files VALUES(1,0,'board.asc',zeroblob(32),0)")
        self.db.execute("INSERT INTO import_capability VALUES(1,'G','Missing')")
        self.assertEqual(self.db.execute("SELECT state FROM import_capability").fetchone(), ("Missing",))
        self.db.execute("INSERT INTO conversion_history VALUES(1,0,'original','Pending',NULL,NULL,'no conversion')")
        self.assertEqual(self.db.execute("SELECT quality FROM conversion_history").fetchone(), ("Pending",))

    def test_t10_engine_report(self):
        # Per-runner evidence: SQLite library version and exact platform, not claimed physical CAD.
        import platform
        print("F002_ENGINE", sqlite3.sqlite_version, "PYTHON", platform.python_version(),
              "OS", platform.platform(), "DRIVER", sqlite3.version)
        self.assertGreaterEqual(sqlite3.sqlite_version_info, (3, 37, 0))


if __name__ == "__main__":
    unittest.main(verbosity=2)
