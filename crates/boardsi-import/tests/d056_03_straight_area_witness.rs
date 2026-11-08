//! D_056_03 independent non-quarter SOURCE-area positive-witness oracle.
//! Explicitly test that an arbitrary straight centreline crossing a narrow
//! area is found without fabricated positive contact inside holes/tangents.
use boardsi_import::copper_preview::{CopperArea, CopperGeometry};
use boardsi_import::layered_source_graph::{from_pads_v94_source, ContactKind};
use boardsi_import::straight_area_witness::positive_straight_area_witness;
use boardsi_import::trace_contact::{Centreline, Stroke};

fn line(a: [i64; 2], b: [i64; 2]) -> Stroke {
    Stroke {
        layer: 1,
        net: "SIGNAL0".into(),
        width_ticks: 2,
        source_line: 12,
        centreline: Centreline::Straight { start: a, end: b },
    }
}
fn area(geometry: CopperGeometry, cutouts: Vec<CopperGeometry>) -> CopperArea {
    CopperArea {
        name: "SOURCE_COP".into(),
        net_label: Some("GND".into()),
        layer: 1,
        source_line: 9,
        geometry,
        cutouts,
    }
}
fn square(x0: i64, y0: i64, x1: i64, y1: i64) -> CopperGeometry {
    CopperGeometry::Polygon(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]])
}
#[test]
fn narrow_midspan_rectangle_contact_not_at_any_quarter_sample() {
    let region = area(square(43, 50, 47, 70), vec![]);
    assert!(positive_straight_area_witness(&line([0, 60], [100, 60]), &region).unwrap());
}
#[test]
fn arbitrary_oblique_angle_positive_copper_crossing() {
    let region = area(square(43, 43, 47, 47), vec![]);
    assert!(positive_straight_area_witness(&line([0, 0], [100, 100]), &region).unwrap());
}
#[test]
fn circle_cutout_inside_polygon_prevents_false_contact() {
    let region = area(
        square(0, 0, 100, 100),
        vec![CopperGeometry::Circle {
            centre: [50, 50],
            radius: 30,
        }],
    );
    assert!(!positive_straight_area_witness(&line([43, 50], [57, 50]), &region).unwrap());
}
#[test]
fn circle_midspan_contact_and_tangent_remain_distinct() {
    let region = area(
        CopperGeometry::Circle {
            centre: [50, 50],
            radius: 10,
        },
        vec![],
    );
    assert!(positive_straight_area_witness(&line([0, 50], [100, 50]), &region).unwrap());
    assert!(!positive_straight_area_witness(&line([0, 60], [100, 60]), &region).unwrap());
}
#[test]
fn no_chord_interpolation_for_circular_trace() {
    let region = area(square(43, 40, 47, 60), vec![]);
    let circular = Stroke {
        layer: 1,
        net: "SIGNAL0".into(),
        width_ticks: 2,
        source_line: 17,
        centreline: Centreline::Circular {
            centre: [50, 50],
            start: [0, 50],
            end: [100, 50],
            clockwise: true,
        },
    };
    assert!(!positive_straight_area_witness(&circular, &region).unwrap());
}
#[test]
fn stale_layer_or_zero_width_is_not_positive() {
    let region = area(square(43, 50, 47, 70), vec![]);
    let mut s = line([0, 60], [100, 60]);
    s.layer = 2;
    assert!(!positive_straight_area_witness(&s, &region).unwrap());
    s.layer = 1;
    s.width_ticks = 0;
    assert!(!positive_straight_area_witness(&s, &region).unwrap());
}
#[test]
fn original_pads_source_narrow_copper_crossing_enters_candidate_graph() {
    let original =
        String::from_utf8(include_bytes!("fixtures/pads_copper_filled_v94.asc").to_vec())
            .unwrap()
            .replace("\r\n", "\n");
    let before = "COP_A COPPER 40 40 1 1 0 GND\nCOPCLS 7 0.01 0 1\n0 0\n100 0\n100 40\n60 40\n60 80\n0 80\n0 0";
    let after = "COP_A COPPER 43 50 1 1 0 GND\nCOPCLS 5 0.01 0 1\n0 0\n4 0\n4 20\n0 20\n0 0";
    let source = original
        .replace(before, after)
        .replace("10 20 1 10 0\n100 20 1 10 0", "0 60 1 2 0\n100 60 1 2 0");
    assert_ne!(source, original);
    assert!(source.contains(after));
    let graph = from_pads_v94_source(source.as_bytes()).unwrap();
    let edges: Vec<_> = graph
        .geometry_contacts
        .iter()
        .filter(|e| e.kind == ContactKind::TraceCopperArea)
        .collect();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].layer, 1);
    assert!(graph.verified_electrical_edges.is_empty());
}
