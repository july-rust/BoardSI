//! D_055 generic finite-width SOURCE copper overlap for trace/via/pad.
//!
//! These are *geometry-only local hypotheses*, not electrical connectivity,
//! manufactured conduction, shorts, return paths, or solver/IBIS evidence.
//! No assumption that a trace endpoint passes through a pad or via centre.
use crate::part_preview::{self, ComponentPad};
use crate::route_preview::{self, ViaInstance};
use crate::trace_contact::{self, Contact, Stroke, BOUNDARY_GUARD_TICKS};
use crate::via_shapes::{NormalizedViaShapes, ViaLayerShape};
use crate::via_topology::NormalizedVia;
use std::collections::BTreeMap;

const MAX_TICKS: i64 = 5_600_000;
const MAX_NARROW_PHASE_PAIRS: usize = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Via(usize),
    SmdPad(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclaredNetRelation {
    Same,
    Different,
    PadNetUnverified,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub trace_index: usize,
    pub trace_source_line: usize,
    pub target: Target,
    pub target_identity: String,
    pub net_relation: DeclaredNetRelation,
}

/// Strictly positive copper area (not centre proximity) on an explicitly
/// conductive via-pad layer. Drill-only overlap is NOT conductive copper.
pub fn trace_to_via(trace: &Stroke, via: &ViaInstance) -> Result<Contact, &'static str> {
    if !(1..=64).contains(&via.first_layer)
        || via.last_layer < via.first_layer
        || via.last_layer > 64
        || via.net.is_empty()
        || via.name.is_empty()
        || via.point.iter().any(|p| p.abs() > MAX_TICKS)
        || via.diameter_ticks <= 0
        || via.diameter_ticks > MAX_TICKS
        || via.drill_ticks <= 0
        || via.drill_ticks >= via.diameter_ticks
    {
        return Err("INVALID_VIA_COPPER_FOOTPRINT");
    }
    if let Some(profile) = &via.shaped {
        if via.first_layer != profile.first_layer
            || via.last_layer != profile.last_layer
            || via.drill_ticks != profile.drill_ticks
            || via.diameter_ticks
                != profile
                    .layer_pad(profile.first_layer)
                    .map(|p| p.size_ticks)
                    .unwrap_or(0)
        {
            return Err("D056_VIA_SOURCE_TOPOLOGY_MISMATCH");
        }
        return trace_to_layered_via(trace, via.point, profile);
    }
    if let Some(normalized) = &via.topology {
        if via.first_layer != normalized.first_layer
            || via.last_layer != normalized.last_layer
            || via.drill_ticks != normalized.drill_ticks
            || via.diameter_ticks != normalized.diameter_on(normalized.first_layer).unwrap_or(0)
        {
            return Err("D056_VIA_SOURCE_TOPOLOGY_MISMATCH");
        }
        return trace_to_normalized_via(trace, via.point, normalized);
    }
    let layer = trace.layer;
    if layer < via.first_layer
        || layer > via.last_layer
        || (layer != via.first_layer && layer != via.last_layer && !via.inner_pad)
    {
        // Validate trace even when its layer does not participate.
        let _ = trace_contact::centreline_envelope(trace)?;
        return Ok(Contact::Separated);
    }
    annular_contact(trace, via.point, via.diameter_ticks, via.drill_ticks)
}

/// A single annulus kernel shared by the legacy restricted SOURCE contact
/// path and normalized topology with per-layer conductive diameters.
fn annular_contact(
    trace: &Stroke,
    point: [i64; 2],
    diameter_ticks: i64,
    drill_ticks: i64,
) -> Result<Contact, &'static str> {
    let dmin = trace_contact::centreline_point_distance(trace, point)?;
    let dmax = trace_contact::centreline_point_farthest(trace, point)?;
    let stroke_radius = trace.width_ticks as f64 / 2.0;
    let outer_clearance = dmin - (diameter_ticks as f64 / 2.0 + stroke_radius);
    let inner_clearance = (drill_ticks as f64 / 2.0 - stroke_radius) - dmax;
    if !outer_clearance.is_finite() || !inner_clearance.is_finite() {
        return Err("NONFINITE_VIA_CONTACT");
    }
    // Both exterior annulus and its drilled aperture matter: a trace wholly
    // inside the bore must never become a positive copper contact.
    Ok(
        if outer_clearance > BOUNDARY_GUARD_TICKS || inner_clearance > BOUNDARY_GUARD_TICKS {
            Contact::Separated
        } else if outer_clearance >= -BOUNDARY_GUARD_TICKS
            || inner_clearance >= -BOUNDARY_GUARD_TICKS
        {
            Contact::BoundaryUncertain
        } else {
            Contact::PositiveArea
        },
    )
}

