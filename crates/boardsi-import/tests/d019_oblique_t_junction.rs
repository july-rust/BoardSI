#[path = "../src/d015_t_junction.rs"]
mod shared;
const SRC: &[u8] = include_bytes!("fixtures/d019_oblique_t_junction_v94.asc");
const RIGHT_ANGLE: &[u8] = include_bytes!("fixtures/d015_t_junction_v94.asc");

fn witness(bytes: &[u8]) -> Result<String, &'static str> {
    shared::encode_oblique(bytes, "NET_A", [1, 4], [0, 1])
}
fn source() -> String {
    String::from_utf8(SRC.to_vec())
        .unwrap()
        .replace("\r\n", "\n")
}
#[test]
fn strict_oblique_source_t_has_candidates_but_no_electrical_promotion() {
    let w = witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D019_OBLIQUE_T_JUNCTION_V1|"));
    assert!(w.contains("SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(w.contains("EDGE|4|5|7|SOURCE_INTERIOR_OBLIQUE_T_CONTACT_CANDIDATE|4"));
    assert!(w.contains(
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(w.contains("OVERALL|OBLIQUE_T_JUNCTION_SOURCE_CANDIDATE_ONLY"));
    assert_eq!(w.lines().filter(|x| x.starts_with("NODE|")).count(), 8);
    assert_eq!(w.lines().filter(|x| x.starts_with("EDGE|")).count(), 5);
    assert_eq!(w, witness(SRC).unwrap());
}
#[test]
fn perpendicular_and_oblique_modes_remain_disjoint() {
    assert!(shared::encode(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(witness(RIGHT_ANGLE).is_err());
    assert!(shared::encode(RIGHT_ANGLE, "NET_A", [1, 4], [0, 1]).is_ok());
    let s = source();
    for free in ["220 350 4 10 0", "280 350 4 10 0", "180 320 4 10 0"] {
        assert!(witness(s.replace("200 350 4 10 0", free).as_bytes()).is_ok());
    }
}
#[test]
fn gap_overrun_endpoint_collinear_perpendicular_wrong_layer_and_net_fail() {
    let s = source();
    let hostile = [
        s.replace("250 250 4 10 0", "250 251 4 10 0"),
        s.replace("250 250 4 10 0", "250 150 4 10 0"),
        s.replace("250 250 4 10 0", "150 250 4 10 0"),
        s.replace("200 350 4 10 0", "250 350 4 10 0"),
        s.replace("200 350 4 10 0", "200 250 4 10 0"),
        s.replace("200 350 4 10 0", "200 350 1 10 0"),
        s.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        s.replace("200 350 4 10 0", "200 350 4 0 0"),
        s.replace("250 250 4 10 0", "250 250 4 10 0\n300 200 4 10 0"),
    ];
    for sample in hostile {
        assert!(
            witness(sample.as_bytes()).is_err(),
            "accepted hostile {sample}"
        );
    }
}
#[test]
fn ambiguous_inventory_and_invalid_source_lexemes_fail() {
    let s = source();
    let hostile = [
        s.replace("*END*", "*LINES*\nRA COPPER 0 0 1 1 0 OTHER\nCOPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*"),
        s.replace("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
        s.replace("*END*", "*PART*\n*END*"),
        s.replace("200 350 4 10 0", "200.001 350 4 10 0"),
        s.replace("200 350 4 10 0", "6000000 350 4 10 0"),
        s.replace("200 350 4 10 0", "NaN 350 4 10 0"),
        s.replace("250 250 4 10 0", "250 250 4 10 0 SIMPLEVIA"),
    ];
    for sample in hostile {
        assert!(
            witness(sample.as_bytes()).is_err(),
            "accepted source {sample}"
        );
    }
}
#[test]
fn selection_and_source_sha_are_bound() {
    let s = source();
    let decimal = s.replace("80 250 1 10 0", "80.00 250 1 10 0");
    assert_ne!(witness(SRC).unwrap(), witness(decimal.as_bytes()).unwrap());
    assert!(shared::encode_oblique(SRC, "OTHER", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 2], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 4], [1, 0]).is_err());
}
