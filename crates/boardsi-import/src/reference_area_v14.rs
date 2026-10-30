//! C_022 V14: bounded 3..=6 source-owned rectangles and geometric components.
//! Positive-area intersections are CANDIDATES, never electrical connectivity.
use crate::{copper_preview, digest, reference_area_v13, route_preview};
use copper_preview::CopperGeometry;
use reference_area_v13::SourceState;
use std::fmt::Write;

const MAX_WIRE_BYTES: usize = 8192;

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("write to string");
    }
    out
}

/// Deterministic, order-independent geometric candidate partition.
/// Component IDs are minimum selected input index, NOT electrical Net IDs.
pub fn components(rects: &[[i64; 4]]) -> Result<(Vec<usize>, usize), &'static str> {
    if !(3..=6).contains(&rects.len()) {
        return Err("C022_AREA_COUNT_INVALID");
    }
    let mut parent: Vec<usize> = (0..rects.len()).collect();
    let mut boundary_pairs = 0usize;
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            match reference_area_v13::rectangle_relation(rects[i], rects[j])? {
                SourceState::PositiveAreaOverlap => {
                    let lo = parent[i].min(parent[j]);
                    let hi = parent[i].max(parent[j]);
                    for group in &mut parent {
                        if *group == hi {
                            *group = lo;
                        }
                    }
                }
                SourceState::BoundaryOnly => boundary_pairs += 1,
                SourceState::DisconnectedIsland => {}
            }
        }
    }
    Ok((parent, boundary_pairs))
}

/// Explicit full source inventory only, never an arbitrary subset hidden within
/// a larger board. V13, V1-V12 and the canonical connectivity graph stay intact.
pub fn encode(
    bytes: &[u8],
    area_names: &[&str],
    reference_net: &str,
    reference_layer: usize,
) -> Result<String, &'static str> {
    if !(3..=6).contains(&area_names.len())
        || reference_net.is_empty()
        || reference_layer == 0
        || area_names
            .iter()
            .any(|name| name.is_empty() || name.len() > 64)
    {
        return Err("C022_INVALID_SELECTION");
    }
    for i in 0..area_names.len() {
        if area_names[..i].contains(&area_names[i]) {
            return Err("C022_DUPLICATE_SELECTION");
        }
    }
    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &areas)?;
    if board.validate().has_errors()
        || board.areas.len() != areas.len()
        || areas.len() != area_names.len()
    {
        return Err("C022_SOURCE_INVENTORY_MISMATCH");
    }
    let canonical_layer = &board
        .stackup
        .layers
        .get(reference_layer - 1)
        .ok_or("C022_REFERENCE_LAYER_INVALID")?
        .meta
        .id;
    let mut bounds = Vec::with_capacity(areas.len());
    let mut ids = Vec::with_capacity(areas.len());
    for name in area_names {
        let index = areas
            .iter()
            .position(|area| area.name == *name)
            .ok_or("C022_SOURCE_AREA_UNKNOWN")?;
        let source = &areas[index];
        let canonical = &board.areas[index];
        if source.layer as usize != reference_layer
            || canonical.layer != *canonical_layer
            || source.net_label.as_deref() != Some(reference_net)
            || canonical.net.is_some()
            || !source.cutouts.is_empty()
            || !canonical.cutouts.is_empty()
            || canonical.outer.segments.len() != 4
        {
            return Err("C022_UNSUPPORTED_SOURCE_GEOMETRY");
        }
        let CopperGeometry::Polygon(vertices) = &source.geometry else {
            return Err("C022_UNSUPPORTED_SOURCE_GEOMETRY");
        };
        let rect = copper_preview::rectangle_bounds(vertices).ok_or("C022_NONRECTANGLE")?;
        for (segment, points) in canonical.outer.segments.iter().zip(vertices.windows(2)) {
            let boardsi_model::BoundarySegment::Line { start, end } = segment else {
                return Err("C022_CANONICAL_RING_MISMATCH");
            };
            if start.ticks != points[0] || end.ticks != points[1] || start.frame != end.frame {
                return Err("C022_CANONICAL_RING_MISMATCH");
            }
        }
        bounds.push(rect);
        ids.push(hex(canonical.meta.id.source_object.as_bytes()));
    }
    let (groups, boundary_pairs) = components(&bounds)?;
    let number_of_groups = groups
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let source_sha = hex(&digest::sha256(bytes));
    let mut wire = format!(
        "BOARDSI_REFERENCE_AREA_V14|{source_sha}|{}|{number_of_groups}|{}|{boundary_pairs}|UNSUPPORTED|NOT_RUN|0|0\n",
        area_names.len(),
        if number_of_groups > 1 { "DISCONNECTED_ISLAND_RISK" }
        else if boundary_pairs > 0 { "BOUNDARY_ONLY_RISK" }
        else { "POSITIVE_AREA_OVERLAP_CANDIDATE" }
    );
    for (i, name) in area_names.iter().enumerate() {
        let rect = bounds[i];
        writeln!(
            &mut wire,
            "AREA|{i}|{}|{}|{reference_layer}|{}|{}|{}|{}|{}|{}",
            hex(name.as_bytes()),
            ids[i],
            rect[0],
            rect[1],
            rect[2],
            rect[3],
            hex(reference_net.as_bytes()),
            groups[i]
        )
        .map_err(|_| "C022_OUTPUT_ERROR")?;
    }
    writeln!(
        &mut wire,
        "END|{}|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL",
        area_names.len()
    )
    .map_err(|_| "C022_OUTPUT_ERROR")?;
    if wire.len() > MAX_WIRE_BYTES {
        return Err("C022_OUTPUT_LIMIT");
    }
    Ok(wire)
}
