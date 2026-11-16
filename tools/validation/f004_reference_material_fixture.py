#!/usr/bin/env python3
"""Independent F004 material/stackup Golden, all values explicitly synthetic."""
from __future__ import annotations

import argparse
import hashlib
import sqlite3
import struct
from pathlib import Path

from f004_reference_fixture import generate, i128

EVIDENCE = ("F004 synthetic, NOT a measurement", "v1", "Assumed Golden fixture")


def insert_material(conn: sqlite3.Connection, object_id: int, name: str, props: dict) -> None:
    conn.execute("INSERT INTO objects VALUES(?,?,?,?,?,?)",
                 (object_id, 1, "Material", name, "", "F004 synthetic property"))
    conn.execute("INSERT INTO materials VALUES(?,?)", (object_id, name))
    for key in ("Dk", "Df", "Conductivity", "ValidBand"):
        value = props.get(key)
        if value is None:
            conn.execute("INSERT INTO material_properties VALUES(?,?,?,?,?,?,?,?,?)",
                         (object_id, key, "Missing", None, None, None, None, None, None))
        else:
            blob = struct.pack("<dd", *value) if key == "ValidBand" else struct.pack("<d", value)
            conn.execute("INSERT INTO material_properties VALUES(?,?,?,?,?,?,?,?,?)",
                         (object_id, key, "Assumed", blob, *EVIDENCE, None, None))


def update_layer_property(conn: sqlite3.Connection, layer: int, key: str) -> None:
    conn.execute("INSERT INTO layer_property_sources VALUES(?,?,?,?,?,?,?)",
                 (layer, key, *EVIDENCE, None, None))


def generate_material(path: Path) -> None:
    generate(path)
    try:
        with sqlite3.connect(path) as conn:
            conn.execute(
                "UPDATE file_meta SET file_uuid=?, producer=?, producer_version=?",
                ("00000000-0000-4000-8000-000000000014", "F004 independent material Oracle", "1"),
            )
            insert_material(conn, 5, "SYNTHETIC_CU", {"Conductivity": 5.8e7})
            insert_material(conn, 6, "SYNTHETIC_FR4", {
                "Dk": 4.2, "Df": 0.015, "ValidBand": (1e6, 30e9)
            })
            # Existing L1: 35 um copper; exact rational 7/200000 m.
            conn.execute(
                "UPDATE layers SET material_id=?, z_status=?,z_num_le=?,z_den_le=?,"
                "thickness_status=?,thickness_num_le=?,thickness_den_le=? WHERE object_id=2",
                (5, "Assumed", i128(0), i128(1),
                 "Assumed", i128(7), i128(200000)),
            )
            update_layer_property(conn, 2, "z_m")
            update_layer_property(conn, 2, "thickness_m")
            conn.execute("INSERT INTO objects VALUES(?,?,?,?,?,?)",
                         (7, 1, "Layer", "CORE", "", "F004 synthetic layer"))
            conn.execute("INSERT INTO layers VALUES(?,?,?,?,?,?,?,?,?,?,?)",
                         (7, 1, "CORE", "DielectricCore", 6,
                          "Assumed", i128(7), i128(200000),
                          "Assumed", i128(1), i128(1000)))
            update_layer_property(conn, 7, "z_m")
            update_layer_property(conn, 7, "thickness_m")
            conn.commit()
            assert conn.execute("PRAGMA integrity_check").fetchone()[0] == "ok"
            assert conn.execute("PRAGMA foreign_key_check").fetchall() == []
        content = path.read_bytes()
        print(f"F004_MATERIAL_SHA256={hashlib.sha256(content).hexdigest()} bytes={len(content)}")
    except Exception:
        path.unlink(missing_ok=True)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    generate_material(args.output)
