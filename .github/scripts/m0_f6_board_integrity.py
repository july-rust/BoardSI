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

def validate_balanced_kicad_sexpressions(content: str) -> None:
    """Fail closed on malformed global S-expression nesting or quoted strings.

    Validate the entire document, not only the tokens that a metadata regex
    happened to recognize. This is a lexical integrity guard, not a CAD parse.
    """
    depth = 0
    quoted = False
    escaped = False
    roots = 0
    for char in content:
        if quoted:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                quoted = False
        elif char == '"':
            quoted = True
        elif char == '(':
            if depth == 0:
                roots += 1
            depth += 1
        elif char == ')':
            depth -= 1
            if depth < 0:
                raise ValueError("Malformed KiCad S-expression: unmatched closing paren")
    if quoted or escaped:
        raise ValueError("Malformed KiCad S-expression: unterminated quoted string")
    if depth != 0 or roots != 1:
        raise ValueError("Malformed KiCad S-expression: unbalanced or multiple roots")


def geometry_inventory(board: bytes) -> dict:
    """Lexical CAD-source inventory, not a topology/closed-board measurement."""
    content = board.decode("utf-8")
    validate_balanced_kicad_sexpressions(content)
    primitives = list(iter_sexpressions(content, "(gr_"))
    edge_layer = re.compile(r'\(layer\s+"?Edge\.Cuts"?\)')
    outline = [p for p in primitives if edge_layer.search(p)]
    point_re = re.compile(
        r'\((?:start|mid|end|xy|center)\s+(-?\d+(?:\.\d+)?)\s+(-?\d+(?:\.\d+)?)\)'
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

    # In KiCad the (layers ...) table enumerates named copper layers.
    # This is an observed table count, NOT validated copper connectivity.
    layers = next(iter_sexpressions(content, "(layers"), "")
    copper_entries = re.findall(
        r'\(\s*(\d{1,3})\s+"((?:F|B|In\d+)\.Cu)"\s+\w+',
        layers
    )
    copper_layers = sorted(
        {name for _, name in copper_entries},
        key=lambda name: (name != "F.Cu", name == "B.Cu", name),
    )
    if len({number for number, _ in copper_entries}) != len(copper_entries):
        raise ValueError("Duplicate copper-layer table indices")

    # KiCad may repeat (net id "name") within pad assignments; count
    # unique seen IDs, not occurrences or independently verified nets.
    net_declarations = re.findall(
        r'\(net\s+(\d+)\s+"((?:\\.|[^"\\])*)"\s*\)', content
    )
    unique_net_names = {}
    for raw_id, name in net_declarations:
        ident = int(raw_id)
        if ident in unique_net_names and unique_net_names[ident] != name:
            raise ValueError("Conflicting observed KiCad net names for one ID")
        unique_net_names[ident] = name
    observed_named_nets = sum(
        1 for ident, name in unique_net_names.items() if ident != 0 and name
    )
    types = {}
    for primitive in outline:
        match = re.match(r'\((gr_[A-Za-z_]+)\b', primitive)
        if match:
            key = match.group(1)
            types[key] = types.get(key, 0) + 1
    # Edge.Cuts might also be inside footprint-local graphic coordinates.
    # They require rotation/translation before any board bounding estimate.
    footprint_edge_graphics = sum(
        1 for p in iter_sexpressions(content, "(fp_")
        if edge_layer.search(p)
    )
    return {
        "edge_cuts_graphic_primitives": len(outline),
        "edge_cuts_types_lexical": dict(sorted(types.items())),
        "footprint_local_edge_cuts_primitives_lexical": footprint_edge_graphics,
        "edge_endpoints_extent_mm_DIAGNOSTIC_ONLY": endpoint_extent,
        "edge_extent_certified": False,
        "reason": (
            "Endpoint/midpoint extents are NOT a closed-board outline; "
            "arc/circle extrema, footprint transforms, holes/cutouts, "
            "manufacturing geometry and connectivity remain unverified."
        ),
        "copper_layer_table_names_lexical": copper_layers,
        "copper_layer_table_count_lexical": len(copper_layers),
        "observed_unique_named_net_ids_lexical": observed_named_nets,
        "count_footprint_tokens": len(re.findall(r'\(footprint\s', content)),
        "count_pad_tokens": len(re.findall(r'\(pad\s', content)),
        "count_segment_tokens": len(re.findall(r'\(segment\s', content)),
        "count_via_tokens": len(re.findall(r'\(via\s', content)),
        "count_net_assignment_occurrences_NOT_distinct_nets": len(net_declarations),
        "counts_are_lexical_not_canonical": True,
        "board_dimensions_mm_verified": False,
        "board_size_classification_verified": False,
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
    license_text = " ".join(license_data.decode(
        "utf-8", errors="replace"
    ).split())
    if entry["license_marker"].lower() not in license_text.lower():
        raise ValueError(f"{entry['name']}: missing expected license marker")
    expected_license_phrase = {
        "RoyalBlue54L-Feather": "CERN Open Hardware Licence Version 2 - Permissive",
        "TinyTapeout-demo": "Apache License Version 2.0, January 2004",
    }[entry["name"]]
    if expected_license_phrase.lower() not in license_text.lower():
        raise ValueError(f"{entry['name']}: unexpected source license family")
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
        "license_family_heading": expected_license_phrase,
        "license_family_heading_found": True,
        "redistribution_legal_clearance": "NOT_VERIFIED",
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
