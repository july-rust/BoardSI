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
        // A thermal's outsize is mostly a PLANE VOID, never a filled via disk.
        // No broad radial sample can qualify its spoke-to-plane boundary.
        ViaLayerShape::Thermal {
            kind,
            first_spoke_degrees,
            outer_size_ticks,
            spoke_width_ticks,
            spoke_count,
        } => {
            let thermal = crate::thermal_source::ThermalPad {
                layer_selector: 1,
                kind,
                size_ticks: outer_ticks,
                first_spoke_degrees,
                outer_size_ticks,
                spoke_width_ticks,
                spoke_count,
            };
            if thermal.validate().is_err() {
                return Vec::new();
            }
            let mut witnesses = Vec::new();
            // Central copper annulus: strictly between declared hole and pad.
            let core = (i128::from(drill_ticks) + i128::from(outer_ticks)) / 4;
            if let Ok(r) = i64::try_from(core) {
                for (x, y) in [(r, 0), (-r, 0), (0, r), (0, -r)] {
                    if let (Some(px), Some(py)) =
                        (centre[0].checked_add(x), centre[1].checked_add(y))
                    {
                        if thermal.zone(x as f64, y as f64, drill_ticks)
                            == crate::thermal_source::ThermalZone::Copper
                        {
                            witnesses.push([px, py]);
                        }
                    }
                }
            }
            // One SOURCE-oriented strictly interior spoke witness per spoke.
            // An absent witness is UNKNOWN, never evidence of no contact.
            let between = (outer_ticks + outer_size_ticks) as f64 / 4.0;
            for angle in thermal.angles_degrees() {
                let radians = angle.to_radians();
                let x = (between * radians.cos()).round() as i64;
                let y = (between * radians.sin()).round() as i64;
                if thermal.zone(x as f64, y as f64, drill_ticks)
                    != crate::thermal_source::ThermalZone::Copper
                {
                    continue;
                }
                if let (Some(px), Some(py)) = (centre[0].checked_add(x), centre[1].checked_add(y)) {
                    witnesses.push([px, py]);
                }
            }
            return witnesses;
        }
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
/// A polygon's exact SOURCE outer boundary belongs to a positive-area
/// filled polygon on its inward side. If an open part of that boundary lies
/// strictly INSIDE another SOURCE-filled area (and outside all owned holes),
/// the two open copper interiors overlap with nonzero area. Merely crossing
/// AABBs or sharing a tangent/edge remains UNKNOWN, never an electrical edge.
///
/// The independently validated COPCUT ownership invariant ensures that the
/// perimeter cannot be swallowed by its *own* hole. Circle-owned perimeters
/// are not approximated with chords; those unresolved cases stay UNKNOWN.
fn polygon_perimeter_overlap(
    origin: &CopperArea,
    target: &CopperArea,
) -> Result<bool, &'static str> {
    let CopperGeometry::Polygon(vertices) = &origin.geometry else {
        return Ok(false);
    };
    for edge in vertices.windows(2) {
        if edge[0] == edge[1] {
            return Err("D05603_DEGENERATE_AREA_PERIMETER");
        }
        let boundary = Stroke {
            layer: origin.layer,
            net: origin.net_label.clone().unwrap_or_else(|| "UNKNOWN".into()),
            width_ticks: 1,
            source_line: origin.source_line,
            centreline: Centreline::Straight {
                start: edge[0],
                end: edge[1],
            },
        };
        if crate::straight_area_witness::positive_straight_area_witness(&boundary, target)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn twice(value: i64) -> Result<i64, &'static str> {
    value.checked_mul(2).ok_or("D05603_PAD_SCALE_OVERFLOW")
}

