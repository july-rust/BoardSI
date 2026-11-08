//! D_056_03: source-owned COPCLS/COPCIR area candidate contacts.
//!
//! Sound-but-incomplete positive-area witnesses only. An accepted interior
//! SOURCE point proves that a positive-width Stroke/Pad has nonzero copper
//! intersection with a source-filled area. Missing witnesses are UNKNOWN,
//! never separation or proof of an Open. No plated/barrel/SI claim.
use crate::copper_preview::{self, CopperArea, CopperGeometry};
use crate::layered_source_graph::{
    ContactKind, CopperFootprint, CopperNode, DeclaredNetCheck, GeometryContact, LayeredSourceGraph,
};
use crate::trace_contact::{Centreline, Stroke};
use crate::via_shapes::ViaLayerShape;
use std::collections::BTreeSet;

const MAX_AREA_NODE_WORK: usize = 10_000_000;
const MAX_NODES: usize = 50_000;
const MAX_CONTACTS: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Region {
    Inside,
    Outside,
    Boundary,
}

fn cross(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> i128 {
    (i128::from(b[0]) - i128::from(a[0])) * (i128::from(p[1]) - i128::from(a[1]))
        - (i128::from(b[1]) - i128::from(a[1])) * (i128::from(p[0]) - i128::from(a[0]))
}
fn ring_region(vertices: &[[i64; 2]], p: [i64; 2]) -> Region {
    // Native parser validates closure, nonzero fill and source topology.
    let mut inside = false;
    for pair in vertices.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let side = cross(a, b, p);
        if side == 0
            && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
            && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
        {
            return Region::Boundary;
        }
        if (a[1] > p[1]) != (b[1] > p[1]) && ((b[1] > a[1]) == (side > 0)) {
            inside = !inside;
        }
    }
    if inside {
        Region::Inside
    } else {
        Region::Outside
    }
}
fn shape_region(shape: &CopperGeometry, p: [i64; 2]) -> Result<Region, &'static str> {
    match shape {
        CopperGeometry::Polygon(ring) => Ok(ring_region(ring, p)),
        CopperGeometry::Circle { centre, radius } => {
            let dx = i128::from(p[0]) - i128::from(centre[0]);
            let dy = i128::from(p[1]) - i128::from(centre[1]);
            let d = dx * dx + dy * dy;
            let r2 = i128::from(*radius) * i128::from(*radius);
            Ok(match d.cmp(&r2) {
                std::cmp::Ordering::Less => Region::Inside,
                std::cmp::Ordering::Greater => Region::Outside,
                std::cmp::Ordering::Equal => Region::Boundary,
            })
        }
        CopperGeometry::Curved { .. } => Err("D05603_UNSUPPORTED_CURVED_AREA_CONTACT"),
    }
}
fn strictly_in_area(area: &CopperArea, p: [i64; 2]) -> Result<bool, &'static str> {
    if shape_region(&area.geometry, p)? != Region::Inside {
        return Ok(false);
    }
    for hole in &area.cutouts {
        // An exact hole boundary is unknown copper, not conductive.
        if shape_region(hole, p)? != Region::Outside {
            return Ok(false);
        }
    }
    Ok(true)
}
fn midpoint(a: [i64; 2], b: [i64; 2], numerator: i128) -> Option<[i64; 2]> {
    let mut out = [0; 2];
    for i in 0..2 {
        let v = (4 - numerator) * i128::from(a[i]) + numerator * i128::from(b[i]);
        if v % 4 != 0 {
            return None;
        }
        out[i] = i64::try_from(v / 4).ok()?;
    }
    Some(out)
}
fn stroke_samples(s: &Stroke) -> Vec<[i64; 2]> {
    let (start, end) = match &s.centreline {
        Centreline::Straight { start, end } | Centreline::Circular { start, end, .. } => {
            (*start, *end)
        }
    };
    let mut samples = vec![start, end];
    // Only points actually on a STRAIGHT centreline. Circular arcs have
    // endpoints only; unproven interpolation cannot qualify an arc contact.
    if matches!(&s.centreline, Centreline::Straight { .. }) {
        for part in 1..4 {
            if let Some(p) = midpoint(start, end, part) {
                samples.push(p);
            }
        }
    }
    samples
}
fn via_samples(
    centre: [i64; 2],
    outer_ticks: i64,
    drill_ticks: i64,
    shape: ViaLayerShape,
) -> Vec<[i64; 2]> {
    let inner = match shape {
        ViaLayerShape::Round | ViaLayerShape::Square => drill_ticks,
        ViaLayerShape::Annular {
            inner_diameter_ticks,
        } => inner_diameter_ticks,
        ViaLayerShape::RoundAntiPad | ViaLayerShape::SquareAntiPad => return Vec::new(),
    };
    // A radial sample strictly *inside* actual conductive annulus/square
    // and strictly *outside* the bore; never test the drilled-out centre.
    let r = (i128::from(inner) + i128::from(outer_ticks)) / 4;
    if 2 * r <= i128::from(inner) || 2 * r >= i128::from(outer_ticks) {
        return Vec::new();
    }
    let Ok(r) = i64::try_from(r) else {
        return Vec::new();
    };
    [
        (centre[0].checked_add(r), Some(centre[1])),
        (centre[0].checked_sub(r), Some(centre[1])),
        (Some(centre[0]), centre[1].checked_add(r)),
        (Some(centre[0]), centre[1].checked_sub(r)),
    ]
    .into_iter()
    .filter_map(|(x, y)| Some([x?, y?]))
    .collect()
}
fn area_witness_samples(area: &CopperArea) -> Vec<[i64; 2]> {
    match &area.geometry {
        CopperGeometry::Polygon(v) => v[..v.len().saturating_sub(1)].to_vec(),
        CopperGeometry::Circle { centre, radius } => {
            let r = radius / 2;
            vec![
                *centre,
                [centre[0] + r, centre[1]],
                [centre[0] - r, centre[1]],
                [centre[0], centre[1] + r],
                [centre[0], centre[1] - r],
            ]
        }
        CopperGeometry::Curved { .. } => Vec::new(),
    }
}
/// A source polygon outer vertex is a valid positive-overlap witness when
/// strictly inside the *other* filled Area: the source polygon has a
/// positive-area wedge there. Circle samples must be strictly inside their
/// OWN filled copper, not merely inside the other Area (which might lie
/// wholly in a circular cutout).
fn valid_own_area_sample(area: &CopperArea, p: [i64; 2]) -> Result<bool, &'static str> {
    if area
        .cutouts
        .iter()
        .any(|hole| shape_region(hole, p).unwrap_or(Region::Boundary) != Region::Outside)
    {
        return Ok(false);
    }
    match &area.geometry {
        CopperGeometry::Circle { .. } => strictly_in_area(area, p),
        CopperGeometry::Polygon(vertices) => Ok(vertices
            .iter()
            .take(vertices.len().saturating_sub(1))
            .any(|v| *v == p)),
        CopperGeometry::Curved { .. } => Err("D05603_UNSUPPORTED_CURVED_AREA_CONTACT"),
    }
}
fn kind_for(node: &CopperFootprint) -> Option<ContactKind> {
    match node {
        CopperFootprint::Stroke { .. } => Some(ContactKind::TraceCopperArea),
        CopperFootprint::ViaPad { .. } => Some(ContactKind::ViaPadCopperArea),
        CopperFootprint::SmdPad { .. } => Some(ContactKind::SmdPadCopperArea),
        CopperFootprint::CopperArea { .. } => None,
    }
}
/// The importer MUST call this after building the existing trace/via/SMD
/// graph, before any electrical graph qualification. Only source-evidenced
/// shapes are added. The result always remains a candidate-only graph.
pub fn augment_source_areas(
    graph: &mut LayeredSourceGraph,
    bytes: &[u8],
    strokes: &[Stroke],
) -> Result<(), &'static str> {
    if crate::digest::sha256(bytes) != graph.source_sha256 {
        return Err("D05603_AREA_STALE_SOURCE");
    }
    let areas = copper_preview::parse(bytes, graph.declared_copper_layers)?;
    if areas.is_empty() {
        return Ok(());
    }
    if strokes.len() > graph.nodes.len() {
        return Err("D05603_AREA_STROKE_IDENTITY");
    }
    for (i, stroke) in strokes.iter().enumerate() {
        if graph.nodes[i].physical_copper_layer != stroke.layer
            || !matches!(&graph.nodes[i].footprint, CopperFootprint::Stroke { .. })
            || graph.nodes[i].source_line != Some(stroke.source_line)
        {
            return Err("D05603_AREA_STROKE_IDENTITY");
        }
    }
    for area in &areas {
        // Retain validated source geometry, but do not pretend the conservative
        // display polyline of an analytical curve proves actual copper contact.
        if matches!(&area.geometry, CopperGeometry::Curved { .. })
            || area
                .cutouts
                .iter()
                .any(|h| matches!(h, CopperGeometry::Curved { .. }))
        {
            return Err("D05603_UNSUPPORTED_CURVED_AREA_CONTACT");
        }
    }
    let preexisting = graph.nodes.len();
    if preexisting + areas.len() > MAX_NODES {
        return Err("D05603_AREA_NODE_LIMIT");
    }
    if preexisting
        .checked_mul(areas.len())
        .ok_or("D05603_AREA_WORK_LIMIT")?
        > MAX_AREA_NODE_WORK
    {
        return Err("D05603_AREA_WORK_LIMIT");
    }
    let mut new_nodes = Vec::new();
    let mut new_edges = Vec::new();
    let mut seen: BTreeSet<(usize, usize, ContactKind)> = graph
        .geometry_contacts
        .iter()
        .map(|e| (e.first_node, e.second_node, e.kind))
        .collect();
    for (area_index, area) in areas.iter().enumerate() {
        let new_index = preexisting + new_nodes.len();
        let new_node = CopperNode {
            source_key: format!(
                "area:{area_index}:{}:line:{}:L{}",
                area.name, area.source_line, area.layer
            ),
            source_line: Some(area.source_line),
            physical_copper_layer: area.layer,
            declared_net: area.net_label.clone(),
            footprint: CopperFootprint::CopperArea {
                cutout_count: area.cutouts.len(),
            },
        };
        for (index, candidate) in graph.nodes.iter().enumerate() {
            if candidate.physical_copper_layer != area.layer {
                continue;
            }
            let kind = kind_for(&candidate.footprint).ok_or("D05603_AREA_KIND")?;
            let samples = match &candidate.footprint {
                CopperFootprint::Stroke { .. } => stroke_samples(&strokes[index]),
                CopperFootprint::ViaPad {
                    centre,
                    outer_ticks,
                    drill_ticks,
                    shape,
                } => via_samples(*centre, *outer_ticks, *drill_ticks, *shape),
                CopperFootprint::SmdPad { centre, .. } => vec![*centre],
                CopperFootprint::CopperArea { .. } => unreachable!(),
            };
            if samples
                .iter()
                .any(|&p| strictly_in_area(area, p).unwrap_or(false))
            {
                // An inner-point witness *proves* nonzero overlap, but missing
                // such a witness does NOT prove separation.
                let relation =
                    declared_relation(candidate.declared_net.as_ref(), area.net_label.as_ref());
                let (a, b) = if index < new_index {
                    (index, new_index)
                } else {
                    (new_index, index)
                };
                if seen.insert((a, b, kind)) {
                    new_edges.push(GeometryContact {
                        first_node: a,
                        second_node: b,
                        layer: area.layer,
                        kind,
                        declared_net_check: relation,
                    });
                }
            }
        }
        // Same-layer independent copper islands are linked only with an
        // explicit same-SOURCE filled-area positive-overlap witness.
        for (old_index, other) in areas.iter().take(area_index).enumerate() {
            if area.layer != other.layer {
                continue;
            }
            let identical = area.geometry == other.geometry && area.cutouts == other.cutouts;
            let mutual = area_witness_samples(area).iter().any(|&p| {
                valid_own_area_sample(area, p).unwrap_or(false)
                    && strictly_in_area(other, p).unwrap_or(false)
            }) || area_witness_samples(other).iter().any(|&p| {
                valid_own_area_sample(other, p).unwrap_or(false)
                    && strictly_in_area(area, p).unwrap_or(false)
            });
            if identical || mutual {
                let (a, b) = (preexisting + old_index, new_index);
                if seen.insert((a, b, ContactKind::CopperAreaCopperArea)) {
                    new_edges.push(GeometryContact {
                        first_node: a,
                        second_node: b,
                        layer: area.layer,
                        kind: ContactKind::CopperAreaCopperArea,
                        declared_net_check: declared_relation(
                            other.net_label.as_ref(),
                            area.net_label.as_ref(),
                        ),
                    });
                }
            }
        }
        new_nodes.push(new_node);
    }
    if graph.geometry_contacts.len() + new_edges.len() > MAX_CONTACTS {
        return Err("D05603_AREA_EDGE_LIMIT");
    }
    graph.nodes.extend(new_nodes);
    graph.geometry_contacts.extend(new_edges);
    graph
        .geometry_contacts
        .sort_by_key(|e| (e.first_node, e.second_node, e.kind));
    Ok(())
}
fn declared_relation(a: Option<&String>, b: Option<&String>) -> DeclaredNetCheck {
    match (a, b) {
        (Some(x), Some(y)) if x == y => DeclaredNetCheck::Same,
        (Some(_), Some(_)) => DeclaredNetCheck::Different,
        _ => DeclaredNetCheck::Unknown,
    }
}
