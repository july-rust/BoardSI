#[path = "../src/d015_t_junction.rs"]
mod shared;

const SRC: &[u8] = include_bytes!("fixtures/d029_non45_oblique_via_a_endpoint_v94.asc");
const D028: &[u8] = include_bytes!("fixtures/d028_non45_oblique_via_b_endpoint_v94.asc");

fn witness(bytes: &[u8]) -> Result<String, &'static str> {
    shared::encode_non45_oblique_via_a_endpoint_contact(bytes, "NET_A", [1, 4], [0, 1])
}

#[test]
fn exact_four_to_three_via_a_geometry_only() {
    let wire = witness(SRC).unwrap();
    assert!(wire.starts_with("BOARDSI_D029_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_V1|"));
    assert!(wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000"));
    assert!(wire.contains("EDGE|4|5|7|SOURCE_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.contains("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.contains(
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|6|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(wire.contains(
        "SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"
    ));
    assert!(wire.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
    assert_eq!(wire.lines().filter(|line| line.starts_with("EDGE|")).count(), 6);
    assert_eq!(wire, witness(SRC).unwrap());
}

#[test]
fn reversed_branch_preserves_contact_and_sha_binding() {
    let s = String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n");
    let reversed = s.replace(
        "150 250 4 10 0\n30 340 4 10 0",
        "30 340 4 10 0\n150 250 4 10 0",
    );
    assert_ne!(s, reversed);
    assert!(witness(reversed.as_bytes()).unwrap().contains(
        "CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000"
    ));
    let lexical = s.replace("80 250 1 10 0", "80.00 250 1 10 0");
    assert_ne!(witness(SRC).unwrap(), witness(lexical.as_bytes()).unwrap());
}

#[test]
fn reject_source_gaps_angles_modes_and_extra_artwork() {
    let s = String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n");
    let pair = "150 250 4 10 0\n30 340 4 10 0";
    for bad in [
        s.replace(pair, "150.01 250 4 10 0\n30 340 4 10 0"),
        s.replace(pair, "150 250.01 4 10 0\n30 340 4 10 0"),
        s.replace("30 340 4 10 0", "30.01 340 4 10 0"),
        s.replace("30 340 4 10 0", "30 340.01 4 10 0"),
        s.replace("30 340 4 10 0", "30 250 4 10 0"),
        s.replace("30 340 4 10 0", "150 340 4 10 0"),
        s.replace("30 340 4 10 0", "150 250 4 10 0"),
        s.replace("30 340 4 10 0", "270 340 4 10 0"),
        s.replace(pair, "150 300 4 10 0\n30 340 4 10 0"),
        s.replace(pair, "150 250 1 10 0\n30 340 1 10 0"),
        s.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        s.replace("*END*", "*LINES*\nCOPCLS 5 0.01 0 4\n*END*"),
        s.replace("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
        s.replace("30 340 4 10 0", "30 340 4 10 0\n40 430 4 10 0"),
    ] {
        assert_ne!(bad, s);
        assert!(witness(bad.as_bytes()).is_err());
    }
    assert!(witness(D028).is_err());
    assert!(
        shared::encode_non45_oblique_via_b_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err()
    );
    assert!(shared::encode_oblique_via_a_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(
        shared::encode_orthogonal_via_a_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err()
    );
    assert!(shared::encode_opposite_endpoint_contact(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(witness(SRC).is_ok());
    assert!(
        shared::encode_non45_oblique_via_a_endpoint_contact(SRC, "OTHER", [1, 4], [0, 1]).is_err()
    );
    assert!(
        shared::encode_non45_oblique_via_a_endpoint_contact(SRC, "NET_A", [1, 2], [0, 1]).is_err()
    );
    assert!(
        shared::encode_non45_oblique_via_a_endpoint_contact(SRC, "NET_A", [1, 4], [1, 0]).is_err()
    );
}
