//! C_002: bounded PADS source-backed reference split and via transition risk.
//! Geometry/topology candidates only: never a current return, P009 or solver pass.
use crate::{copper_preview, route_preview};
use boardsi_model::{
    strict_simple_area_contains, BoardModel, BoundarySegment, LayerKind, ObjectId, PadRole, Point,
    ViaKind,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopologyRiskAudit {
    pub state: &'static str,
    pub reason: &'static str,
    pub trace_ids: Vec<ObjectId>,
    pub reference_area_ids: Vec<ObjectId>,
    pub via_id: Option<ObjectId>,
    /// Source-grid coordinates are witnesses only, not a resolved EM field.
    pub risk_source_ticks: Vec<[i64; 2]>,
    pub return_path_p009: &'static str,
    pub solver_ready: bool,
    pub physical_si: &'static str,
    pub cache_reuse_eligible: bool,
}
impl TopologyRiskAudit {
    fn new(state: &'static str, reason: &'static str) -> Self {
        Self {
            state,
            reason,
            trace_ids: Vec::new(),
            reference_area_ids: Vec::new(),
            via_id: None,
            risk_source_ticks: Vec::new(),
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
fn touch(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
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
fn boundary_clear(segs: &[BoundarySegment], a: [i64; 2], b: [i64; 2]) -> bool {
    segs.iter().all(|s| match s {
        BoundarySegment::Line { start, end } => !touch(a, b, start.ticks, end.ticks),
        _ => false,
    })
}
fn covered(
    area: &boardsi_model::CopperArea,
    a: &Point,
    b: &Point,
) -> Result<bool, &'static str> {
    if a.frame != b.frame || a.ticks == b.ticks {
        return Err("INVALID_REFERENCE_TRACE_FRAME");
    }
    let is_inside_a = strict_simple_area_contains(area, a)
        .map_err(|_| "REFERENCE_GEOMETRY_PREDICATE_UNSUPPORTED")?;
    let is_inside_b = strict_simple_area_contains(area, b)
        .map_err(|_| "REFERENCE_GEOMETRY_PREDICATE_UNSUPPORTED")?;
    Ok(is_inside_a
        && is_inside_b
        && boundary_clear(&area.outer.segments, a.ticks, b.ticks)
        && area
            .cutouts
            .iter()
            .all(|hole| boundary_clear(&hole.boundary.segments, a.ticks, b.ticks)))
}
fn bounds(area: &boardsi_model::CopperArea) -> Option<[i64; 4]> {
    let mut pts = area.outer.segments.iter().map(|s| match s {
        BoundarySegment::Line { start, .. } => Some(start.ticks),
        _ => None,
    });
    let first = pts.next()??;
    let mut out = [first[0], first[1], first[0], first[1]];
    for p in pts {
        let p = p?;
        out[0] = out[0].min(p[0]);
        out[1] = out[1].min(p[1]);
        out[2] = out[2].max(p[0]);
        out[3] = out[3].max(p[1]);
    }
    Some(out)
}

/// Compare every relevant imported source object, including unselected branches.
/// Previously-added local contact edges are allowed, never used as physical proof.
fn source_exact(
    board: &BoardModel,
    bytes: &[u8],
) -> Result<Vec<copper_preview::CopperArea>, &'static str> {
    if board.validate().has_errors() || board.sources.len() != 1 || board.frames.len() != 1 {
        return Err("INVALID_CANONICAL");
    }
    let parsed = route_preview::parse(bytes).map_err(|_| "SOURCE_ROUTE_UNSUPPORTED")?;
    let mut expected =
        route_preview::into_canonical(bytes, &parsed).map_err(|_| "SOURCE_MODEL_UNSUPPORTED")?;
    let areas =
        copper_preview::parse(bytes, parsed.max_layer).map_err(|_| "SOURCE_COPPER_UNSUPPORTED")?;
    copper_preview::augment(&mut expected, &areas).map_err(|_| "SOURCE_COPPER_UNSUPPORTED")?;
    if board.sources[0].id != expected.sources[0].id {
        return Err("SOURCE_SHA_MISMATCH");
    }
    if board.traces != expected.traces
        || board.arcs != expected.arcs
        || board.vias != expected.vias
        || board.drills != expected.drills
        || board.padstacks != expected.padstacks
        || board.pads != expected.pads
        || board.areas.len() != expected.areas.len()
        || board.nets.len() != expected.nets.len()
        || !board.nets.iter().all(|net| {
            expected
                .nets
                .iter()
                .any(|raw| raw.meta.id == net.meta.id && raw.name == net.name)
        })
        || board.stackup.layers.len() != expected.stackup.layers.len()
        || !board
            .stackup
            .layers
            .iter()
            .zip(&expected.stackup.layers)
            .all(|(a, b)| a.meta.id == b.meta.id && a.kind == b.kind && a.order == b.order)
    {
        return Err("SOURCE_INVENTORY_MISMATCH");
    }
    for (actual, original) in board.areas.iter().zip(&expected.areas) {
        if actual.meta.id != original.meta.id
            || actual.layer != original.layer
            || actual.outer != original.outer
            || actual.cutouts != original.cutouts
            || actual.net.as_ref().is_some_and(|n| {
                !board.nets.iter().any(|net| {
                    net.meta.id == *n
                        && areas.iter().zip(&expected.areas).any(|(src, imported)| {
                            imported.meta.id == actual.meta.id
                                && src.net_label.as_deref() == Some(net.name.as_str())
                        })
                })
            })
        {
            return Err("SOURCE_AREA_MISMATCH");
        }
    }
    Ok(areas)
}

/// Exactly one straight trace, or exactly two straight source traces on different
/// layers meeting at an explicitly plated Through Via centre. Caller supplies
/// reference-layer candidate(s) in signal traversal order. No reference is inferred.
pub fn audit(
    board: &BoardModel,
    original_bytes: &[u8],
    trace_ids: &[ObjectId],
    reference_net_name: &str,
    reference_layers: &[ObjectId],
) -> TopologyRiskAudit {
    let reject = |reason| TopologyRiskAudit::new("UNSUPPORTED", reason);
    if !(1..=2).contains(&trace_ids.len()) || reference_layers.len() != trace_ids.len() {
        return reject("BOUNDED_PATH_OR_REFERENCE_COUNT");
    }
    let source_areas = match source_exact(board, original_bytes) {
        Ok(areas) => areas,
        Err(reason) => return reject(reason),
    };
    let mut unique = BTreeSet::new();
    let mut traces = Vec::new();
    for id in trace_ids {
        if !unique.insert(id) {
            return reject("DUPLICATE_TRACE");
        }
        let Some(t) = board.traces.iter().find(|t| &t.meta.id == id) else {
            return reject("UNKNOWN_TRACE");
        };
        if t.centreline.len() != 2 || t.centreline[0].ticks == t.centreline[1].ticks {
            return reject("UNSUPPORTED_TRACE");
        }
        traces.push(t);
    }
    let Some(net) = traces[0].net.as_ref() else {
        return reject("UNKNOWN_SIGNAL_NET");
    };
    if traces.iter().any(|t| t.net.as_ref() != Some(net)) {
        return reject("MIXED_SIGNAL_NET");
    }
    let Some(reference_net) = board.nets.iter().find(|n| n.name == reference_net_name) else {
        return reject("UNKNOWN_REFERENCE_NET");
    };
    if reference_net.meta.id == *net {
        return reject("SIGNAL_EQUALS_REFERENCE");
    }
    for (t, ref_id) in traces.iter().zip(reference_layers) {
        let sl = board.stackup.layers.iter().find(|l| l.meta.id == t.layer);
        let rl = board.stackup.layers.iter().find(|l| &l.meta.id == ref_id);
        let (Some(sl), Some(rl)) = (sl, rl) else {
            return reject("UNKNOWN_LAYER");
        };
        if sl.kind != LayerKind::Copper
            || rl.kind != LayerKind::Copper
            || sl.order.abs_diff(rl.order) != 1
        {
            return reject("REFERENCE_NOT_SOURCE_ORDER_ADJACENT");
        }
    }
    let mut refs = Vec::new();
    for (src, actual) in source_areas.iter().zip(&board.areas) {
        if src.net_label.as_deref() != Some(reference_net_name) {
            continue;
        }
        if reference_layers.iter().any(|id| *id == actual.layer) {
            if !matches!(src.geometry, copper_preview::CopperGeometry::Polygon(_))
                || src.cutouts.len() > 8
                || src
                    .cutouts
                    .iter()
                    .any(|h| !matches!(h, copper_preview::CopperGeometry::Polygon(_)))
            {
                return reject("REFERENCE_GEOMETRY_UNSUPPORTED");
            }
            refs.push(actual);
        }
    }
    if refs.is_empty()
        || reference_layers
            .iter()
            .any(|id| !refs.iter().any(|a| &a.layer == id))
    {
        return reject("MISSING_SOURCE_REFERENCE_AREA");
    }
    for (i, a) in refs.iter().enumerate() {
        let Some(x) = bounds(a) else {
            return reject("REFERENCE_BOUNDARY_UNSUPPORTED");
        };
        for b in refs.iter().skip(i + 1).filter(|b| b.layer == a.layer) {
            let Some(y) = bounds(b) else {
                return reject("REFERENCE_BOUNDARY_UNSUPPORTED");
            };
            if x[0] <= y[2] && y[0] <= x[2] && x[1] <= y[3] && y[1] <= x[3] {
                return reject("AMBIGUOUS_REFERENCE_AREA_BOUNDS");
            }
        }
    }
    if traces.len() == 1 {
        let t = traces[0];
        let candidates: Vec<_> = refs
            .iter()
            .filter(|a| a.layer == reference_layers[0])
            .collect();
        let mut fully_covered = false;
        for area in &candidates {
            match covered(area, &t.centreline[0], &t.centreline[1]) {
                Ok(true) => fully_covered = true,
                Ok(false) => {}
                Err(reason) => return reject(reason),
            }
        }
        let mut result = if fully_covered {
            TopologyRiskAudit::new(
                "SOURCE_GEOMETRY_ONLY",
                "SOURCE_CENTRELINE_COVERED_NOT_PHYSICAL",
            )
        } else {
            TopologyRiskAudit::new("REFERENCE_SPLIT_RISK", "SOURCE_REFERENCE_GAP_OR_CUTOUT")
        };
        result.trace_ids = trace_ids.to_vec();
        result.reference_area_ids = candidates.iter().map(|a| a.meta.id.clone()).collect();
        if result.state == "REFERENCE_SPLIT_RISK" {
            result.risk_source_ticks.push([
                (t.centreline[0].ticks[0] + t.centreline[1].ticks[0]) / 2,
                (t.centreline[0].ticks[1] + t.centreline[1].ticks[1]) / 2,
            ]);
        }
        return result;
    }
    let a = traces[0];
    let b = traces[1];
    if a.layer == b.layer {
        return reject("TWO_TRACE_SINGLE_LAYER_UNSUPPORTED");
    }
    let joints: Vec<_> = a
        .centreline
        .iter()
        .flat_map(|p| {
            b.centreline
                .iter()
                .filter(move |q| p.frame == q.frame && p.ticks == q.ticks)
                .map(move |_| p.ticks)
        })
        .collect();
    if joints.len() != 1 {
        return reject("TRANSITION_JOINT_UNSUPPORTED");
    }
    let joint = joints[0];
    let candidates: Vec<_> = board
        .vias
        .iter()
        .filter(|via| {
            if via.position.ticks != joint
                || via.net.as_ref() != Some(net)
                || via.kind != ViaKind::Through
                || !via.connected_layers.contains(&a.layer)
                || !via.connected_layers.contains(&b.layer)
            {
                return false;
            }
            let drill = board.drills.iter().find(|d| d.meta.id == via.drill);
            let padstack = board.padstacks.iter().find(|s| s.meta.id == via.padstack);
            drill.is_some_and(|d| d.plated && d.span == via.span)
                && padstack.is_some_and(|s| {
                    [a.layer.clone(), b.layer.clone()].iter().all(|layer| {
                        s.entries
                            .iter()
                            .any(|e| e.layer == *layer && e.role == PadRole::Conductive)
                    })
                })
        })
        .collect();
    if candidates.len() != 1 {
        return reject("NO_UNAMBIGUOUS_PLATED_SOURCE_VIA");
    }
    let mut result = TopologyRiskAudit::new(
        "LAYER_TRANSITION_RISK",
        "SOURCE_PLATED_VIA_REFERENCE_TRANSITION_NOT_PHYSICAL",
    );
    result.trace_ids = trace_ids.to_vec();
    result.reference_area_ids = refs.iter().map(|a| a.meta.id.clone()).collect();
    result.via_id = Some(candidates[0].meta.id.clone());
    result.risk_source_ticks.push(joint);
    result
}
