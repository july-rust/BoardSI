#[path = "../src/d015_t_junction.rs"]
mod shared;

const SRC: &[u8] = include_bytes!("fixtures/d031_non45_steep_via_a_endpoint_v94.asc");
const OBLIQUE_45_B: &[u8] = include_bytes!("fixtures/d026_oblique_via_b_endpoint_v94.asc");
const OBLIQUE_45_A: &[u8] = include_bytes!("fixtures/d027_oblique_via_a_endpoint_v94.asc");
const ORTHOGONAL_B: &[u8] = include_bytes!("fixtures/d024_orthogonal_via_b_endpoint_v94.asc");
const ORTHOGONAL_A: &[u8] = include_bytes!("fixtures/d025_orthogonal_via_a_endpoint_v94.asc");
const COLLINEAR: &[u8] = include_bytes!("fixtures/d022_collinear_endpoint_v94.asc");
const PARTIAL: &[u8] = include_bytes!("fixtures/d021_partial_collinear_overlap_v94.asc");
const STEEP_D030: &[u8] = include_bytes!("fixtures/d030_non45_steep_via_b_endpoint_v94.asc");
const SHALLOW_D028: &[u8] = include_bytes!("fixtures/d028_non45_oblique_via_b_endpoint_v94.asc");
const SHALLOW_D029: &[u8] = include_bytes!("fixtures/d029_non45_oblique_via_a_endpoint_v94.asc");

fn witness(src: &[u8]) -> Result<String, &'static str> {
    shared::encode_non45_steep_via_a_endpoint_contact(src, "NET_A", [1, 4], [0, 1])
}
fn source() -> String {
    String::from_utf8(SRC.to_vec())
        .unwrap()
        .replace("\r\n", "\n")
}

#[test]
fn exact_oblique_3_to_4_via_a_source_only() {
    let w = witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D031_NON45_STEEP_VIA_A_ENDPOINT_CONTACT_V1|"));
    assert!(w.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000"));
    assert!(w.contains("EDGE|4|5|7|SOURCE_NON45_STEEP_VIA_A_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(w.contains("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(w.contains(
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|6|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(w.contains("SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(w.contains("OVERALL|NON45_STEEP_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY"));
    assert!(w.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
    assert_eq!(
        w.lines().filter(|line| line.starts_with("EDGE|")).count(),
        6
    );
    assert_eq!(w, witness(SRC).unwrap());
}

#[test]
fn reverse_branch_and_source_lexical_sha_bound() {
    let s = source();
    let reversed = s.replace(
        "150 250 4 10 0\n60 370 4 10 0",
        "60 370 4 10 0\n150 250 4 10 0",
    );
    assert!(witness(reversed.as_bytes())
        .unwrap()
        .contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000"));
    assert_ne!(
        witness(SRC).unwrap(),
        witness(s.replace("80 250 1 10 0", "80.00 250 1 10 0").as_bytes()).unwrap()
    );
}

#[test]
fn reject_1tick_angle_gap_wrong_mode_and_source_forgery() {
    let s = source();
    assert!(s.contains("60 370 4 10 0"));
    assert!(!s.contains("450 350 4 10 0"));
    for bad in [
        s.replace(
            "150 250 4 10 0\n60 370 4 10 0",
            "150.01 250 4 10 0\n60 370 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n60 370 4 10 0",
            "150 250.01 4 10 0\n60 370 4 10 0",
        ),
        s.replace("60 370 4 10 0", "60.01 370 4 10 0"),
        s.replace("60 370 4 10 0", "60 370.01 4 10 0"),
        s.replace("60 370 4 10 0", "30 250 4 10 0"),
        s.replace("60 370 4 10 0", "60 340 4 10 0"), // 45-degree
        s.replace("60 370 4 10 0", "30 340 4 10 0"), // shallow 4:3
        s.replace("60 370 4 10 0", "150 340 4 10 0"),
        s.replace("60 370 4 10 0", "350 250 4 10 0"),
        s.replace("60 370 4 10 0", "230 340 4 10 0"),
        s.replace(
            "150 250 4 10 0\n60 370 4 10 0",
            "150 300 4 10 0\n60 370 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n60 370 4 10 0",
            "150 250 1 10 0\n60 370 1 10 0",
        ),
        s.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        s.replace("60 370 4 10 0", "6000000 350 4 10 0"),
        s.replace("60 370 4 10 0", "60 370 4 0 0"),
        s.replace("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
        s.replace("*END*", "*LINES*\nCOPCLS 5 0.01 0 4\n*END*"),
        s.replace("60 370 4 10 0", "60 370 4 10 0\n570 430 4 10 0"),
    ] {
        assert_ne!(bad, s, "hostile mutation was a no-op");
        assert!(witness(bad.as_bytes()).is_err(), "accepted hostile {bad}");
    }
    for incompatible in [
        ORTHOGONAL_B,
        ORTHOGONAL_A,
        COLLINEAR,
        PARTIAL,
        OBLIQUE_45_B,
        OBLIQUE_45_A,
        STEEP_D030,
        SHALLOW_D028,
        SHALLOW_D029,
    ] {
        assert!(witness(incompatible).is_err());
    }
    assert!(shared::encode_orthogonal_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(
        shared::encode_orthogonal_via_a_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err()
    );
    assert!(shared::encode_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_opposite_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique_via_b_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique_via_a_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(
        shared::encode_non45_oblique_via_b_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err()
    );
    assert!(
        shared::encode_non45_oblique_via_a_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err()
    );
    assert!(
        shared::encode_non45_steep_via_b_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err()
    );
    assert!(witness(SRC).is_ok());
    assert!(
        shared::encode_non45_steep_via_a_endpoint_contact(SRC, "OTHER", [1, 4], [0, 1]).is_err()
    );
    assert!(
        shared::encode_non45_steep_via_a_endpoint_contact(SRC, "NET_A", [1, 2], [0, 1]).is_err()
    );
    assert!(
        shared::encode_non45_steep_via_a_endpoint_contact(SRC, "NET_A", [1, 4], [1, 0]).is_err()
    );
}