/// Data-driven geometry path: no fixed layer count, span, or pad diameter.
/// Plating/net continuity is a separate canonical graph qualification.
pub fn trace_to_normalized_via(
    trace: &Stroke,
    point: [i64; 2],
    via: &NormalizedVia,
) -> Result<Contact, &'static str> {
    via.validate()?;
    if point.iter().any(|p| !(-MAX_TICKS..=MAX_TICKS).contains(p)) {
        return Err("INVALID_VIA_COPPER_FOOTPRINT");
    }
    let Some(diameter) = via.diameter_on(trace.layer) else {
        // Even a nonparticipating layer must not hide malformed trace data.
        let _ = trace_contact::centreline_envelope(trace)?;
        return Ok(Contact::Separated);
    };
    annular_contact(trace, point, diameter, via.drill_ticks)
}

/// Shape-aware positive area contact. Plane anti-pads are explicit voids,
/// and must never create a trace-to-via conductive contact candidate.
pub fn trace_to_layered_via(
    trace: &Stroke,
    point: [i64; 2],
    via: &NormalizedViaShapes,
) -> Result<Contact, &'static str> {
    via.validate()?;
    if point.iter().any(|p| !(-MAX_TICKS..=MAX_TICKS).contains(p)) {
        return Err("INVALID_VIA_COPPER_FOOTPRINT");
    }
    let Some(pad) = via.conductive_pad(trace.layer) else {
        let _ = trace_contact::centreline_envelope(trace)?;
        return Ok(Contact::Separated);
    };
    if pad.shape == ViaLayerShape::Round {
        return annular_contact(trace, point, pad.size_ticks, via.drill_ticks);
    }
    if let ViaLayerShape::Annular {
        inner_diameter_ticks,
    } = pad.shape
    {
        return annular_contact(trace, point, pad.size_ticks, inner_diameter_ticks);
    }
    if let ViaLayerShape::Thermal {
        kind,
        first_spoke_degrees,
        outer_size_ticks,
        spoke_width_ticks,
        spoke_count,
    } = pad.shape
    {
        return crate::thermal_contact::trace_thermal_candidate(
            trace,
            point,
            via.drill_ticks,
            &crate::thermal_source::ThermalPad {
                layer_selector: i64::from(trace.layer),
                kind,
                size_ticks: pad.size_ticks,
                first_spoke_degrees,
                outer_size_ticks,
                spoke_width_ticks,
                spoke_count,
            },
        );
    }
    if pad.shape != ViaLayerShape::Square {
        return Err("D056_UNEXPECTED_CONDUCTIVE_PAD_SHAPE");
    }
    // A square external boundary with an explicitly circular drilled hole.
    // Both outer and bore distances must prove positive copper area.
    let distance =
        trace_contact::centreline_rectangle_distance(trace, point, pad.size_ticks, pad.size_ticks)?;
    let dmax = trace_contact::centreline_point_farthest(trace, point)?;
    let trace_radius = trace.width_ticks as f64 / 2.0;
    let outer_clearance = distance - trace_radius;
    let inner_clearance = (via.drill_ticks as f64 / 2.0 - trace_radius) - dmax;
    if !outer_clearance.is_finite() || !inner_clearance.is_finite() {
        return Err("NONFINITE_VIA_CONTACT");
    }
    Ok(
        if outer_clearance > BOUNDARY_GUARD_TICKS || inner_clearance > BOUNDARY_GUARD_TICKS {
            Contact::Separated
        } else if outer_clearance >= -BOUNDARY_GUARD_TICKS
            || inner_clearance >= -BOUNDARY_GUARD_TICKS
        {
            Contact::BoundaryUncertain
        } else {
            Contact::PositiveArea
        },
    )
}

