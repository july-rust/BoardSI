//! D_056_03 strict half-tick rectangular SMD-to-COPPER original geometry
//! witnesses, including a crossing with NO centre or vertex containment.
use boardsi_import::copper_preview::{CopperArea, CopperGeometry};
use boardsi_import::layered_area_contacts::positive_rectangular_smd_area_witness;

fn region(outline: CopperGeometry, cutouts: Vec<CopperGeometry>) -> CopperArea {
    CopperArea {
        name: "COP_SOURCE".into(),
        net_label: Some("GND".into()),
        layer: 3,
        source_line: 42,
        geometry: outline,
        cutouts,
    }
}
fn rectangle(x0: i64, y0: i64, x1: i64, y1: i64) -> CopperGeometry {
    CopperGeometry::Polygon(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]])
}
#[test]
fn s_m_d_rectangle_and_source_polygon_cross_without_centre_or_corner_inside() {
    // Long thin source vertical bar x40..60; wide SMD horizontal bar
    // x-40..60, y40..60. SMD centre x10 is outside source copper;
    // only the actual finite-width pad intersects the source-filled region.
    let area = region(rectangle(40, 0, 60, 100), vec![]);
    assert!(positive_rectangular_smd_area_witness(&area, [10, 50], 100, 20).unwrap());
}
#[test]
fn odd_size_pad_half_tick_sliver_is_positive_source_contact() {
    let area = region(rectangle(100, 0, 120, 100), vec![]);
    // Actual pad x=-0.5..100.5, positive overlap x=100..100.5.
    assert!(positive_rectangular_smd_area_witness(&area, [50, 50], 101, 21).unwrap());
}
#[test]
fn exact_boundary_tangency_must_not_be_promoted_to_short() {
    let area = region(rectangle(100, 0, 120, 100), vec![]);
    // Right edge exactly x100, only zero-area boundary intersection.
    assert!(!positive_rectangular_smd_area_witness(&area, [50, 50], 100, 20).unwrap());
}
#[test]
fn rectangular_pad_entirely_in_source_owned_hole_is_isolated() {
    let area = region(rectangle(0, 0, 100, 100), vec![rectangle(40, 40, 60, 60)]);
    assert!(!positive_rectangular_smd_area_witness(&area, [50, 50], 10, 10).unwrap());
}
#[test]
fn circle_inside_rectangle_with_different_centres_is_detected() {
    let area = region(
        CopperGeometry::Circle {
            centre: [50, 50],
            radius: 10,
        },
        vec![],
    );
    // The circular region x40..60 is within a wide pad x-60..60.
    assert!(positive_rectangular_smd_area_witness(&area, [0, 50], 120, 40).unwrap());
}
#[test]
fn pad_input_zero_size_fails_closed() {
    let area = region(rectangle(0, 0, 100, 100), vec![]);
    assert_eq!(
        positive_rectangular_smd_area_witness(&area, [50, 50], 0, 20).unwrap_err(),
        "D05603_INVALID_SMD_GEOMETRY"
    );
}
