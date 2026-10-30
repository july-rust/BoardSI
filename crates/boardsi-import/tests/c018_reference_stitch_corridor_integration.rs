//! C_018 bounded V5–V8 receipt integration on public PADS V9.4 MILS.
//! Source-geometry risk only, never real electrical return current or SI.
use boardsi_import::{reference_route_v5 as v5, reference_route_v6 as v6, reference_route_v7 as v7, reference_route_v8 as v8};

const HEAD: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const COPPER: &str = "*LINES*\nRA COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nRB COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 3\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const ROUTE: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n";
const REF: &str = "*SIGNAL* NET_REF\nR1.1 R2.2\n140 280 2 10 0\n180 280 3 10 0 SIMPLEVIA\n190 280 3 10 0\n";

fn board(copper: &str, route: &str, reference: &str) -> Vec<u8> {
    format!("{HEAD}{copper}{route}{reference}*END*\n").into_bytes()
}
fn all(bytes: &[u8]) -> [String; 4] {
    [
        v5::encode(bytes, &[0, 1], 1, ["NET_REF"; 2], [2, 3], 1).unwrap(),
        v6::encode(bytes, &[0, 1], 1, ["NET_REF"; 2], [2, 3], 1).unwrap(),
        v7::encode(bytes, &[0, 1], 1, ["NET_REF"; 2], [2, 3], 1).unwrap(),
        v8::encode(bytes, &[0, 1], 1, ["NET_REF"; 2], [2, 3], 1).unwrap(),
    ]
}
fn field(wire: &str, n: usize) -> &str {
    wire.lines().next().unwrap().split('|').nth(n).unwrap()
}
fn safe(wire: &str) {
    assert!(wire.contains("|UNSUPPORTED|NOT_RUN|0|"));
    assert!(wire.ends_with("SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
    assert!(!wire.contains("PHYSICAL_PASS"));
}

#[test]
fn original_sha_and_exact_nested_receipts_are_preserved() {
    let r = all(&board(COPPER, ROUTE, REF));
    for (i, w) in r.iter().enumerate() {
        assert_eq!(field(w, 0), format!("BOARDSI_REFERENCE_ROUTE_V{}", i + 5));
        assert_eq!(field(w, 1), field(&r[0], 1));
        safe(w);
    }
    assert!(r[1].contains(&r[0]));
    assert!(r[2].contains(&r[1]));
    assert!(r[3].contains(&r[2]));
    assert_eq!(field(&r[1], 2), "SAME_SOURCE_AREA_CORRIDOR_CANDIDATE");
    assert_eq!(field(&r[2], 2), "PAD_FOOTPRINT_CONTAINED_CANDIDATE");
    assert_eq!(field(&r[3], 2), "SOURCE_PAD_SEPARATE_OBSERVED");
}

#[test]
fn same_net_disconnected_area_cannot_qualify_v8() {
    let copper = COPPER.replace(
        "RA COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n",
        "RA COPPER 170 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n30 0\n30 100\n0 100\n0 0\nRC COPPER 110 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n50 0\n50 100\n0 100\n0 0\n",
    );
    let r = all(&board(&copper, ROUTE, REF));
    assert_eq!(field(&r[0], 2), "VERIFIED_SOURCE_STITCH_CANDIDATE");
    assert_eq!(field(&r[1], 2), "REFERENCE_AREA_MISMATCH");
    assert_eq!(field(&r[2], 2), "V6_CORRIDOR_RISK");
    assert_eq!(field(&r[3], 2), "V7_REFERENCE_FOOTPRINT_RISK");
    assert!(r[3].contains(&r[2]));
    safe(&r[3]);
}

#[test]
fn pad_outer_boundary_and_copper_cutout_remain_risk() {
    let boundary = COPPER.replacen("100 0\n100 100", "97.5 0\n97.5 100", 1);
    let r = all(&board(&boundary, ROUTE, REF));
    assert_eq!(field(&r[2], 2), "PAD_BOUNDARY_RISK");
    assert_eq!(field(&r[3], 2), "V7_REFERENCE_FOOTPRINT_RISK");
    let hole = COPPER
        .replacen("RA COPPER 100 200 1", "RA COPPER 100 200 2", 1)
        .replacen("RB COPPER", "COPCUT 5 0.01 0 2\n65 65\n75 65\n75 95\n65 95\n65 65\nRB COPPER", 1);
    let r = all(&board(&hole, ROUTE, REF));
    assert_eq!(field(&r[2], 2), "PAD_CUTOUT_RISK");
    assert_eq!(field(&r[3], 2), "V7_REFERENCE_FOOTPRINT_RISK");
    safe(&r[3]);
}

#[test]
fn tangent_overlap_and_one_tick_separation_remain_distinct() {
    for (point, expected) in [
        ("171 278", "SOURCE_PAD_TANGENT_RISK"),
        ("171 277", "SOURCE_PAD_OVERLAP_RISK"),
        ("171.01 278", "SOURCE_PAD_SEPARATE_OBSERVED"),
    ] {
        let reference = REF.replace("180 280 3 10 0 SIMPLEVIA", &format!("{point} 3 10 0 SIMPLEVIA"));
        let r = all(&board(COPPER, ROUTE, &reference));
        assert_eq!(field(&r[2], 2), "PAD_FOOTPRINT_CONTAINED_CANDIDATE");
        assert_eq!(field(&r[3], 2), expected);
        assert!(r[3].contains(&r[2]));
        safe(&r[3]);
    }
}

#[test]
fn source_change_and_invalid_selection_fail_closed() {
    let original = board(COPPER, ROUTE, REF);
    let revised = board(COPPER, &ROUTE.replace("130 250", "131 250"), REF);
    let a = all(&original);
    let b = all(&revised);
    for i in 0..4 {
        assert_ne!(field(&a[i], 1), field(&b[i], 1));
        safe(&b[i]);
    }
    assert!(b[3].contains(&b[2]));
    assert!(!b[3].contains(&a[2]));
    for (index, layers, nets) in [
        (0, [2, 3], ["NET_REF", "NET_REF"]),
        (99, [2, 3], ["NET_REF", "NET_REF"]),
        (1, [2, 4], ["NET_REF", "NET_REF"]),
        (1, [2, 3], ["NET_REF", "NET_OTHER"]),
    ] {
        assert!(v8::encode(&original, &[0, 1], 1, nets, layers, index).is_err());
    }
}