/// Generic footprint intersection for the original supported SMD shapes.
/// Rectangles include every edge and corner. Circle contact uses actual radius.
pub fn trace_to_pad(trace: &Stroke, pad: &ComponentPad) -> Result<Contact, &'static str> {
    if !(1..=64).contains(&pad.layer)
        || pad.reference.is_empty()
        || pad.pin.is_empty()
        || pad
            .point
            .iter()
            .any(|p| !(-MAX_TICKS..=MAX_TICKS).contains(p))
        || pad.width_ticks <= 0
        || pad.height_ticks <= 0
        || pad.width_ticks > MAX_TICKS
        || pad.height_ticks > MAX_TICKS
        || (pad.square && pad.rectangular)
        || (!pad.rectangular && pad.width_ticks != pad.height_ticks)
    {
        return Err("INVALID_SMD_COPPER_FOOTPRINT");
    }
    if trace.layer != pad.layer {
        let _ = trace_contact::centreline_envelope(trace)?;
        return Ok(Contact::Separated);
    }
    let distance = if pad.square || pad.rectangular {
        trace_contact::centreline_rectangle_distance(
            trace,
            pad.point,
            pad.width_ticks,
            pad.height_ticks,
        )?
    } else {
        trace_contact::centreline_point_distance(trace, pad.point)?
    };
    // Rectangles include their interior, so only the trace half-width is
    // subtracted from the centreline's distance to the full copper footprint.
    let limit = if pad.square || pad.rectangular {
        trace.width_ticks as f64 / 2.0
    } else {
        (trace.width_ticks as f64 + pad.width_ticks as f64) / 2.0
    };
    trace_contact::classify_proximity(distance, limit)
}

#[derive(Clone)]
struct Footprint {
    kind: Target,
    box_xy: [f64; 4],
}
fn target_envelope(kind: Target, vias: &[ViaInstance], pads: &[ComponentPad]) -> [f64; 4] {
    let (c, rx, ry) = match kind {
        Target::Via(i) => {
            let v = &vias[i];
            // Broadphase must enclose the whole SOURCE copper, not only the
            // central land of an RT/ST thermal. It may overapproximate but
            // must never drop a real spoke-contact candidate.
            let diameter = v
                .shaped
                .as_ref()
                .and_then(|via| {
                    via.pads
                        .iter()
                        .map(|pad| match pad.shape {
                            ViaLayerShape::Thermal {
                                outer_size_ticks, ..
                            } => outer_size_ticks,
                            _ => pad.size_ticks,
                        })
                        .max()
                })
                .or_else(|| {
                    v.topology
                        .as_ref()
                        .and_then(|via| via.pads.iter().map(|pad| pad.diameter_ticks).max())
                })
                .unwrap_or(v.diameter_ticks);
            (v.point, diameter as f64 / 2.0, diameter as f64 / 2.0)
        }
        Target::SmdPad(i) => {
            let p = &pads[i];
            (
                p.point,
                p.width_ticks as f64 / 2.0,
                p.height_ticks as f64 / 2.0,
            )
        }
    };
    [
        c[0] as f64 - rx,
        c[1] as f64 - ry,
        c[0] as f64 + rx,
        c[1] as f64 + ry,
    ]
}

