#[path = "../src/d015_t_junction.rs"]
mod shared;

const SRC: &[u8] = include_bytes!("fixtures/d034_non45_oblique_mirrored_via_b_endpoint_v94.asc");
const POSITIVE_D028: &[u8] = include_bytes!("fixtures/d028_non45_oblique_via_b_endpoint_v94.asc");
const NEGATIVE_STEEP_D032: &[u8] =
    include_bytes!("fixtures/d032_non45_steep_mirrored_via_b_endpoint_v94.asc");
const POSITIVE_STEEP_D030: &[u8] =
    include_bytes!("fixtures/d030_non45_steep_via_b_endpoint_v94.asc");
const VIA_A_D033: &[u8] =
    include_bytes!("fixtures/d033_non45_steep_mirrored_via_a_endpoint_v94.asc");
const VIA_A_D029: &[u8] = include_bytes!("fixtures/d029_non45_oblique_via_a_endpoint_v94.asc");
const OBLIQUE_45: &[u8] = include_bytes!("fixtures/d026_oblique_via_b_endpoint_v94.asc");
const ORTHOGONAL: &[u8] = include_bytes!("fixtures/d024_orthogonal_via_b_endpoint_v94.asc");
const COLLINEAR: &[u8] = include_bytes!("fixtures/d022_collinear_endpoint_v94.asc");

fn witness(bytes: &[u8]) -> Result<String, &'static str> {
    shared::encode_non45_oblique_mirrored_via_b_endpoint_contact(bytes, "NET_A", [1, 4], [0, 1])
}
fn source() -> String {
    String::from_utf8(SRC.to_vec())
        .unwrap()
        .replace("\r\n", "\n")
}
#[test]
fn negative_shallow_via_b_source_only_and_exact_candidate_graph() {
    let w = witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D034_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_V1|"));
    assert!(w.contains("SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(w.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|35000|Y|25000"));
    assert!(
        w.contains("EDGE|4|5|7|SOURCE_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE|4")
    );
    assert!(w.contains("EDGE|5|2|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(w.contains(
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|6|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(
        w.contains("OVERALL|NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY")
    );
    assert!(w.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
    assert_eq!(w.lines().filter(|s| s.starts_with("EDGE|")).count(), 6);
    assert_eq!(w, witness(SRC).unwrap());
}
#[test]
fn branch_reversal_and_source_sha_lexical_binding() {
    let s = source();
    let reverse = s.replace(
        "350 250 4 10 0\n470 160 4 10 0",
        "470 160 4 10 0\n350 250 4 10 0",
    );
    assert_ne!(reverse, s);
    assert!(witness(reverse.as_bytes())
        .unwrap()
        .contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|35000|Y|25000"));
    let lexical = s.replace("80 250 1 10 0", "80.00 250 1 10 0");
    assert_ne!(lexical, s);
    assert_ne!(witness(lexical.as_bytes()).unwrap(), witness(SRC).unwrap());
}
#[test]
fn hostile_source_fails_closed_and_legacy_modes_are_isolated() {
    let s = source();
    let variants = [
        ("470 340 4 10 0", "positive signed D028"),
        ("440 130 4 10 0", "negative steep D032"),
        ("440 370 4 10 0", "positive steep D030"),
        ("440 160 4 10 0", "negative 45 degree"),
        ("350 340 4 10 0", "orthogonal"),
        ("470 250 4 10 0", "collinear"),
        ("470.01 160 4 10 0", "1 tick X drift"),
        ("470 160.01 4 10 0", "1 tick Y drift"),
        ("470 160 1 10 0", "wrong branch layer"),
        ("350 250 4 10 0", "degenerate"),
        ("300 340 4 10 0", "inward"),
        ("6000000 160 4 10 0", "coordinates out of range"),
        ("470 160 4 0 0", "zero width"),
    ];
    for (to, name) in variants {
        let bad = s.replace("470 160 4 10 0", to);
        assert_ne!(bad, s, "mutation no-op: {name}");
        assert!(
            witness(bad.as_bytes()).is_err(),
            "unexpected acceptance: {name}"
        );
    }
    for (original, changed) in [
        (
            "350 250 4 10 0\n470 160 4 10 0",
            "350.01 250 4 10 0\n470 160 4 10 0",
        ),
        (
            "350 250 4 10 0\n470 160 4 10 0",
            "350 250.01 4 10 0\n470 160 4 10 0",
        ),
        (
            "350 250 4 10 0\n470 160 4 10 0",
            "350 250 1 10 0\n470 160 1 10 0",
        ),
        ("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        ("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
        ("*END*", "*LINES*\nCOPCLS 5 0.01 0 4\n*END*"),
        ("470 160 4 10 0", "470 160 4 10 0\n570 430 4 10 0"),
    ] {
        let bad = s.replace(original, changed);
        assert_ne!(bad, s, "mutation no-op");
        assert!(witness(bad.as_bytes()).is_err(), "accepted hostile");
    }
    for source in [
        POSITIVE_D028,
        NEGATIVE_STEEP_D032,
        POSITIVE_STEEP_D030,
        VIA_A_D033,
        VIA_A_D029,
        OBLIQUE_45,
        ORTHOGONAL,
        COLLINEAR,
    ] {
        assert!(witness(source).is_err());
    }
    assert!(
        shared::encode_non45_oblique_via_b_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err()
    );
    assert!(shared::encode_non45_oblique_via_b_endpoint_contact(
        POSITIVE_D028,
        "NET_A",
        [1, 4],
        [0, 1]
    )
    .is_ok());
    assert!(shared::encode_non45_steep_mirrored_via_b_endpoint_contact(
        SRC,
        "NET_A",
        [1, 4],
        [0, 1]
    )
    .is_err());
    assert!(shared::encode_oblique_via_b_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_orthogonal_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(
        shared::encode_non45_oblique_mirrored_via_b_endpoint_contact(SRC, "OTHER", [1, 4], [0, 1])
            .is_err()
    );
    assert!(
        shared::encode_non45_oblique_mirrored_via_b_endpoint_contact(SRC, "NET_A", [1, 2], [0, 1])
            .is_err()
    );
    assert!(
        shared::encode_non45_oblique_mirrored_via_b_endpoint_contact(SRC, "NET_A", [1, 4], [1, 0])
            .is_err()
    );
}
