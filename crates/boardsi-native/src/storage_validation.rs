//! Exact, solver-independent structural admission for native persistent data.
//! The canonical solver validator can short-circuit on PendingRotation or
//! f64 PrecisionLoss before checking actual geometry. Never interpret those
//! errors as permission to skip source-grid structural invariants.
use boardsi_model::*;
use std::cmp::Ordering;
use std::collections::BTreeSet;

fn point_exists(board: &BoardModel, point: &Point) -> bool {
    board.frames.iter().any(|f| f.meta.id == point.frame)
}
fn same_point(board: &BoardModel, a: &Point, b: &Point) -> bool {
    if !point_exists(board, a) || !point_exists(board, b) {
        return false;
    }
    if a.frame == b.frame {
        return a.ticks == b.ticks;
    }
    match (board.exact_point_m(a), board.exact_point_m(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false, // Mixed PendingRotation frames are not exactly comparable yet.
    }
}
fn arc(board: &BoardModel, centre: &Point, start: &Point, end: &Point) -> bool {
    if !point_exists(board, centre)
        || !point_exists(board, start)
        || !point_exists(board, end)
        || centre.frame != start.frame
        || start.frame != end.frame
        || start == end
    {
        return false;
    }
    let squared = |p: &Point| -> Option<i128> {
        let dx = i128::from(p.ticks[0]) - i128::from(centre.ticks[0]);
        let dy = i128::from(p.ticks[1]) - i128::from(centre.ticks[1]);
        dx.checked_mul(dx)
            .and_then(|x| dy.checked_mul(dy).and_then(|y| x.checked_add(y)))
    };
    matches!((squared(start), squared(end)), (Some(a), Some(b)) if a > 0 && a == b)
}
fn boundary(board: &BoardModel, b: &Boundary) -> bool {
    if b.segments.len() < 2 || b.segments.len() > 1_000_000 {
        return false;
    }
    let mut endpoints = Vec::with_capacity(b.segments.len());
    for s in &b.segments {
        let (start, end) = match s {
            BoundarySegment::Line { start, end } => (start, end),
            BoundarySegment::Arc {
                centre, start, end, ..
            } => {
                if !arc(board, centre, start, end) {
                    return false;
                }
                (start, end)
            }
        };
        if same_point(board, start, end) || !point_exists(board, start) || !point_exists(board, end)
        {
            return false;
        }
        endpoints.push((start, end));
    }
    endpoints
        .iter()
        .enumerate()
        .all(|(i, pair)| same_point(board, pair.1, endpoints[(i + 1) % endpoints.len()].0))
}
fn shape(board: &BoardModel, s: &PadShape) -> bool {
    match s {
        PadShape::Circle { diameter_m } => diameter_m.positive(),
        PadShape::Rectangle { size_m } => size_m.iter().all(|r| r.positive()),
        PadShape::Annulus {
            outer_diameter_m,
            inner_diameter_m,
        } => {
            inner_diameter_m.positive()
                && outer_diameter_m.positive()
                && outer_diameter_m.checked_cmp(*inner_diameter_m).ok() == Some(Ordering::Greater)
        }
        PadShape::Thermal {
            centre_size_m,
            outer_size_m,
            spoke_width_m,
            first_spoke_degrees,
            spoke_count,
            ..
        } => {
            centre_size_m.positive()
                && outer_size_m.checked_cmp(*centre_size_m).ok() == Some(Ordering::Greater)
                && spoke_width_m.positive()
                && centre_size_m
                    .checked_cmp(*spoke_width_m)
                    .ok()
                    .is_some_and(|o| o != Ordering::Less)
                && *first_spoke_degrees < 360
                && (1..=64).contains(spoke_count)
        }
        PadShape::Custom(b) => boundary(board, b),
    }
}
fn copper_order(board: &BoardModel, id: &ObjectId) -> Option<u32> {
    board
        .stackup
        .layers
        .iter()
        .find(|layer| layer.meta.id == *id && layer.kind == LayerKind::Copper)
        .map(|layer| layer.order)
}
fn span(board: &BoardModel, s: &ViaSpan) -> Option<(u32, u32)> {
    let (first, last) = (
        copper_order(board, &s.first_layer)?,
        copper_order(board, &s.last_layer)?,
    );
    (first < last).then_some((first, last))
}
fn net(board: &BoardModel, id: &Option<ObjectId>) -> bool {
    id.as_ref()
        .map_or(true, |net| board.nets.iter().any(|x| x.meta.id == *net))
}
fn orientation_valid(orientation: &Orientation) -> bool {
    match orientation {
        Orientation::Orthogonal { .. } => true,
        Orientation::PendingRotation { radians, .. } => radians.is_finite(),
    }
}
pub(crate) fn valid(board: &BoardModel) -> bool {
    for trace in &board.traces {
        if trace.centreline.len() < 2
            || !trace.width_m.positive()
            || copper_order(board, &trace.layer).is_none()
            || !net(board, &trace.net)
            || trace.centreline.iter().any(|p| !point_exists(board, p))
            || trace.centreline.windows(2).any(|pair| {
                (pair[0].frame != pair[1].frame
                    && (board.exact_point_m(&pair[0]).is_err()
                        || board.exact_point_m(&pair[1]).is_err()))
                    || same_point(board, &pair[0], &pair[1])
            })
        {
            return false;
        }
    }
    for x in &board.arcs {
        if !x.width_m.positive()
            || copper_order(board, &x.layer).is_none()
            || !net(board, &x.net)
            || !arc(board, &x.centre, &x.start, &x.end)
        {
            return false;
        }
    }
    for x in &board.areas {
        if copper_order(board, &x.layer).is_none()
            || !net(board, &x.net)
            || !boundary(board, &x.outer)
            || x.cutouts.iter().any(|c| !boundary(board, &c.boundary))
        {
            return false;
        }
    }
    for x in &board.outlines {
        if !boundary(board, &x.outer) || x.cutouts.iter().any(|c| !boundary(board, &c.boundary)) {
            return false;
        }
    }
    for stack in &board.padstacks {
        let mut entries = BTreeSet::new();
        if stack.entries.is_empty() {
            return false;
        }
        for entry in &stack.entries {
            if copper_order(board, &entry.layer).is_none()
                || !shape(board, &entry.shape)
                || !entries.insert((entry.layer.clone(), entry.role as u8))
            {
                return false;
            }
        }
    }
    for x in &board.pads {
        if !point_exists(board, &x.position)
            || !orientation_valid(&x.orientation)
            || !net(board, &x.net)
            || !board.padstacks.iter().any(|s| s.meta.id == x.padstack)
        {
            return false;
        }
    }
    for x in &board.components {
        if !point_exists(board, &x.placement.position)
            || !orientation_valid(&x.placement.orientation)
        {
            return false;
        }
    }
    for x in &board.drills {
        if !x.diameter_m.positive() || span(board, &x.span).is_none() {
            return false;
        }
    }
    let all_copper: Vec<_> = board
        .stackup
        .layers
        .iter()
        .filter(|x| x.kind == LayerKind::Copper)
        .map(|x| x.order)
        .collect();
    for x in &board.vias {
        let Some((first, last)) = span(board, &x.span) else {
            return false;
        };
        let Some(stack) = board.padstacks.iter().find(|s| s.meta.id == x.padstack) else {
            return false;
        };
        let Some(drill) = board.drills.iter().find(|d| d.meta.id == x.drill) else {
            return false;
        };
        if !point_exists(board, &x.position)
            || !net(board, &x.net)
            || drill.span != x.span
            || all_copper.is_empty()
        {
            return false;
        }
        let (minimum, maximum) = (
            *all_copper.iter().min().expect("nonempty copper"),
            *all_copper.iter().max().expect("nonempty copper"),
        );
        let kind_ok = match x.kind {
            ViaKind::Through => first == minimum && last == maximum,
            ViaKind::Blind => (first == minimum) != (last == maximum),
            ViaKind::Buried => first > minimum && last < maximum,
            ViaKind::Microvia => !all_copper
                .iter()
                .any(|order| *order > first && *order < last),
        };
        if !kind_ok
            || stack.entries.iter().any(|entry| {
                !copper_order(board, &entry.layer)
                    .is_some_and(|order| order >= first && order <= last)
            })
        {
            return false;
        }
        let mut seen = BTreeSet::new();
        for layer in &x.connected_layers {
            if !drill.plated
                || !seen.insert(layer)
                || !copper_order(board, layer).is_some_and(|order| order >= first && order <= last)
                || !stack
                    .entries
                    .iter()
                    .any(|e| &e.layer == layer && e.role == PadRole::Conductive)
            {
                return false;
            }
        }
    }
    true
}
