//! A_030: public fixture and exact importer/Canonical/V14 transport regressions.
use boardsi_import::{copper_preview, route_preview};
use std::process::Command;

const POSITIVE: &[u8] = include_bytes!("fixtures/pads_curved_copper_polygon_cutout_v94.asc");

#[test]
fn a030_source_exact_arc_and_cutout_survive_canonical_without_fake_net() {
    let copper =
        copper_preview::parse(POSITIVE, 4).expect("source-owned hole inside curved copper");
    assert_eq!(copper.len(), 1);
    assert_eq!(copper[0].cutouts.len(), 1);
    match &copper[0].geometry {
        copper_preview::CopperGeometry::Curved {
            segments,
            display_ring,
        } => {
            assert_eq!(segments.len(), 8);
            assert_eq!(display_ring.len(), 37);
        }
        _ => panic!("analytical arcs must remain source-owned"),
    }
    let parsed = route_preview::parse(POSITIVE).unwrap();
    let mut board = route_preview::into_canonical(POSITIVE, &parsed).unwrap();
    copper_preview::augment(&mut board, &copper).unwrap();
    assert_eq!(board.areas.len(), 1);
    assert_eq!(board.areas[0].cutouts.len(), 1);
    assert_eq!(board.areas[0].cutouts[0].boundary.segments.len(), 4);
    assert_eq!(board.areas[0].outer.segments.len(), 8);
    assert!(board.areas[0].net.is_none());
    assert!(!board.validate().has_errors());
}
#[test]
fn a030_source_owned_hole_is_not_inferred_from_geometry() {
    let src = std::str::from_utf8(POSITIVE).unwrap();
    let bad = [
        src.replace("COP_ARC COPPER 0 0 2 1", "COP_ARC COPPER 0 0 1 1"),
        src.replace("COPCUT 5 0.01 0 2", "COPCUT 5 0.01 0 3"),
        src.replace("150 110", "0 110"),
        src.replace("150 110", "-100 110"),
        src.replace("250 110", "400 110"),
        src.replace("250 190", "190 120"), // nonconvex or degenerate hole
        src.replace("COPCUT 5 0.01 0 2", "COPCCO 5 0.01 0 2"),
        src.replace(
            "*END*",
            "COPCUT 5 0.01 0 2\n150 110\n250 110\n250 190\n150 190\n150 110\n*END*",
        ),
    ];
    for (i, v) in bad.iter().enumerate() {
        assert_ne!(v.as_bytes(), POSITIVE);
        assert!(
            copper_preview::parse(v.as_bytes(), 4).is_err(),
            "accepted mutated fixture {i}"
        );
    }
}
#[test]
fn a030_cli_v14_carries_analytic_r_and_source_owned_h() {
    let executable = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_curved_copper_polygon_cutout_v94.asc");
    let output = Command::new(executable)
        .arg(fixture)
        .output()
        .expect("importer");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let wire = String::from_utf8(output.stdout).expect("ASCII wire");
    assert!(wire.starts_with("BOARDSI_P2U_PARTIAL_V14|V9.4-MILS|4\n"));
    let g = wire
        .lines()
        .find(|v| v.starts_with("G|R|"))
        .expect("source arc G");
    assert!(g.contains("COP_ARC"));
    assert!(g.contains("|A|"));
    let h: Vec<_> = wire.lines().filter(|v| v.starts_with("H|")).collect();
    assert_eq!(h.len(), 1);
    assert!(h[0].starts_with("H|0|5|38100|27940|63500|27940|"));
    let end = wire
        .lines()
        .find(|v| v.starts_with("END|"))
        .expect("V14 footer");
    let parts: Vec<_> = end.split('|').collect();
    assert_eq!(parts.len(), 15);
    assert_eq!(parts[11], "1"); // curved count
    assert_eq!(parts[12], "1"); // source-owned hole count
    assert_eq!(parts[13], "CANONICAL_PARTIAL");
    assert_eq!(parts[14], "SI_NOT_RUN");
}