fn doubled_shape(shape: &CopperGeometry) -> Result<CopperGeometry, &'static str> {
    match shape {
        CopperGeometry::Polygon(points) => {
            let mut vertices = Vec::with_capacity(points.len());
            for p in points {
                vertices.push([twice(p[0])?, twice(p[1])?]);
            }
            Ok(CopperGeometry::Polygon(vertices))
        }
        CopperGeometry::Circle { centre, radius } => Ok(CopperGeometry::Circle {
            centre: [twice(centre[0])?, twice(centre[1])?],
            radius: twice(*radius)?,
        }),
        CopperGeometry::Curved { .. } => Err("D05603_UNSUPPORTED_CURVED_AREA_CONTACT"),
    }
}

/// A proof of SOURCE-filled polygon/circle touching a rectangular SMD land
/// even when the pad centre is outside the copper. Source coordinates are
/// doubled to preserve exact half-tick pad boundaries (odd widths/heights).
/// All witnesses are strict interior, never unqualified edge tangency or
/// a SOURCE-owned COPCUT void. A missing witness means UNKNOWN.
pub fn positive_rectangular_smd_area_witness(
    area: &CopperArea,
    centre: [i64; 2],
    width_ticks: i64,
    height_ticks: i64,
) -> Result<bool, &'static str> {
    if width_ticks <= 0 || height_ticks <= 0 {
        return Err("D05603_INVALID_SMD_GEOMETRY");
    }
    let cx = twice(centre[0])?;
    let cy = twice(centre[1])?;
    let (x0, x1) = (
        cx.checked_sub(width_ticks)
            .ok_or("D05603_PAD_SCALE_OVERFLOW")?,
        cx.checked_add(width_ticks)
            .ok_or("D05603_PAD_SCALE_OVERFLOW")?,
    );
    let (y0, y1) = (
        cy.checked_sub(height_ticks)
            .ok_or("D05603_PAD_SCALE_OVERFLOW")?,
        cy.checked_add(height_ticks)
            .ok_or("D05603_PAD_SCALE_OVERFLOW")?,
    );
    let rect = CopperArea {
        name: "D05603_SMD_WITNESS".into(),
        net_label: None,
        layer: area.layer,
        source_line: area.source_line,
        geometry: CopperGeometry::Polygon(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]),
        cutouts: Vec::new(),
    };
    let mut holes = Vec::with_capacity(area.cutouts.len());
    for cutout in &area.cutouts {
        holes.push(doubled_shape(cutout)?);
    }
    let source = CopperArea {
        name: area.name.clone(),
        net_label: area.net_label.clone(),
        layer: area.layer,
        source_line: area.source_line,
        geometry: doubled_shape(&area.geometry)?,
        cutouts: holes,
    };
    if area_witness_samples(&source).into_iter().any(|p| {
        valid_own_area_sample(&source, p).unwrap_or(false)
            && strictly_in_area(&rect, p).unwrap_or(false)
    }) {
        return Ok(true);
    }
    if area_witness_samples(&rect)
        .into_iter()
        .any(|p| strictly_in_area(&source, p).unwrap_or(false))
    {
        return Ok(true);
    }
    if polygon_perimeter_overlap(&rect, &source)? {
        Ok(true)
    } else {
        polygon_perimeter_overlap(&source, &rect)
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
    // Bound event interval work *before* geometry construction. Source
    // areas can each contain up to hundreds of explicit vertices/holes.
    let mut complexity = 0usize;
    for area in &areas {
        let count = |shape: &CopperGeometry| match shape {
            CopperGeometry::Polygon(ring) => ring.len(),
            CopperGeometry::Circle { .. } => 2,
            CopperGeometry::Curved { .. } => 0,
        };
        let mut vertices = count(&area.geometry);
        for hole in &area.cutouts {
            vertices = vertices
                .checked_add(count(hole))
                .ok_or("D05603_AREA_EVENT_WORK_LIMIT")?;
        }
        complexity = complexity
            .checked_add(vertices)
            .ok_or("D05603_AREA_EVENT_WORK_LIMIT")?;
    }
    let rectangular_pad_count = graph
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                &n.footprint,
                CopperFootprint::SmdPad {
                    rectangular: true,
                    ..
                }
            )
        })
        .count();
    let event_users = strokes
        .len()
        .checked_add(rectangular_pad_count)
        .ok_or("D05603_AREA_EVENT_WORK_LIMIT")?;
    if complexity
        .checked_mul(event_users)
        .ok_or("D05603_AREA_EVENT_WORK_LIMIT")?
        > 8_000_000
    {
        return Err("D05603_AREA_EVENT_WORK_LIMIT");
    }
    // Boundary-event combinations can be quadratic in vertex counts.
    // Preflight their worst-case work before allocating candidate edges.
    // The geometry contact inventory remains opt-in and strictly bounded.
    let mut perimeter_work = 0usize;
    for (i, area) in areas.iter().enumerate() {
        for other in areas.iter().take(i) {
            if area.layer != other.layer {
                continue;
            }
            let edge_count = |a: &CopperArea| match &a.geometry {
                CopperGeometry::Polygon(v) => v.len().saturating_sub(1),
                _ => 0,
            };
            let shape_count = |a: &CopperArea| {
                let n = match &a.geometry {
                    CopperGeometry::Polygon(v) => v.len(),
                    CopperGeometry::Circle { .. } => 2,
                    CopperGeometry::Curved { .. } => 0,
                };
                n + a
                    .cutouts
                    .iter()
                    .map(|h| match h {
                        CopperGeometry::Polygon(v) => v.len(),
                        CopperGeometry::Circle { .. } => 2,
                        CopperGeometry::Curved { .. } => 0,
                    })
                    .sum::<usize>()
            };
            let work = edge_count(area)
                .checked_mul(shape_count(other))
                .and_then(|a| {
                    edge_count(other)
                        .checked_mul(shape_count(area))
                        .and_then(|b| a.checked_add(b))
                })
                .ok_or("D05603_AREA_PERIMETER_WORK_LIMIT")?;
            perimeter_work = perimeter_work
                .checked_add(work)
                .ok_or("D05603_AREA_PERIMETER_WORK_LIMIT")?;
            if perimeter_work > 8_000_000 {
                return Err("D05603_AREA_PERIMETER_WORK_LIMIT");
            }
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
            let proven_by_sample = samples
                .iter()
                .any(|&p| strictly_in_area(area, p).unwrap_or(false));
            let proven_by_boundary_events = if !proven_by_sample
                && matches!(&candidate.footprint, CopperFootprint::Stroke { .. })
            {
                crate::straight_area_witness::positive_straight_area_witness(&strokes[index], area)?
            } else {
                false
            };
            let proven_by_rectangular_pad = if !proven_by_sample && !proven_by_boundary_events {
                match &candidate.footprint {
                    CopperFootprint::SmdPad {
                        centre,
                        width_ticks,
                        height_ticks,
                        rectangular: true,
                    } => positive_rectangular_smd_area_witness(
                        area,
                        *centre,
                        *width_ticks,
                        *height_ticks,
                    )?,
                    _ => false,
                }
            } else {
                false
            };
            if proven_by_sample || proven_by_boundary_events || proven_by_rectangular_pad {
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
                    if graph.geometry_contacts.len() + new_edges.len() >= MAX_CONTACTS {
                        return Err("D05603_AREA_EDGE_LIMIT");
                    }
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
            let crossed_perimeter = if !identical && !mutual {
                polygon_perimeter_overlap(area, other)? || polygon_perimeter_overlap(other, area)?
            } else {
                false
            };
            if identical || mutual || crossed_perimeter {
                let (a, b) = (preexisting + old_index, new_index);
                if seen.insert((a, b, ContactKind::CopperAreaCopperArea)) {
                    if graph.geometry_contacts.len() + new_edges.len() >= MAX_CONTACTS {
                        return Err("D05603_AREA_EDGE_LIMIT");
                    }
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
