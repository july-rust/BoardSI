//! B_005 restricted source-bound Reference geometry coverage audit.
//! An exact centreline-in-polygon proof is NOT a physical return-current/P009 proof.
use crate::copper_preview::{CopperArea as SourceArea, CopperGeometry};
use crate::{copper_preview, route_preview};
use boardsi_model::{
    strict_simple_area_contains, BoardModel, BoundarySegment, ConnectionEvidence, ContactSite,
    LayerKind, ObjectId, ObjectKind,
};
use std::collections::BTreeSet;

const MAX_PATH_SEGMENTS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceGeometryAudit {
    /// SOURCE_ORDER_GEOMETRY_ONLY is deliberately not a physical/solver PASS.
    pub state: &'static str,
    pub reason: &'static str,
    pub trace_ids: Vec<ObjectId>,
    pub area_id: Option<ObjectId>,
    pub signal_layer: Option<ObjectId>,
    pub reference_layer: Option<ObjectId>,
    pub return_path_p009: &'static str,
    pub solver_ready: bool,
    pub physical_si: &'static str,
    pub cache_reuse_eligible: bool,
}
impl ReferenceGeometryAudit {
    fn unsupported(reason: &'static str) -> Self {
        Self {
            state: "UNSUPPORTED",
            reason,
            trace_ids: Vec::new(),
            area_id: None,
            signal_layer: None,
            reference_layer: None,
            return_path_p009: "UNSUPPORTED",
            solver_ready: false,
            physical_si: "NOT_RUN",
            cache_reuse_eligible: false,
        }
    }
}

