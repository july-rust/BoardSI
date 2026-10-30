//! C_015: actual public synthetic PADS parser -> analytic mixed-hole oracle.
//! This is NOT a V11 worker receipt, electrical DRC or Qt acceptance test.
use boardsi_import::copper_preview::{self, CopperGeometry};
use boardsi_import::reference_route_v11::geometry::{
    classify, pad_inside_outer, validate_holes, PadRisk, SourceHole,
};
const PREFIX: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\n";
fn source() -> String {
    let mut s = String::from(PREFIX);
    for (name, layer) in [("RA", 2), ("RB", 3)] {
        s.push_str(&format!(
            "{name} COPPER 100 200 3 1 0 NET_REF\nCOPCLS 5 0.01 0 {layer}\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 {layer}\n5 5\n12 5\n12 12\n5 12\n5 5\nCOPCCO 2 0.01 0 {layer}\n30 50\n70 50\n"
        ));
    }
    s.push_str("*END*\n");
    s
}
fn holes(area: &copper_preview::CopperArea) -> Vec<SourceHole> {
    area.cutouts
        .iter()
        .map(|h| match h {
            CopperGeometry::Polygon(p) => {
                SourceHole::Rectangle(copper_preview::rectangle_bounds(p).unwrap())
            }
            CopperGeometry::Circle { centre, radius } => SourceHole::Circle {
                centre: *centre,
                radius: *radius,
            },
            CopperGeometry::Curved { .. } => panic!("unsupported source arc"),
        })
        .collect()
}
#[test]
fn actual_pads_source_exact_mixed_holes_on_two_layers() {
    let parsed = copper_preview::parse(source().as_bytes(), 4).unwrap();
    assert_eq!(parsed.len(), 2);
    for (i, area) in parsed.iter().enumerate() {
        assert_eq!(area.layer, [2, 3][i]);
        assert_eq!(area.net_label.as_deref(), Some("NET_REF"));
        assert_eq!(area.cutouts.len(), 2);
        let CopperGeometry::Polygon(outer_ring) = &area.geometry else {
            panic!("unexpected copper outline");
        };
        let outer = copper_preview::rectangle_bounds(outer_ring).unwrap();
        assert_eq!(outer, [10_000, 20_000, 20_000, 30_000]);
        let h = holes(area);
        assert_eq!(
            h[1],
            SourceHole::Circle {
                centre: [15_000, 25_000],
                radius: 2_000,
            }
        );
        assert!(validate_holes(&h, outer));
        assert!(pad_inside_outer([15_000, 25_000], 3_500, outer));
        assert_eq!(
            classify([15_000, 25_000], 3_500, &h).unwrap(),
            (PadRisk::GapObserved, Some(1))
        );
        assert_eq!(
            classify([15_000, 25_000], 4_000, &h).unwrap(),
            (PadRisk::TangencyRisk, Some(1))
        );
        assert_eq!(
            classify([15_000, 25_000], 4_001, &h).unwrap(),
            (PadRisk::CopperOverlapRisk, Some(1))
        );
    }
}
#[test]
fn malformed_pads_circular_ownership_or_contact_refused() {
    let original = source();
    let wrong_layer = original.replacen("COPCCO 2 0.01 0 2", "COPCCO 2 0.01 0 4", 1);
    assert!(copper_preview::parse(wrong_layer.as_bytes(), 4).is_err());
    let unowned = original.replacen("COPCCO 2 0.01 0 2", "CIRCUR 2 0.01 0 2", 1);
    assert!(copper_preview::parse(unowned.as_bytes(), 4).is_err());
    let overlapping = original.replacen("30 50\n70 50", "8 8\n28 8", 1);
    assert!(copper_preview::parse(overlapping.as_bytes(), 4).is_err());
}