/// Bounded broadphase and one shared narrow-phase, not an exhaustive list of
/// pad corners, via orientations, trace endpoint locations or trace angles.
///
/// An uncertain tangency aborts the whole diagnostic (fail closed) rather
/// than quietly omitting an electrical candidate.
pub fn contact_candidates(
    strokes: &[Stroke],
    vias: &[ViaInstance],
    pads: &[ComponentPad],
    pin_nets: &BTreeMap<String, String>,
) -> Result<Vec<Candidate>, &'static str> {
    let mut targets = Vec::with_capacity(vias.len() + pads.len());
    for i in 0..vias.len() {
        targets.push(Footprint {
            kind: Target::Via(i),
            box_xy: target_envelope(Target::Via(i), vias, pads),
        });
    }
    for i in 0..pads.len() {
        targets.push(Footprint {
            kind: Target::SmdPad(i),
            box_xy: target_envelope(Target::SmdPad(i), vias, pads),
        });
    }
    // Fail before broadphase, including target objects on another layer.
    // Source parse cannot silently hide a malformed pad/via behind a bounding box.
    for v in vias {
        if let Some(normalized) = &v.topology {
            normalized.validate()?;
        }
        if let Some(profile) = &v.shaped {
            profile.validate()?;
        }
        if v.diameter_ticks <= v.drill_ticks
            || v.drill_ticks <= 0
            || v.diameter_ticks > MAX_TICKS
            || v.point
                .iter()
                .any(|p| !(-MAX_TICKS..=MAX_TICKS).contains(p))
            || v.first_layer == 0
            || v.last_layer < v.first_layer
            || v.last_layer > 64
            || v.name.is_empty()
            || v.net.is_empty()
        {
            return Err("INVALID_VIA_COPPER_FOOTPRINT");
        }
    }
    for p in pads {
        if p.width_ticks <= 0
            || p.height_ticks <= 0
            || p.width_ticks > MAX_TICKS
            || p.height_ticks > MAX_TICKS
            || p.point
                .iter()
                .any(|v| !(-MAX_TICKS..=MAX_TICKS).contains(v))
            || p.layer == 0
            || p.layer > 64
            || p.pin.is_empty()
            || p.reference.is_empty()
            || (p.square && p.rectangular)
            || (!p.rectangular && p.width_ticks != p.height_ticks)
        {
            return Err("INVALID_SMD_COPPER_FOOTPRINT");
        }
    }
    targets.sort_by(|a, b| a.box_xy[0].total_cmp(&b.box_xy[0]));
    let mut out = Vec::new();
    let mut checked = 0usize;
    for (i, stroke) in strokes.iter().enumerate() {
        let bbox = trace_contact::centreline_envelope(stroke)?;
        for footprint in targets.iter() {
            let pbox = footprint.box_xy;
            if pbox[0] > bbox[2] + BOUNDARY_GUARD_TICKS {
                break;
            }
            if pbox[2] < bbox[0] - BOUNDARY_GUARD_TICKS
                || pbox[1] > bbox[3] + BOUNDARY_GUARD_TICKS
                || pbox[3] < bbox[1] - BOUNDARY_GUARD_TICKS
            {
                continue;
            }
            checked += 1;
            if checked > MAX_NARROW_PHASE_PAIRS {
                return Err("PAD_VIA_CONTACT_PAIR_BUDGET");
            }
            let (status, identity, relation) = match footprint.kind {
                Target::Via(j) => {
                    let v = &vias[j];
                    let rel = if stroke.net == v.net {
                        DeclaredNetRelation::Same
                    } else {
                        DeclaredNetRelation::Different
                    };
                    (
                        trace_to_via(stroke, v)?,
                        format!("{}@{}", v.name, v.source_line),
                        rel,
                    )
                }
                Target::SmdPad(j) => {
                    let p = &pads[j];
                    let name = format!("{}.{}", p.reference, p.pin);
                    let rel = match pin_nets.get(&name) {
                        Some(net) if net == &stroke.net => DeclaredNetRelation::Same,
                        Some(_) => DeclaredNetRelation::Different,
                        None => DeclaredNetRelation::PadNetUnverified,
                    };
                    (trace_to_pad(stroke, p)?, name, rel)
                }
            };
            match status {
                Contact::PositiveArea => out.push(Candidate {
                    trace_index: i,
                    trace_source_line: stroke.source_line,
                    target: footprint.kind,
                    target_identity: identity,
                    net_relation: relation,
                }),
                Contact::Separated => {}
                Contact::BoundaryUncertain => return Err("UNCERTAIN_PAD_VIA_COPPER_TANGENCY"),
            }
        }
    }
    out.sort_by_key(|c| {
        (
            c.trace_index,
            match c.target {
                Target::Via(index) => index,
                Target::SmdPad(index) => vias.len() + index,
            },
        )
    });
    Ok(out)
}

/// Genuine source-owned PADS V9.4 inventory; no fabricated pad pin Net.
pub fn source_candidates(bytes: &[u8]) -> Result<Vec<Candidate>, &'static str> {
    let art = route_preview::parse(bytes)?;
    let strokes = trace_contact::from_artwork(&art)?;
    let imported = part_preview::parse(bytes, art.max_layer);
    if imported.skipped != 0 {
        return Err("UNSUPPORTED_SOURCE_PAD_INVENTORY");
    }
    let pin_nets = part_preview::declared_route_pin_nets(bytes)?;
    contact_candidates(&strokes, &art.vias, &imported.pads, &pin_nets)
}

