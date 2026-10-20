#!/usr/bin/env python3
"""Conservative M0 KiCad Edge.Cuts line-only board outline qualification.

No importer, no CAD connectivity, no physics and no license determination.
Only one closed, simple, unambiguous GR_LINE outline without footprint-local
Edge.Cuts may establish bounding dimensions. Every other shape fails closed.
"""
from __future__ import annotations

from collections import defaultdict
from decimal import Decimal
import re

COORD = r"(-?\d+(?:\.\d+)?)"
POINT = lambda field: re.compile(r"\(" + field + r"\s+" + COORD + r"\s+" + COORD + r"\)")
CAP = 1500


def _orientation(a, b, c):
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])


def _on_segment(a, b, p):
    return (_orientation(a, b, p) == 0
            and min(a[0], b[0]) <= p[0] <= max(a[0], b[0])
            and min(a[1], b[1]) <= p[1] <= max(a[1], b[1]))


def _strict_inside(a, b, p):
    return p != a and p != b and _on_segment(a, b, p)


def _intersection(a, b, c, d):
    # Inclusive: used only for pairs without shared endpoints.
    o1, o2 = _orientation(a, b, c), _orientation(a, b, d)
    o3, o4 = _orientation(c, d, a), _orientation(c, d, b)
    if o1 == 0 and _on_segment(a, b, c):
        return True
    if o2 == 0 and _on_segment(a, b, d):
        return True
    if o3 == 0 and _on_segment(c, d, a):
        return True
    if o4 == 0 and _on_segment(c, d, b):
        return True
    return ((o1 > 0) != (o2 > 0)) and ((o3 > 0) != (o4 > 0))


def qualify_line_outline(outline: list[str], footprint_edges: int) -> dict:
    """Return a result without claiming unsupported shapes as dimensions.

    The geometry is a single simple line-segment polygon in KiCad native mm.
    This is strictly narrower than a general PCB Edge.Cuts outline parser:
    arcs, circles, polygons, rectangles and footprint-local transforms are
    deliberately excluded rather than approximated from endpoints.
    """
    def not_verified(reason):
        return {
            "status": "NOT_VERIFIED",
            "reason": reason,
            "dimensions_mm": None,
            "single_simple_closed_line_loop": False,
        }

    if footprint_edges:
        return not_verified("Footprint-local Edge.Cuts requires coordinate transforms")
    if not 3 <= len(outline) <= CAP:
        return not_verified("Need 3..1500 explicit board-level Edge.Cuts edges")
    segments = []
    adjacency = defaultdict(list)
    for i, p in enumerate(outline):
        if not re.match(r"^\(gr_line(?:\s|\()", p):
            return not_verified("Curves, rectangles and non-line shapes not qualified")
        st, en = POINT("start").findall(p), POINT("end").findall(p)
        if len(st) != 1 or len(en) != 1:
            return not_verified("Unparseable or duplicate line coordinates")
        a = tuple(Decimal(q) for q in st[0])
        b = tuple(Decimal(q) for q in en[0])
        if a == b:
            return not_verified("Zero-length Edge.Cuts segment")
        segments.append((a, b))
        adjacency[a].append((b, i))
        adjacency[b].append((a, i))
    if any(len(neighbors) != 2 for neighbors in adjacency.values()):
        return not_verified("Outline has dangling or branch endpoints")
    if len(adjacency) != len(segments):
        return not_verified("Outline is not one simple vertex/edge cycle")

    # Detect crossing edges, T-junctions and overlapping collinear edges.
    for i, (a, b) in enumerate(segments):
        for c, d in segments[i + 1:]:
            if (_strict_inside(a, b, c) or _strict_inside(a, b, d)
                    or _strict_inside(c, d, a) or _strict_inside(c, d, b)):
                return not_verified("Overlapping edges or endpoint-on-interior")
            if {a, b}.isdisjoint({c, d}) and _intersection(a, b, c, d):
                return not_verified("Self-intersecting Edge.Cuts outline")

    # Follow connected edges; distinct endpoint graph does not imply one loop.
    start = min(adjacency)
    ordered = [start]
    used_edges = set()
    current = start
    for _ in segments:
        choices = [(node, eid) for node, eid in adjacency[current]
                   if eid not in used_edges]
        if len(choices) != 1 and used_edges:
            return not_verified("Ambiguous or disconnected outline path")
        if not choices:
            return not_verified("Contour ends before closure")
        node, eid = choices[0]
        used_edges.add(eid)
        current = node
        if current == start:
            break
        if current in ordered:
            return not_verified("Premature loop intersection")
        ordered.append(current)
    if current != start or len(used_edges) != len(segments):
        return not_verified("More than one closed contour or open contour")
    area_twice = sum(
        ordered[i][0] * ordered[(i + 1) % len(ordered)][1]
        - ordered[(i + 1) % len(ordered)][0] * ordered[i][1]
        for i in range(len(ordered))
    )
    if area_twice == 0:
        return not_verified("Zero-area outline")
    xmin, xmax = min(p[0] for p in ordered), max(p[0] for p in ordered)
    ymin, ymax = min(p[1] for p in ordered), max(p[1] for p in ordered)
    if xmin == xmax or ymin == ymax:
        return not_verified("Zero-width board extents")
    return {
        "status": "VERIFIED_LINE_ONLY",
        "reason": "One closed, non-self-intersecting board-level gr_line polygon; no other Edge.Cuts",
        "dimensions_mm": {"width": float(xmax - xmin), "height": float(ymax - ymin),
                          "xmin": float(xmin), "xmax": float(xmax),
                          "ymin": float(ymin), "ymax": float(ymax)},
        "single_simple_closed_line_loop": True,
        "shape_area_mm2": float(abs(area_twice) / 2),
        "caveat": "Geometry-only bounds. No board manufacturing, copper, net, SI, or license certification.",
    }
