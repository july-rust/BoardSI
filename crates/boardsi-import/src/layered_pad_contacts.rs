//! D_056_03 source-bound same-layer pad-to-pad *positive-area witnesses*.
//!
//! PADS source-normalized via/SMD footprints are NOT automatically linked
//! by Net name, drill span or coincident XY. This solver is sound-positive:
//! exact circular ring and solid rectangular intersection are analytical;
//! mixed square/round/hollow footprints accept only an explicit interior
//! witness in BOTH regions. Missing contact candidates remain UNKNOWN, not Open.
use crate::layered_source_graph::{
    ContactKind, CopperFootprint, DeclaredNetCheck, GeometryContact, LayeredSourceGraph,
};
use crate::via_shapes::ViaLayerShape;
use std::collections::{BTreeMap, BTreeSet};

const MAX_NARROW_PHASE: usize = 1_000_000;
const MAX_CONTACTS: usize = 100_000;

#[derive(Clone, Copy)]
enum LocalShape {
    Round {
        centre: [i64; 2],
        outer: i64,
        hole: i64,
    },
    SquareWithRoundHole {
        centre: [i64; 2],
        width: i64,
        height: i64,
        hole: i64,
    },
}
impl LocalShape {
    fn aabb(self) -> [i128; 4] {
        match self {
            Self::Round { centre, outer, .. } => {
                let x = i128::from(centre[0]) * 2;
                let y = i128::from(centre[1]) * 2;
                let r = i128::from(outer);
                [x - r, y - r, x + r, y + r]
            }
            Self::SquareWithRoundHole {
                centre,
                width,
                height,
                ..
            } => {
                let x = i128::from(centre[0]) * 2;
                let y = i128::from(centre[1]) * 2;
                [
                    x - i128::from(width),
                    y - i128::from(height),
                    x + i128::from(width),
                    y + i128::from(height),
                ]
            }
        }
    }
    fn centre(self) -> [i64; 2] {
        match self {
            Self::Round { centre, .. } | Self::SquareWithRoundHole { centre, .. } => centre,
        }
    }
    fn interior(self, p: [i64; 2]) -> bool {
        let c = self.centre();
        let x = (i128::from(p[0]) - i128::from(c[0])) * 2;
        let y = (i128::from(p[1]) - i128::from(c[1])) * 2;
        let squared = x * x + y * y;
        match self {
            Self::Round { outer, hole, .. } => {
                squared < i128::from(outer) * i128::from(outer)
                    && (hole == 0 || squared > i128::from(hole) * i128::from(hole))
            }
            Self::SquareWithRoundHole {
                width,
                height,
                hole,
                ..
            } => {
                x.abs() < i128::from(width)
                    && y.abs() < i128::from(height)
                    && squared > i128::from(hole) * i128::from(hole)
            }
        }
    }
}
fn footprint(footprint: &CopperFootprint) -> Result<Option<LocalShape>, &'static str> {
    let value = match *footprint {
        CopperFootprint::ViaPad {
            centre,
            outer_ticks,
            drill_ticks,
            shape,
        } => {
            let (round, hole) = match shape {
                ViaLayerShape::Round => (true, drill_ticks),
                ViaLayerShape::Square => (false, drill_ticks),
                ViaLayerShape::Annular {
                    inner_diameter_ticks,
                } => (true, inner_diameter_ticks),
                // Thermal shape is NOT a filled disk: absent a separate verified
                // spoke/plane overlap witness, local pad-pair stays UNKNOWN.
                ViaLayerShape::Thermal { .. } => return Ok(None),
                ViaLayerShape::RoundAntiPad | ViaLayerShape::SquareAntiPad => {
                    return Err("D05603_ANTIPAD_AS_COPPER")
                }
            };
            if outer_ticks <= 0 || hole < 0 || hole >= outer_ticks {
                return Err("D05603_INVALID_PAD_FOOTPRINT");
            }
            if round {
                Some(LocalShape::Round {
                    centre,
                    outer: outer_ticks,
                    hole,
                })
            } else {
                Some(LocalShape::SquareWithRoundHole {
                    centre,
                    width: outer_ticks,
                    height: outer_ticks,
                    hole,
                })
            }
        }
        CopperFootprint::SmdPad {
            centre,
            width_ticks,
            height_ticks,
            rectangular,
        } => {
            if width_ticks <= 0 || height_ticks <= 0 {
                return Err("D05603_INVALID_PAD_FOOTPRINT");
            }
            if rectangular {
                Some(LocalShape::SquareWithRoundHole {
                    centre,
                    width: width_ticks,
                    height: height_ticks,
                    hole: 0,
                })
            } else {
                if width_ticks != height_ticks {
                    return Err("D05603_NONCIRCULAR_ROUND_SMD");
                }
                Some(LocalShape::Round {
                    centre,
                    outer: width_ticks,
                    hole: 0,
                })
            }
        }
        _ => None,
    };
    Ok(value)
}
fn distinct_area(x: LocalShape, y: LocalShape) -> bool {
    match (x, y) {
        (
            LocalShape::Round {
                centre: a,
                outer: ra,
                hole: ha,
            },
            LocalShape::Round {
                centre: b,
                outer: rb,
                hole: hb,
            },
        ) => {
            // Exact strict overlap of two circular annuli, tested with
            // doubled-radius integer arithmetic to avoid f64 tangency errors.
            let dx = i128::from(a[0]) - i128::from(b[0]);
            let dy = i128::from(a[1]) - i128::from(b[1]);
            let four_d2 = 4 * (dx * dx + dy * dy);
            let outer_sum = i128::from(ra) + i128::from(rb);
            let ha_minus_rb = i128::from(ha) - i128::from(rb);
            let hb_minus_ra = i128::from(hb) - i128::from(ra);
            let outside_a_hole = ha_minus_rb < 0
                || (ha_minus_rb == 0 && four_d2 > 0)
                || (ha_minus_rb > 0 && four_d2 > ha_minus_rb * ha_minus_rb);
            let outside_b_hole = hb_minus_ra < 0
                || (hb_minus_ra == 0 && four_d2 > 0)
                || (hb_minus_ra > 0 && four_d2 > hb_minus_ra * hb_minus_ra);
            four_d2 < outer_sum * outer_sum && outside_a_hole && outside_b_hole
        }
        (
            LocalShape::SquareWithRoundHole {
                centre: a,
                width: ax,
                height: ay,
                hole: 0,
            },
            LocalShape::SquareWithRoundHole {
                centre: b,
                width: bx,
                height: by,
                hole: 0,
            },
        ) => {
            let dx = 2 * (i128::from(a[0]) - i128::from(b[0])).abs();
            let dy = 2 * (i128::from(a[1]) - i128::from(b[1])).abs();
            dx < i128::from(ax) + i128::from(bx) && dy < i128::from(ay) + i128::from(by)
        }
        _ => {
            // A grid witness can only establish *positive* intersection.
            // It cannot establish separation for holes/curved outlines.
            let a = x.aabb();
            let b = y.aabb();
            let left = a[0].max(b[0]);
            let right = a[2].min(b[2]);
            let bottom = a[1].max(b[1]);
            let top = a[3].min(b[3]);
            if left >= right || bottom >= top {
                return false;
            }
            let mut points = vec![x.centre(), y.centre()];
            for ix in 1..16 {
                for iy in 1..16 {
                    let px = (left * (16 - ix) + right * ix) / 16;
                    let py = (bottom * (16 - iy) + top * iy) / 16;
                    // Source coordinates are integral ticks; do not round a
                    // sub-tick witness into false copper.
                    if px % 2 == 0 && py % 2 == 0 {
                        if let (Ok(px), Ok(py)) = (i64::try_from(px / 2), i64::try_from(py / 2)) {
                            points.push([px, py]);
                        }
                    }
                }
            }
            points.into_iter().any(|p| x.interior(p) && y.interior(p))
        }
    }
}
fn relation(a: Option<&String>, b: Option<&String>) -> DeclaredNetCheck {
    match (a, b) {
        (Some(x), Some(y)) if x == y => DeclaredNetCheck::Same,
        (Some(_), Some(_)) => DeclaredNetCheck::Different,
        _ => DeclaredNetCheck::Unknown,
    }
}
fn kind(a: &CopperFootprint, b: &CopperFootprint) -> Result<ContactKind, &'static str> {
    match (a, b) {
        (CopperFootprint::ViaPad { .. }, CopperFootprint::ViaPad { .. }) => {
            Ok(ContactKind::ViaPadViaPad)
        }
        (CopperFootprint::ViaPad { .. }, CopperFootprint::SmdPad { .. })
        | (CopperFootprint::SmdPad { .. }, CopperFootprint::ViaPad { .. }) => {
            Ok(ContactKind::ViaPadSmdPad)
        }
        (CopperFootprint::SmdPad { .. }, CopperFootprint::SmdPad { .. }) => {
            Ok(ContactKind::SmdPadSmdPad)
        }
        _ => Err("D05603_NON_PAD_PAIR"),
    }
}
/// Local pad pairs only. This mutates the graph transactionally: a failure
/// leaves the original source candidate inventory untouched.
pub fn augment_pad_contacts(graph: &mut LayeredSourceGraph) -> Result<(), &'static str> {
    let mut by_layer: BTreeMap<u32, Vec<(usize, LocalShape)>> = BTreeMap::new();
    for (id, node) in graph.nodes.iter().enumerate() {
        if let Some(shape) = footprint(&node.footprint)? {
            if node.physical_copper_layer == 0
                || node.physical_copper_layer > graph.declared_copper_layers
            {
                return Err("D05603_INVALID_PAD_LAYER");
            }
            by_layer
                .entry(node.physical_copper_layer)
                .or_default()
                .push((id, shape));
        }
    }
    let mut seen: BTreeSet<(usize, usize)> = graph
        .geometry_contacts
        .iter()
        .map(|e| {
            (
                e.first_node.min(e.second_node),
                e.first_node.max(e.second_node),
            )
        })
        .collect();
    let mut additions = Vec::new();
    let mut narrow = 0usize;
    for (layer, mut pads) in by_layer {
        pads.sort_by_key(|p| (p.1.aabb()[0], p.0));
        for i in 0..pads.len() {
            let (a_id, shape_a) = pads[i];
            let bounds = shape_a.aabb();
            for &(b_id, shape_b) in pads.iter().skip(i + 1) {
                let other = shape_b.aabb();
                if other[0] >= bounds[2] {
                    break;
                }
                if bounds[1] >= other[3] || other[1] >= bounds[3] {
                    continue;
                }
                narrow += 1;
                if narrow > MAX_NARROW_PHASE {
                    return Err("D05603_PAD_PAIR_WORK_LIMIT");
                }
                let (first, second) = (a_id.min(b_id), a_id.max(b_id));
                if !seen.contains(&(first, second)) && distinct_area(shape_a, shape_b) {
                    if graph.geometry_contacts.len() + additions.len() >= MAX_CONTACTS {
                        return Err("D05603_PAD_PAIR_EDGE_LIMIT");
                    }
                    seen.insert((first, second));
                    additions.push(GeometryContact {
                        first_node: first,
                        second_node: second,
                        layer,
                        kind: kind(
                            &graph.nodes[first].footprint,
                            &graph.nodes[second].footprint,
                        )?,
                        declared_net_check: relation(
                            graph.nodes[first].declared_net.as_ref(),
                            graph.nodes[second].declared_net.as_ref(),
                        ),
                    });
                }
            }
        }
    }
    graph.geometry_contacts.extend(additions);
    graph
        .geometry_contacts
        .sort_by_key(|e| (e.first_node, e.second_node, e.kind));
    Ok(())
}