/// Opt-in native partial-via source candidates. Never used by legacy
/// Qt V1-V14 wire, and never promoted to global electrical graph proof.
pub fn source_candidates_generalized(bytes: &[u8]) -> Result<Vec<Candidate>, &'static str> {
    let art = route_preview::parse_generalized(bytes)?;
    if art.skipped != 0 {
        return Err("D056_UNSUPPORTED_SOURCE_ROUTE_INVENTORY");
    }
    let strokes = trace_contact::from_artwork(&art)?;
    let imported = part_preview::parse(bytes, art.max_layer);
    if imported.skipped != 0 {
        return Err("UNSUPPORTED_SOURCE_PAD_INVENTORY");
    }
    let pin_nets = part_preview::declared_route_pin_nets(bytes)?;
    contact_candidates(&strokes, &art.vias, &imported.pads, &pin_nets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace_contact::Centreline;

    fn line(a: [i64; 2], b: [i64; 2], width: i64, layer: u32) -> Stroke {
        Stroke {
            layer,
            net: "NET_A".to_owned(),
            width_ticks: width,
            source_line: 10,
            centreline: Centreline::Straight { start: a, end: b },
        }
    }
    fn arc(start: [i64; 2], end: [i64; 2]) -> Stroke {
        Stroke {
            layer: 1,
            net: "NET_A".to_owned(),
            width_ticks: 8,
            source_line: 11,
            centreline: Centreline::Circular {
                centre: [0, 0],
                start,
                end,
                clockwise: false,
            },
        }
    }
    fn via() -> ViaInstance {
        ViaInstance {
            name: "PLATED".to_owned(),
            net: "NET_A".to_owned(),
            point: [0, 0],
            diameter_ticks: 100,
            drill_ticks: 40,
            first_layer: 1,
            last_layer: 4,
            inner_pad: false,
            topology: None,
            shaped: None,
            source_line: 13,
        }
    }
    fn pad(square: bool, rectangular: bool) -> ComponentPad {
        ComponentPad {
            reference: "U1".into(),
            pin: "1".into(),
            point: [0, 0],
            component_origin: [0, 0],
            rotation: 0,
            layer: 1,
            size_ticks: 100,
            square,
            rectangular,
            bottom: false,
            width_ticks: 100,
            height_ticks: if rectangular { 60 } else { 100 },
        }
    }
    #[test]
    fn d05606_spoke_outer_broadphase_matches_narrowphase() {
        use crate::thermal_source::ThermalKind;
        use crate::via_shapes::ViaLayerFootprint;
        let mut thermal = via();
        thermal.diameter_ticks = 6000;
        thermal.drill_ticks = 1500;
        thermal.shaped = Some(NormalizedViaShapes {
            copper_layers: 4,
            first_layer: 1,
            last_layer: 4,
            kind: boardsi_model::ViaKind::Through,
            drill_ticks: 1500,
            plated: true,
            pads: vec![
                ViaLayerFootprint {
                    layer: 1,
                    size_ticks: 6000,
                    shape: ViaLayerShape::Thermal {
                        kind: ThermalKind::Round,
                        first_spoke_degrees: 0,
                        outer_size_ticks: 8400,
                        spoke_width_ticks: 1500,
                        spoke_count: 4,
                    },
                },
                ViaLayerFootprint {
                    layer: 4,
                    size_ticks: 6000,
                    shape: ViaLayerShape::Round,
                },
            ],
        });
        let on_spoke = line([3500, 0], [4000, 0], 100, 1);
        assert_eq!(trace_to_via(&on_spoke, &thermal), Ok(Contact::PositiveArea));
        let vias = [thermal];
        let results = contact_candidates(&[on_spoke], &vias, &[], &BTreeMap::new())
            .expect("same broadphase and narrowphase must retain a positive spoke witness");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].target, Target::Via(0));

        // Near-relief clearance can conservatively be BoundaryUncertain;
        // it must never be promoted to a verified positive contact.
        let in_clearance = line([3500, 1500], [4000, 1500], 100, 1);
        assert_ne!(
            trace_to_via(&in_clearance, &vias[0]),
            Ok(Contact::PositiveArea)
        );
        // A strictly outside-envelope witness is provably separated.
        let off_spoke = line([3500, 3000], [4000, 3000], 100, 1);
        assert_eq!(trace_to_via(&off_spoke, &vias[0]), Ok(Contact::Separated));
        assert!(
            contact_candidates(&[off_spoke], &vias, &[], &BTreeMap::new())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn d056_native_partial_source_candidates_are_layer_qualified() {
        let bytes = b"!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n\
*PCB*\nMAXLAYER 6\n*VIA*\n\
V_TEST 15 2 2 4\n-2 35 R\n0 40 R\n\
*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n\
100 200 2 10 0\n300 200 4 10 0 V_TEST\n500 200 4 10 0\n\
U3.1 U4.2\n100 100 3 10 0\n300 200 3 10 0\n*END*\n";
        let candidates = source_candidates_generalized(bytes).unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|candidate| {
            candidate.target == Target::Via(0)
                && candidate.net_relation == DeclaredNetRelation::Same
        }));
        // The third trace reaches the same XY, but Layer 3 has no
        // explicitly declared via conductive pad.
        assert_eq!(candidates.iter().filter(|c| c.trace_index == 2).count(), 0);
        let wrong_span = String::from_utf8(bytes.to_vec())
            .unwrap()
            .replace("V_TEST 15 2 2 4", "V_TEST 15 2 2 3");
        assert_eq!(
            source_candidates_generalized(wrong_span.as_bytes()).unwrap_err(),
            "D056_ROUTE_OUTSIDE_VIA_SPAN"
        );
    }

    #[test]
    fn d056_square_copper_and_plane_void_qualify_by_actual_layer() {
        use crate::via_shapes::ViaLayerFootprint;
        let shaped = NormalizedViaShapes {
            copper_layers: 6,
            first_layer: 2,
            last_layer: 4,
            kind: boardsi_model::ViaKind::Buried,
            drill_ticks: 40,
            plated: true,
            pads: vec![
                ViaLayerFootprint {
                    layer: 2,
                    size_ticks: 100,
                    shape: ViaLayerShape::Square,
                },
                ViaLayerFootprint {
                    layer: 3,
                    size_ticks: 150,
                    shape: ViaLayerShape::RoundAntiPad,
                },
                ViaLayerFootprint {
                    layer: 4,
                    size_ticks: 100,
                    shape: ViaLayerShape::Round,
                },
            ],
        };
        assert_eq!(
            trace_to_layered_via(&line([-100, 45], [100, 45], 10, 2), [0, 0], &shaped),
            Ok(Contact::PositiveArea)
        );
        assert_eq!(
            trace_to_layered_via(&line([-100, 0], [100, 0], 10, 3), [0, 0], &shaped),
            Ok(Contact::Separated)
        );
        assert_eq!(
            trace_to_layered_via(&line([-100, 0], [100, 0], 10, 1), [0, 0], &shaped),
            Ok(Contact::Separated)
        );
        assert_eq!(
            trace_to_layered_via(&line([-5, 0], [5, 0], 4, 2), [0, 0], &shaped),
            Ok(Contact::Separated)
        );
    }

    #[test]
    fn d056_annular_inner_void_excludes_copper_contacts_inside_clearance() {
        use crate::via_shapes::ViaLayerFootprint;
        let ring = NormalizedViaShapes {
            copper_layers: 6,
            first_layer: 2,
            last_layer: 4,
            kind: boardsi_model::ViaKind::Buried,
            drill_ticks: 30,
            plated: true,
            pads: vec![
                ViaLayerFootprint {
                    layer: 2,
                    size_ticks: 100,
                    shape: ViaLayerShape::Square,
                },
                ViaLayerFootprint {
                    layer: 3,
                    size_ticks: 130,
                    shape: ViaLayerShape::Annular {
                        inner_diameter_ticks: 100,
                    },
                },
                ViaLayerFootprint {
                    layer: 4,
                    size_ticks: 100,
                    shape: ViaLayerShape::Round,
                },
            ],
        };
        assert_eq!(
            trace_to_layered_via(&line([-4, 0], [4, 0], 4, 3), [0, 0], &ring),
            Ok(Contact::Separated)
        );
        assert_eq!(
            trace_to_layered_via(&line([-100, 57], [100, 57], 10, 3), [0, 0], &ring),
            Ok(Contact::PositiveArea)
        );
    }

    #[test]
    fn d056_arbitrary_buried_span_and_unequal_layer_pads_share_annulus_kernel() {
        use crate::via_topology::CircularLayerPad;
        let mut via = NormalizedVia {
            copper_layers: 6,
            first_layer: 2,
            last_layer: 4,
            kind: boardsi_model::ViaKind::Buried,
            drill_ticks: 40,
            plated: true,
            pads: vec![
                CircularLayerPad {
                    layer: 2,
                    diameter_ticks: 100,
                },
                CircularLayerPad {
                    layer: 4,
                    diameter_ticks: 60,
                },
            ],
        };
        let wide = line([-100, 59], [100, 59], 20, 2);
        assert_eq!(
            trace_to_normalized_via(&wide, [0, 0], &via),
            Ok(Contact::PositiveArea)
        );
        assert_eq!(
            trace_to_normalized_via(&line([-100, 59], [100, 59], 20, 4), [0, 0], &via),
            Ok(Contact::Separated)
        );
        assert_eq!(
            trace_to_normalized_via(&line([-100, 39], [100, 39], 20, 4), [0, 0], &via),
            Ok(Contact::PositiveArea)
        );
        for layer in [1, 3, 5, 6] {
            assert_eq!(
                trace_to_normalized_via(&line([-100, 0], [100, 0], 20, layer), [0, 0], &via),
                Ok(Contact::Separated)
            );
        }
        assert_eq!(
            trace_to_normalized_via(&line([-5, 0], [5, 0], 4, 2), [0, 0], &via),
            Ok(Contact::Separated)
        );
        // Same code also handles the L2-L3 buried case, without introducing
        // a separate layer-pair-specific branch.
        via.last_layer = 3;
        via.pads[1].layer = 3;
        assert_eq!(
            trace_to_normalized_via(&line([-100, 39], [100, 39], 20, 3), [0, 0], &via),
            Ok(Contact::PositiveArea)
        );
    }

    #[test]
    fn off_centre_trace_contacts_conductive_annulus_not_just_via_centre() {
        let v = via();
        let tangent = line([-100, 60], [100, 60], 20, 1);
        assert_eq!(trace_to_via(&tangent, &v), Ok(Contact::BoundaryUncertain));
        assert_eq!(
            trace_to_via(&line([-100, 59], [100, 59], 20, 1), &v),
            Ok(Contact::PositiveArea)
        );
        assert_eq!(
            trace_to_via(&line([-100, 61], [100, 61], 20, 1), &v),
            Ok(Contact::Separated)
        );
        // Endpoints and entire segment lie within the copper-free drill aperture.
        assert_eq!(
            trace_to_via(&line([-5, 0], [5, 0], 4, 1), &v),
            Ok(Contact::Separated)
        );
        assert_eq!(
            trace_to_via(&line([-10, 0], [10, 0], 20, 1), &v),
            Ok(Contact::BoundaryUncertain)
        );
        assert_eq!(
            trace_to_via(&line([-11, 0], [11, 0], 20, 1), &v),
            Ok(Contact::PositiveArea)
        );
    }
    #[test]
    fn via_only_connects_explicit_conductive_layers() {
        let v = via();
        assert_eq!(
            trace_to_via(&line([0, 35], [100, 35], 10, 2), &v),
            Ok(Contact::Separated)
        );
        assert_eq!(
            trace_to_via(&line([0, 35], [100, 35], 10, 4), &v),
            Ok(Contact::PositiveArea)
        );
        let mut inner = via();
        inner.inner_pad = true;
        assert_eq!(
            trace_to_via(&line([0, 35], [100, 35], 10, 2), &inner),
            Ok(Contact::PositiveArea)
        );
    }
    #[test]
    fn whole_pad_surface_including_edges_corners_and_midspan() {
        let rect = pad(false, true);
        assert_eq!(
            trace_to_pad(&line([48, 28], [80, 60], 8, 1), &rect),
            Ok(Contact::PositiveArea)
        );
        assert_eq!(
            trace_to_pad(&line([-90, 0], [90, 0], 8, 1), &rect),
            Ok(Contact::PositiveArea)
        );
        assert_eq!(
            trace_to_pad(&line([50, 34], [70, 34], 8, 1), &rect),
            Ok(Contact::BoundaryUncertain)
        );
        assert_eq!(
            trace_to_pad(&line([50, 35], [70, 35], 8, 1), &rect),
            Ok(Contact::Separated)
        );
        assert_eq!(
            trace_to_pad(&line([50, 33], [70, 33], 8, 1), &rect),
            Ok(Contact::PositiveArea)
        );
        assert_eq!(
            trace_to_pad(&line([48, 28], [80, 60], 8, 2), &rect),
            Ok(Contact::Separated)
        );
        let round = pad(false, false);
        assert_eq!(
            trace_to_pad(&line([45, -100], [45, 100], 12, 1), &round),
            Ok(Contact::PositiveArea)
        );
        let square = pad(true, false);
        assert_eq!(
            trace_to_pad(&line([48, 48], [90, 90], 8, 1), &square),
            Ok(Contact::PositiveArea)
        );
    }
    #[test]
    fn arc_to_via_and_corner_pad_stays_analytic() {
        let mut big = via();
        big.diameter_ticks = 120;
        big.drill_ticks = 20;
        assert_eq!(
            trace_to_via(&arc([45, 0], [0, 45]), &big),
            Ok(Contact::PositiveArea)
        );
        let mut p = pad(false, true);
        p.point = [70, 70];
        assert_eq!(
            trace_to_pad(&arc([100, 0], [0, 100]), &p),
            Ok(Contact::PositiveArea)
        );
    }
    #[test]
    fn source_provenance_conflict_labels_and_missing_net_are_not_edges() {
        let mut v = via();
        v.net = "NET_B".into();
        let strokes = vec![line([-80, 34], [80, 34], 10, 1)];
        let pads = vec![pad(false, true)];
        let result = contact_candidates(&strokes, &[v], &pads, &BTreeMap::new()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result
                .iter()
                .find(|x| matches!(x.target, Target::Via(_)))
                .unwrap()
                .net_relation,
            DeclaredNetRelation::Different
        );
        assert_eq!(
            result
                .iter()
                .find(|x| matches!(x.target, Target::SmdPad(_)))
                .unwrap()
                .net_relation,
            DeclaredNetRelation::PadNetUnverified
        );
        let mut net = BTreeMap::new();
        net.insert("U1.1".to_owned(), "NET_A".to_owned());
        let result = contact_candidates(&strokes, &[], &pads, &net).unwrap();
        assert_eq!(result[0].net_relation, DeclaredNetRelation::Same);
    }
    #[test]
    fn translated_angular_families_use_one_overlap_rule() {
        let mut v = via();
        v.point = [1000, -1000];
        for deg in 0..360 {
            let angle = (deg as f64).to_radians();
            let x = (40.0 * angle.cos()).round() as i64;
            let y = (40.0 * angle.sin()).round() as i64;
            // Short segment entirely in annular copper, not through centre.
            let test = line(
                [1000 + x, -1000 + y],
                [1000 + x - y / 20, -1000 + y + x / 20],
                6,
                1,
            );
            assert_eq!(
                trace_to_via(&test, &v),
                Ok(Contact::PositiveArea),
                "source direction sampled at {deg}"
            );
        }
    }
    #[test]
    fn invalid_shapes_fail_closed() {
        let mut v = via();
        v.drill_ticks = 100;
        assert!(matches!(
            trace_to_via(&line([0, 0], [100, 0], 10, 1), &v),
            Err("INVALID_VIA_COPPER_FOOTPRINT")
        ));
        let mut p = pad(false, true);
        p.width_ticks = 0;
        assert!(matches!(
            trace_to_pad(&line([0, 0], [100, 0], 10, 1), &p),
            Err("INVALID_SMD_COPPER_FOOTPRINT")
        ));
    }
    #[test]
    fn original_source_off_centre_via_route_is_detected() {
        let original = include_str!("../tests/fixtures/pads_via_round_through_v94.asc");
        let modified = original.replace("*END*", "U3.1 U4.1\n310 210 1 5 0\n330 210 1 5 0\n*END*");
        let art = route_preview::parse(modified.as_bytes()).unwrap();
        assert_eq!(art.vias.len(), 1);
        assert_eq!(art.skipped, 0);
        let candidates = source_candidates(modified.as_bytes()).unwrap();
        let offcentre = art
            .segments
            .iter()
            .find(|t| t.a == [31_000, 21_000] && t.b == [33_000, 21_000])
            .unwrap();
        assert_ne!(offcentre.a, art.vias[0].point);
        assert_ne!(offcentre.b, art.vias[0].point);
        assert!(candidates
            .iter()
            .any(|c| c.trace_source_line == offcentre.source_line
                && matches!(c.target, Target::Via(0))));
    }
    #[test]
    fn original_source_square_pad_corner_attachment_is_detected() {
        let original = include_str!("../tests/fixtures/d040_multi_component_v94.asc");
        let modified =
            original.replace("*END*", "U1.1 U1.2\n215 415 1 10 0\n240 440 1 10 0\n*END*");
        let art = route_preview::parse(modified.as_bytes()).unwrap();
        assert_eq!(art.skipped, 0);
        let imported = part_preview::parse(modified.as_bytes(), art.max_layer);
        assert_eq!(imported.skipped, 0);
        let corner_trace = art
            .segments
            .iter()
            .find(|t| t.a == [21_500, 41_500] && t.b == [24_000, 44_000])
            .unwrap();
        let pad_index = imported
            .pads
            .iter()
            .position(|p| p.reference == "U1" && p.pin == "2")
            .unwrap();
        let corner_pad = &imported.pads[pad_index];
        assert_ne!(corner_trace.a, corner_pad.point);
        assert_ne!(corner_trace.b, corner_pad.point);
        assert!(source_candidates(modified.as_bytes())
            .unwrap()
            .iter()
            .any(|c| c.trace_source_line == corner_trace.source_line
                && c.target == Target::SmdPad(pad_index)));
    }
    #[test]
    fn original_source_via_and_smd_inventories_remain_accepted() {
        let bytes = include_bytes!("../tests/fixtures/pads_via_round_through_v94.asc");
        let candidates = source_candidates(bytes).unwrap();
        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .any(|c| matches!(c.target, Target::Via(_))));
        let bytes = include_bytes!("../tests/fixtures/d040_multi_component_v94.asc");
        let candidates = source_candidates(bytes).unwrap();
        assert!(candidates
            .iter()
            .any(|c| matches!(c.target, Target::SmdPad(_))));
    }
}