fn cross(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> i128 {
    (i128::from(b[0]) - i128::from(a[0])) * (i128::from(p[1]) - i128::from(a[1]))
        - (i128::from(b[1]) - i128::from(a[1])) * (i128::from(p[0]) - i128::from(a[0]))
}
fn on(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> bool {
    cross(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn touches(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let (x, y, z, w) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (x == 0 && on(a, b, c))
        || (y == 0 && on(a, b, d))
        || (z == 0 && on(c, d, a))
        || (w == 0 && on(c, d, b))
        || ((x > 0) != (y > 0) && (z > 0) != (w > 0))
}
fn exact_source_ring(
    area: &boardsi_model::CopperArea,
    source: &SourceArea,
    frame: &ObjectId,
    allow_cutout: bool,
) -> bool {
    let CopperGeometry::Polygon(ring) = &source.geometry else {
        return false;
    };
    if source.cutouts.len() > usize::from(allow_cutout)
        || !(4..=65).contains(&ring.len())
        || ring.first() != ring.last()
        || area.outer.segments.len() + 1 != ring.len()
        || area.cutouts.len() != source.cutouts.len()
    {
        return false;
    }
    let exact_ring = |source_ring: &[[i64; 2]], segments: &[BoundarySegment]| {
        source_ring.len() == segments.len() + 1
            && segments
                .iter()
                .zip(source_ring.windows(2))
                .all(|(seg, pair)| {
                    matches!(seg, BoundarySegment::Line { start, end }
                if start.frame == *frame && end.frame == *frame
                    && start.ticks == pair[0] && end.ticks == pair[1])
                })
    };
    exact_ring(ring, &area.outer.segments)
        && source
            .cutouts
            .iter()
            .zip(&area.cutouts)
            .all(|(source_hole, canonical_hole)| {
                let CopperGeometry::Polygon(hole) = source_hole else {
                    return false;
                };
                (4..=65).contains(&hole.len())
                    && hole.first() == hole.last()
                    && exact_ring(hole, &canonical_hole.boundary.segments)
            })
}
fn segment_strictly_inside(
    area: &boardsi_model::CopperArea,
    a: &boardsi_model::Point,
    b: &boardsi_model::Point,
) -> bool {
    if a.frame != b.frame || a.ticks == b.ticks {
        return false;
    }
    // Canonical strict containment excludes points in voids and on EVERY
    // contour boundary. The straight segment must also avoid every hole edge.
    // Endpoint-only checking is insufficient for a path that crosses a cutout.
    if strict_simple_area_contains(area, a).ok() != Some(true)
        || strict_simple_area_contains(area, b).ok() != Some(true)
    {
        return false;
    }
    // Endpoints inside a valid simple polygon + NO intersection with ANY
    // boundary edge is an exact containment certificate for this straight line.
    // This catches concave excursions even when both endpoints are inside.
    let misses_boundary = |s: &BoundarySegment| match s {
        BoundarySegment::Line { start, end } => !touches(a.ticks, b.ticks, start.ticks, end.ticks),
        _ => false,
    };
    area.outer.segments.iter().all(misses_boundary)
        && area
            .cutouts
            .iter()
            .all(|hole| hole.boundary.segments.iter().all(misses_boundary))
}
fn exact_join(
    board: &BoardModel,
    a: &boardsi_model::Trace,
    b: &boardsi_model::Trace,
) -> Option<[i64; 2]> {
    let mut possible = None;
    for (ia, pa) in a.centreline.iter().enumerate() {
        for (ib, pb) in b.centreline.iter().enumerate() {
            if pa.frame != pb.frame || pa.ticks != pb.ticks {
                continue;
            }
            let aid = board
                .connectivity
                .nodes
                .iter()
                .find(|n| {
                    n.conductor == a.meta.id
                        && n.layer == a.layer
                        && n.site == ContactSite::TraceVertex(ia)
                })
                .map(|n| &n.meta.id);
            let bid = board
                .connectivity
                .nodes
                .iter()
                .find(|n| {
                    n.conductor == b.meta.id
                        && n.layer == b.layer
                        && n.site == ContactSite::TraceVertex(ib)
                })
                .map(|n| &n.meta.id);
            let linked = match (aid, bid) {
                (Some(x), Some(y)) => board.connectivity.edges.iter().any(|edge| {
                    matches!(edge.evidence, ConnectionEvidence::SameLayerContact { .. })
                        && ((&edge.a == x && &edge.b == y) || (&edge.a == y && &edge.b == x))
                }),
                _ => false,
            };
            if linked {
                if possible.is_some() {
                    return None; // ambiguous shared endpoints or a loop
                }
                possible = Some(pa.ticks);
            }
        }
    }
    possible
}

/// Caller explicitly selects an ordered, same-layer source Trace path and
/// the Reference Net/Layer. This does not pick ports, trace widths or a physical
/// return path. A source-Net-labelled area is a geometry candidate, never
/// electrical proof. Layer adjacency uses ONLY the imported order, not z/stackup.
pub fn audit(
    board: &BoardModel,
    source_areas: &[SourceArea],
    original_bytes: &[u8],
    ordered_trace_ids: &[ObjectId],
    reference_net_name: &str,
    reference_layer_id: &ObjectId,
) -> ReferenceGeometryAudit {
    audit_impl(
        board,
        source_areas,
        original_bytes,
        ordered_trace_ids,
        reference_net_name,
        reference_layer_id,
        false,
    )
}

/// B_008: restrict the same source-bound audit to one straight polygon COPCUT.
/// This proves only geometric centreline coverage in source-layer order.
/// It never certifies electrical Reference/Return Path continuity or P009.
pub fn audit_cutout_aware(
    board: &BoardModel,
    source_areas: &[SourceArea],
    original_bytes: &[u8],
    ordered_trace_ids: &[ObjectId],
    reference_net_name: &str,
    reference_layer_id: &ObjectId,
) -> ReferenceGeometryAudit {
    audit_impl(
        board,
        source_areas,
        original_bytes,
        ordered_trace_ids,
        reference_net_name,
        reference_layer_id,
        true,
    )
}

fn audit_impl(
    board: &BoardModel,
    source_areas: &[SourceArea],
    original_bytes: &[u8],
    ordered_trace_ids: &[ObjectId],
    reference_net_name: &str,
    reference_layer_id: &ObjectId,
    allow_cutout: bool,
) -> ReferenceGeometryAudit {
    let unsupported = ReferenceGeometryAudit::unsupported;
    if board.validate().has_errors() {
        return unsupported("INVALID_CANONICAL_MODEL");
    }
    if source_areas.len() != board.areas.len()
        || board.sources.len() != 1
        || board.frames.len() != 1
    {
        return unsupported("SOURCE_INVENTORY_MISMATCH");
    }
    if ordered_trace_ids.is_empty() || ordered_trace_ids.len() > MAX_PATH_SEGMENTS {
        return unsupported("PATH_LENGTH_UNSUPPORTED");
    }
    // Freshly parse the caller-supplied original PADS bytes. The file SHA,
    // exact source-area labels/geometries and selected source ROUTE records
    // must match the Canonical imported model; a modified detached inventory
    // cannot silently certify a different Reference Net.
    let Ok(parsed_route) = route_preview::parse(original_bytes) else {
        return unsupported("ORIGINAL_PADS_ROUTE_UNSUPPORTED");
    };
    let Ok(source_board) = route_preview::into_canonical(original_bytes, &parsed_route) else {
        return unsupported("ORIGINAL_PADS_MODEL_UNSUPPORTED");
    };
    let Ok(parsed_areas) = copper_preview::parse(original_bytes, parsed_route.max_layer) else {
        return unsupported("ORIGINAL_PADS_COPPER_UNSUPPORTED");
    };
    if board.sources[0].id != source_board.sources[0].id {
        return unsupported("ORIGINAL_SOURCE_SHA_MISMATCH");
    }
    if parsed_areas != source_areas {
        return unsupported("ORIGINAL_COPPER_INVENTORY_MISMATCH");
    }
    // A source ID/hash match alone is not proof that an editable Canonical
    // inventory still contains the original Net names or complete geometry.
    // In particular a deleted unselected branch must not become a valid path.
    if board.nets.len() != source_board.nets.len()
        || !board.nets.iter().all(|net| {
            source_board
                .nets
                .iter()
                .any(|original| original.meta.id == net.meta.id && original.name == net.name)
        })
    {
        return unsupported("ORIGINAL_NET_IDENTITY_MISMATCH");
    }
    if board.traces.len() != source_board.traces.len()
        || !board.traces.iter().all(|trace| {
            source_board.traces.iter().any(|original| {
                original.meta.id == trace.meta.id
                    && original.layer == trace.layer
                    && original.net == trace.net
                    && original.centreline == trace.centreline
                    && original.width_m == trace.width_m
            })
        })
    {
        return unsupported("ORIGINAL_ROUTE_INVENTORY_MISMATCH");
    }
    if board.arcs.len() != source_board.arcs.len()
        || !board.arcs.iter().all(|arc| {
            source_board.arcs.iter().any(|original| {
                original.meta.id == arc.meta.id
                    && original.layer == arc.layer
                    && original.net == arc.net
                    && original.centre == arc.centre
                    && original.start == arc.start
                    && original.end == arc.end
                    && original.clockwise == arc.clockwise
                    && original.width_m == arc.width_m
            })
        })
    {
        return unsupported("ORIGINAL_ARC_INVENTORY_MISMATCH");
    }
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for id in ordered_trace_ids {
        if !seen.insert(id.clone()) {
            return unsupported("DUPLICATE_TRACE");
        }
        let Some(trace) = board.traces.iter().find(|t| &t.meta.id == id) else {
            return unsupported("UNKNOWN_TRACE");
        };
        if trace.centreline.len() != 2
            || trace.centreline[0].frame != trace.centreline[1].frame
            || trace.centreline[0].ticks == trace.centreline[1].ticks
            || trace.centreline[0].frame != board.frames[0].meta.id
        {
            return unsupported("UNSUPPORTED_TRACE_GEOMETRY");
        }
        // Trace ID alone does not bind its points, layer or Net to PADS.
        let Some(raw_trace) = source_board
            .traces
            .iter()
            .find(|t| t.meta.id == trace.meta.id)
        else {
            return unsupported("ORIGINAL_TRACE_NOT_FOUND");
        };
        if raw_trace.layer != trace.layer
            || raw_trace.net != trace.net
            || raw_trace.centreline != trace.centreline
        {
            return unsupported("ORIGINAL_TRACE_GEOMETRY_MISMATCH");
        }
        selected.push(trace);
    }
    let first = selected[0];
    let Some(signal_net) = &first.net else {
        return unsupported("UNKNOWN_SIGNAL_NET");
    };
    if !board.nets.iter().any(|n| n.meta.id == *signal_net) {
        return unsupported("UNRECOGNIZED_SIGNAL_NET");
    }
    if selected
        .iter()
        .any(|t| t.layer != first.layer || t.net.as_ref() != Some(signal_net))
    {
        return unsupported("MIXED_LAYER_OR_NET_PATH");
    }
    let Some(reference_net) = board.nets.iter().find(|n| n.name == reference_net_name) else {
        return unsupported("UNKNOWN_REFERENCE_NET");
    };
    if reference_net.meta.id == *signal_net {
        return unsupported("SIGNAL_REFERENCE_NET_IDENTICAL");
    }
    // Never take positional adjacency from user-mutated Canonical orders.
    // Imported PADS source layers supply only SOURCE ORDER, not physical z.
    if board.stackup.layers.len() != source_board.stackup.layers.len()
        || !board
            .stackup
            .layers
            .iter()
            .zip(&source_board.stackup.layers)
            .all(|(a, b)| a.meta.id == b.meta.id && a.order == b.order && a.kind == b.kind)
    {
        return unsupported("ORIGINAL_LAYER_ORDER_MISMATCH");
    }
    let sl = board
        .stackup
        .layers
        .iter()
        .find(|l| l.meta.id == first.layer);
    let rl = board
        .stackup
        .layers
        .iter()
        .find(|l| &l.meta.id == reference_layer_id);
    let (Some(sl), Some(rl)) = (sl, rl) else {
        return unsupported("UNKNOWN_LAYER");
    };
    if sl.kind != LayerKind::Copper
        || rl.kind != LayerKind::Copper
        || sl.order.abs_diff(rl.order) != 1
    {
        return unsupported("NOT_ADJACENT_SOURCE_COPPER_ORDER");
    }
    let mut previous_joint = None;
    for pair in selected.windows(2) {
        let Some(joint) = exact_join(board, pair[0], pair[1]) else {
            return unsupported("MISSING_EXACT_GRAPH_JOIN");
        };
        if previous_joint == Some(joint) {
            return unsupported("BACKTRACK_OR_AMBIGUOUS_PATH");
        }
        // Oppositely oriented collinear segments may overlap even when the
        // exact contact graph has an unambiguous endpoint edge.
        let a_other = pair[0]
            .centreline
            .iter()
            .find(|p| p.ticks != joint)
            .unwrap()
            .ticks;
        let b_other = pair[1]
            .centreline
            .iter()
            .find(|p| p.ticks != joint)
            .unwrap()
            .ticks;
        if on(a_other, joint, b_other) || on(joint, b_other, a_other) {
            return unsupported("COLLINEAR_BACKTRACK");
        }
        previous_joint = Some(joint);
    }
    // A declared path cannot silently cross itself or pass an unselected
    // same-Net straight Trace. Check every selected centreline, not just joins,
    // including paths of one Trace and interior T junctions from B_003.
    for (i, trace) in selected.iter().enumerate() {
        let a = trace.centreline[0].ticks;
        let b = trace.centreline[1].ticks;
        for later in selected.iter().skip(i + 2) {
            if touches(a, b, later.centreline[0].ticks, later.centreline[1].ticks) {
                return unsupported("SELF_INTERSECTING_PATH");
            }
        }
        for other in &board.traces {
            if selected.iter().any(|t| t.meta.id == other.meta.id)
                || other.layer != first.layer
                || other.net.as_ref() != Some(signal_net)
            {
                continue;
            }
            if other.centreline.len() != 2 {
                return unsupported("BRANCH_GEOMETRY_UNSUPPORTED");
            }
            if touches(a, b, other.centreline[0].ticks, other.centreline[1].ticks) {
                return unsupported("UNSELECTED_TRACE_TOUCHES_PATH");
            }
        }
    }
    // This bounded single-layer path audit cannot certify a via-connected
    // branch, even when the via and downstream Trace have another layer.
    // Check the reparsed original Via positions so a detached canonical
    // inventory cannot hide a source-through conductor.
    if source_board.vias.iter().any(|via| {
        selected.iter().any(|trace| {
            on(
                trace.centreline[0].ticks,
                trace.centreline[1].ticks,
                via.position.ticks,
            )
        })
    }) {
        return unsupported("SOURCE_VIA_CONTACT_UNSUPPORTED");
    }
    // Exact source Arc endpoints touching the selected centreline are
    // established non-straight graph branches, not covered by B_005.
    for arc in &board.arcs {
        if arc.layer != first.layer || arc.net.as_ref() != Some(signal_net) {
            continue;
        }
        if selected.iter().any(|trace| {
            let a = trace.centreline[0].ticks;
            let b = trace.centreline[1].ticks;
            on(a, b, arc.start.ticks) || on(a, b, arc.end.ticks)
        }) {
            return unsupported("UNSELECTED_ARC_BRANCH");
        }
    }
    let source_id = &board.sources[0].id;
    let frame_id = &board.frames[0].meta.id;
    let mut qualifying = None;
    for (area, src) in board.areas.iter().zip(source_areas) {
        let expected = ObjectId::new(
            source_id,
            ObjectKind::Area,
            &format!("source-copper-{}-{}", src.name, src.source_line),
            "",
        );
        if area.meta.id != expected {
            return unsupported("ALTERED_SOURCE_AREA_ID");
        }
        if &area.layer != reference_layer_id || src.net_label.as_deref() != Some(reference_net_name)
        {
            continue;
        }
        // Source order and Canonical shape must agree, not just name/Net.
        let Some(source_layer) = board.stackup.layers.get(
            usize::try_from(src.layer)
                .ok()
                .and_then(|v| v.checked_sub(1))
                .unwrap_or(usize::MAX),
        ) else {
            return unsupported("INVALID_SOURCE_LAYER");
        };
        if source_layer.meta.id != area.layer
            || !exact_source_ring(area, src, frame_id, allow_cutout)
            || area
                .net
                .as_ref()
                .is_some_and(|net| *net != reference_net.meta.id)
        {
            return unsupported("UNSUPPORTED_OR_MUTATED_REFERENCE_AREA");
        }
        let covers_all = selected
            .iter()
            .all(|t| segment_strictly_inside(area, &t.centreline[0], &t.centreline[1]));
        if covers_all {
            if qualifying.is_some() {
                return unsupported("AMBIGUOUS_OVERLAPPING_REFERENCE_AREAS");
            }
            qualifying = Some(area.meta.id.clone());
        }
    }
    let Some(area_id) = qualifying else {
        return unsupported("NO_SINGLE_AREA_COVERS_ENTIRE_PATH");
    };
    ReferenceGeometryAudit {
        state: "SOURCE_ORDER_GEOMETRY_ONLY",
        reason: "EXACT_CENTRELINE_COVERAGE_NOT_PHYSICAL_RETURN",
        trace_ids: ordered_trace_ids.to_vec(),
        area_id: Some(area_id),
        signal_layer: Some(first.layer.clone()),
        reference_layer: Some(reference_layer_id.clone()),
        return_path_p009: "UNSUPPORTED",
        solver_ready: false,
        physical_si: "NOT_RUN",
        cache_reuse_eligible: false,
    }
}
