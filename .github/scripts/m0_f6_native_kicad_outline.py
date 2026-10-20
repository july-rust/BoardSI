#!/usr/bin/python3
"""M0 public KiCad reference-board native outline oracle.

Uses official pcbnew SWIG bindings, NOT the BoardSI importer/solver.
FAIL CLOSED: a valid SHA alone never certifies geometry or licensing.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

import pcbnew
from m0_f6_board_integrity import BOARDS, COMMIT, OUTPUT, download


def geometry(path: Path) -> dict:
    board = pcbnew.LoadBoard(str(path))
    if board is None:
        raise ValueError("KiCad LoadBoard returned no board")
    poly = pcbnew.SHAPE_POLY_SET()
    accepted = bool(board.GetBoardPolygonOutlines(poly))
    if not accepted:
        raise ValueError("KiCad failed to construct valid board outline")
    if poly.OutlineCount() != 1:
        raise ValueError(f"Expected one outer contour, got {poly.OutlineCount()}")
    if poly.IsSelfIntersecting():
        raise ValueError("KiCad outline polygon self-intersects")
    outline_holes = int(poly.HoleCount(0))
    box = poly.BBox()
    board_box = board.GetBoardEdgesBoundingBox()
    dimensions = {
        "x_min_mm": pcbnew.ToMM(box.GetLeft()),
        "y_min_mm": pcbnew.ToMM(box.GetTop()),
        "width_mm": pcbnew.ToMM(box.GetWidth()),
        "height_mm": pcbnew.ToMM(box.GetHeight()),
    }
    if not (0 < dimensions["width_mm"] < 10000 and
            0 < dimensions["height_mm"] < 10000):
        raise ValueError("Invalid KiCad native board dimensions")
    comparison = {
        "width_mm": pcbnew.ToMM(board_box.GetWidth()),
        "height_mm": pcbnew.ToMM(board_box.GetHeight()),
        "delta_width_mm": round(
            abs(pcbnew.ToMM(board_box.GetWidth()) - dimensions["width_mm"]), 6),
        "delta_height_mm": round(
            abs(pcbnew.ToMM(board_box.GetHeight()) - dimensions["height_mm"]), 6),
    }
    return {
        "status": "KICAD_NATIVE_OUTLINE_PASSED",
        "tool": "pcbnew.SHAPE_POLY_SET from KiCad",
        "num_outer_contours": poly.OutlineCount(),
        "num_cutouts": outline_holes,
        "outline_closed": accepted,
        "self_intersection": False,
        "dimensions": dimensions,
        "alternate_kicad_edges_bbox": comparison,
        "note": "Native geometry qualification ONLY; not independent physical manufacturing, legal distribution, BoardSI import, or SI proof.",
    }


def main() -> int:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    version = str(pcbnew.GetBuildVersion())
    if not version.startswith("9."):
        raise ValueError("Exact KiCad major 9 required; got " + version)
    package = subprocess.check_output(
        ["dpkg-query", "-W", "-f=" + "$" + "{Version}", "kicad"], text=True).strip()
    report = {
        "gate": "M0 F6 native board outline",
        "overall_f6": "BLOCKED",
        "ki_cad_version": version,
        "ki_cad_apt_version": package,
        "upstream_commit": COMMIT,
        "boards": [],
        "not_proven": ["independent non-KiCad geometry oracle",
                       "board fixture redistribution legal clearance",
                       "genuine ~8 GiB physical reference PC",
                       "BoardSI CAD import", "physical SI accuracy"],
    }
    failures = []
    with tempfile.TemporaryDirectory(prefix="boardsi-m0-f6-kicad-") as temp:
        for candidate in BOARDS:
            name = candidate["name"]
            item = {"name": name, "state": "NOT_VERIFIED"}
            try:
                raw = download(candidate["path"], candidate["bytes"])
                if len(raw) != candidate["bytes"]:
                    raise ValueError("Source size mismatch")
                if hashlib.sha256(raw).hexdigest() != candidate["sha256"]:
                    raise ValueError("Pinned source SHA-256 mismatch")
                if hashlib.sha1(
                    b"blob " + str(len(raw)).encode() + b"\0" + raw
                ).hexdigest() != candidate["git_blob_sha1"]:
                    raise ValueError("Pinned source Git blob SHA-1 mismatch")
                fp = Path(temp) / (name + ".kicad_pcb")
                fp.write_bytes(raw)
                item["geometry"] = geometry(fp)
                item["state"] = "KICAD_NATIVE_OUTLINE_PASSED"
                item["source_sha256"] = candidate["sha256"]
            except Exception as exc:
                item["error"] = f"{type(exc).__name__}: {exc}"
                failures.append(name)
            report["boards"].append(item)
    report["native_outline_success_count"] = 2 - len(failures)
    path = OUTPUT / "native-kicad-outline.json"
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps(report, indent=2))
    if failures:
        print("F6 native outline NOT_VERIFIED for: " + ", ".join(failures))
        return 1
    print("F6 native KiCad outline proof PASS, overall F6 still BLOCKED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
