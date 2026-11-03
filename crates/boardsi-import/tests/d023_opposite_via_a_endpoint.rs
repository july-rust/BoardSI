#[path = "../src/d015_t_junction.rs"]
mod shared;

const SRC: &[u8] = include_bytes!("fixtures/d023_opposite_via_a_endpoint_v94.asc");
const VIA_B: &[u8] = include_bytes!("fixtures/d022_collinear_endpoint_v94.asc");
const PARTIAL: &[u8] = include_bytes!("fixtures/d021_partial_collinear_overlap_v94.asc");
const CONTAINED: &[u8] = include_bytes!("fixtures/d020_contained_collinear_overlap_v94.asc");

fn witness(src: &[u8]) -> Result<String, &'static str> {
    shared::encode_opposite_endpoint_contact(src, "NET_A", [1, 4], [0, 1])
}
fn source() -> String {
    String::from_utf8(SRC.to_vec())
        .unwrap()
        .replace("\r\n", "\n")
}

#[test]
fn exact_opposite_via_a_triangle_remains_source_geometry_only() {
    let w = witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D023_COLLINEAR_OPPOSITE_ENDPOINT_CONTACT_V1|"));
    assert!(w.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000"));
    assert!(w.contains("EDGE|4|5|7|SOURCE_COLLINEAR_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(w.contains("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(w.contains(
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|6|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(w.contains("SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(w.contains("OVERALL|COLLINEAR_OPPOSITE_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY"));
    assert!(w.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
    assert_eq!(
        w.lines().filter(|line| line.starts_with("EDGE|")).count(),
        6
    );
    assert_eq!(w, witness(SRC).unwrap());
}

#[test]
fn reverse_orientation_and_source_lexical_sha() {
    let s = source();
    let reverse = s.replace(
        "150 250 4 10 0\n50 250 4 10 0",
        "50 250 4 10 0\n150 250 4 10 0",
    );
    assert!(witness(reverse.as_bytes())
        .unwrap()
        .contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000"));
    assert_ne!(
        witness(SRC).unwrap(),
        witness(s.replace("80 250 1 10 0", "80.00 250 1 10 0").as_bytes()).unwrap()
    );
}

#[test]
fn reject_hostiles_and_cross_mode_confusion() {
    let s = source();
    for bad in [
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "150.01 250 4 10 0\n50 250 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "150 250.01 4 10 0\n50 250 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "150 250 4 10 0\n50 250.01 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "149.99 250 4 10 0\n50 250 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "150 250 4 10 0\n150 250 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "150 250 4 10 0\n200 250 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "100 250 4 10 0\n200 250 4 10 0",
        ),
        s.replace(
            "150 250 4 10 0\n50 250 4 10 0",
            "150 250 1 10 0\n50 250 1 10 0",
        ),
        s.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        s.replace("50 250 4 10 0", "6000000 250 4 10 0"),
        s.replace("50 250 4 10 0", "50 250 4 0 0"),
        s.replace("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
        s.replace("*END*", "*LINES*\nCOPCLS 5 0.01 0 4\n*END*"),
        s.replace("50 250 4 10 0", "50 250 4 10 0\n60 250 4 10 0"),
    ] {
        assert!(witness(bad.as_bytes()).is_err(), "accepted hostile {bad}");
    }
    for incompatible in [VIA_B, PARTIAL, CONTAINED] {
        assert!(witness(incompatible).is_err());
    }
    assert!(shared::encode_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_partial_overlap(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_contained_overlap(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_opposite_endpoint_contact(SRC, "OTHER", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_opposite_endpoint_contact(SRC, "NET_A", [1, 2], [0, 1]).is_err());
    assert!(shared::encode_opposite_endpoint_contact(SRC, "NET_A", [1, 4], [1, 0]).is_err());
}
