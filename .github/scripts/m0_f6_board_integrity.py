#!/usr/bin/env python3
"""M0 F6 candidate-source integrity probe (no BoardSI CAD import or physics)."""
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import sys
import urllib.request

COMMIT = "181d683185b20890085b0e8f1e0e89fbda63a56b"
ROOT = "https://raw.githubusercontent.com/KiCad/kicad-source-mirror/" + COMMIT + "/"
OUTPUT = pathlib.Path("build/m0-f6-public-boards")
BOARDS = (
    {
        "name": "RoyalBlue54L-Feather",
        "path": "demos/royalblue54L_feather/RoyalBlue54L-Feather.kicad_pcb",
        "license": "demos/royalblue54L_feather/LICENSE",
        "bytes": 3549654,
        "sha256": "535dbe212f43a8bab12b23277e176a94ac9694177258af2981c0e7b08e4b7252",
        "git_blob_sha1": "5d47510c87e3d4c09bfd18d91dec003a718a6ca6",
        "license_marker": "CERN",
    },
    {
        "name": "TinyTapeout-demo",
        "path": "demos/tiny_tapeout/tinytapeout-demo.kicad_pcb",
        "license": "demos/tiny_tapeout/LICENSE.txt",
        "bytes": 4541774,
        "sha256": "0d26bf9b6d561e32cf610d428541d6433e60b45e1838f34f73467e25b9144814",
        "git_blob_sha1": "a8465878d46656c36617870f511e62c979298854",
        "license_marker": "Apache",
    },
)

def download(path: str, max_bytes: int) -> bytes:
    request = urllib.request.Request(
        ROOT + path, headers={"User-Agent": "BoardSI-M0-F6-evidence/1.0"}
    )
    with urllib.request.urlopen(request, timeout=70) as response:
        content = response.read(max_bytes + 1)
    if len(content) > max_bytes:
        raise ValueError(f"{path}: response exceeds expected size budget")
    return content

def iter_sexpressions(raw: str, prefix: str):
    """Yield balanced KiCad (gr_...) primitives; escaped quotes are respected."""
    search_from = 0
    while (start := raw.find(prefix, search_from)) >= 0:
        depth = 0
        in_string = False
        escaped = False
        end = None
        for pos in range(start, len(raw)):
            c = raw[pos]
            if in_string:
                if escaped:
                    escaped = False
                elif c == "\\":
                    escaped = True
                elif c == '"':
                    in_string = False
            elif c == '"':
                in_string = True
            elif c == '(':
                depth += 1
            elif c == ')':
                depth -= 1
                if depth == 0:
                    end = pos + 1
                    break
        if end is None:
            raise ValueError(f"unbalanced KiCad primitive at position {start}")
        yield raw[start:end]
        search_from = end

def geometry_inventory(board: bytes) -> dict:
    content = board.decode("utf-8")
    primitives = list(iter_sexpressions(content, "(gr_"))
    outline = [p for p in primitives if re.search(r'\(layer\s+"?Edge\.Cuts"?\)', p)]
    point_re = re.compile(
        r'\((?:start|end|xy|center)\s+(-?\d+(?:\.\d+)?)\s+(-?\d+(?:\.\d+)?)\)'
    )
    points = [(float(x), float(y)) for p in outline for x, y in point_re.findall(p)]
    endpoint_extent = None
    if points:
        xmin, xmax = min(p[0] for p in points), max(p[0] for p in points)
        ymin, ymax = min(p[1] for p in points), max(p[1] for p in points)
        endpoint_extent = {
            "x_min": xmin, "x_max": xmax, "y_min": ymin, "y_max": ymax,
            "delta_x": round(xmax - xmin, 6),
            "delta_y": round(ymax - ymin, 6),
        }
    return {
        "edge_cuts_graphic_primitives": len(outline),
        "edge_endpoints_extent_mm_DIAGNOSTIC_ONLY": endpoint_extent,
        "edge_extent_certified": False,
        "reason": (
            "Endpoint extents are not a closed-board outline; arc extrema, "
            "cutouts, footprints and board-shape validation need KiCad geometry tools."
        ),
        "count_footprint_tokens": len(re.findall(r'\(footprint\s', content)),
        "count_pad_tokens": len(re.findall(r'\(pad\s', content)),
        "count_segment_tokens": len(re.findall(r'\(segment\s', content)),
        "count_via_tokens": len(re.findall(r'\(via\s', content)),
        "count_net_assignment_occurrences_NOT_distinct_nets": len(re.findall(r'\(net\s+\d+\s+"', content)),
        "counts_are_lexical_not_canonical": True,
    }

def verify(entry: dict) -> dict:
    payload = download(entry["path"], entry["bytes"])
    size = len(payload)
    sha256 = hashlib.sha256(payload).hexdigest()
    git_sha1 = hashlib.sha1(
        b"blob " + str(size).encode("ascii") + b"\0" + payload
    ).hexdigest()
    if (size, sha256, git_sha1) != (
        entry["bytes"], entry["sha256"], entry["git_blob_sha1"]
    ):
        raise ValueError(
            f"{entry['name']}: exact upstream bytes/hash mismatch: "
            f"bytes={size}, sha256={sha256}, git_sha1={git_sha1}"
        )
    license_data = download(entry["license"], 500_000)
    if entry["license_marker"].lower() not in license_data.decode(
        "utf-8", errors="replace"
    ).lower():
        raise ValueError(f"{entry['name']}: missing expected license marker")
    return {
        "name": entry["name"],
        "upstream_commit": COMMIT,
        "board_path": entry["path"],
        "license_path": entry["license"],
        "raw_bytes": size,
        "raw_sha256": sha256,
        "git_blob_sha1": git_sha1,
        "license_raw_sha256": hashlib.sha256(license_data).hexdigest(),
        "license_marker_found": True,
        "integrity_pass": True,
        "redistribution_approved": False,
        "physical_board_size_certified": False,
        "m0_8_gib_pc_test_executed": False,
        "boardsi_import_test_executed": False,
        "scope": "Candidate integrity only: not F6 PASS",
        "inventory": geometry_inventory(payload),
    }

def main() -> int:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    results = []
    failures = []
    for entry in BOARDS:
        try:
            result = verify(entry)
            results.append(result)
            print(
                f"VERIFIED candidate source {entry['name']} "
                f"SHA256={result['raw_sha256']}, bytes={result['raw_bytes']}"
            )
        except Exception as exc:
            failures.append({"name": entry["name"], "error": str(exc)})
            print(f"FAIL candidate source {entry['name']}: {exc}", file=sys.stderr)
    report = {
        "phase": "BoardSI M0",
        "gate": "F6",
        "state": "BLOCKED (candidate-byte proof does not fulfill all F6 gates)",
        "upstream": "KiCad/kicad-source-mirror",
        "commit": COMMIT,
        "platform": sys.platform,
        "results": results,
        "failures": failures,
        "remaining": [
            "Independently classify closed Edge.Cuts physical outline and size",
            "Confirm redistribution rights and third-party attribution",
            "Qualify genuine ~8 GiB CPU-only reference PC and real M0 measurements",
        ],
    }
    destination = OUTPUT / "candidate-integrity.json"
    destination.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print("F6: original bytes independently checked; F6 overall BLOCKED.")
    return 1 if failures else 0

if __name__ == "__main__":
    sys.exit(main())
