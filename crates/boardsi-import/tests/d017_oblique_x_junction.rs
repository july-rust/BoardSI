#[path = "../src/d016_x_junction.rs"]
mod shared;
const SRC: &[u8] = include_bytes!("fixtures/d017_oblique_x_junction_v94.asc");
const RIGHT_ANGLE: &[u8] = include_bytes!("fixtures/d016_x_junction_v94.asc");
fn witness(bytes: &[u8]) -> Result<String, &'static str> {
    shared::encode_oblique(bytes, "NET_A", [1, 4], [0, 1])
}
fn source() -> String {
    String::from_utf8(SRC.to_vec()).unwrap()
}
#[test]
fn strict_oblique_interior_x_is_only_same_layer_source_candidate() {
    let w = witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D017_OBLIQUE_X_JUNCTION_V1|"));
    assert!(w.contains("SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(w.contains("EDGE|4|5|7|SOURCE_INTERIOR_OBLIQUE_X_CONTACT_CANDIDATE|4"));
    assert!(w.contains(
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(w.contains("OVERALL|OBLIQUE_X_JUNCTION_SOURCE_CANDIDATE_ONLY"));
    assert_eq!(w.lines().filter(|l| l.starts_with("NODE|")).count(), 8);
    assert_eq!(w.lines().filter(|l| l.starts_with("EDGE|")).count(), 5);
    assert_eq!(w, witness(SRC).unwrap());
}
#[test]
fn complementary_angle_classification_preserves_d016() {
    assert!(shared::encode(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(witness(RIGHT_ANGLE).is_err());
    assert!(shared::encode(RIGHT_ANGLE, "NET_A", [1, 4], [0, 1]).is_ok());
    for (a, b) in [
        ("210 350", "290 150"),
        ("240 350", "320 150"),
        ("220 350", "280 150"),
    ] {
        let s = source().replace("220 350", a).replace("300 150", b);
        assert!(
            witness(s.as_bytes()).is_ok(),
            "oblique segment rejected: {a} -> {b}"
        );
    }
}
#[test]
fn no_endpoint_t_parallel_collinear_near_miss_or_wrong_net_layer() {
    let s = source();
    let bad = [
        s.replace("300 150 4 10 0", "300 251 4 10 0"),
        s.replace("300 150 4 10 0", "250 250 4 10 0"),
        s.replace("220 350 4 10 0", "120 350 4 10 0")
            .replace("300 150 4 10 0", "180 150 4 10 0"),
        s.replace("220 350 4 10 0", "200 250 4 10 0")
            .replace("300 150 4 10 0", "300 250 4 10 0"),
        s.replace("220 350 4 10 0", "200 350 4 10 0")
            .replace("300 150 4 10 0", "300 350 4 10 0"),
        s.replace("220 350 4 10 0", "220 350 1 10 0"),
        s.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        s.replace("220 350 4 10 0", "260 350 4 10 0")
            .replace("300 150 4 10 0", "260 150 4 10 0"),
        s.replace("220 350 4 10 0", "220 350 4 0 0"),
    ];
    for b in bad {
        assert!(witness(b.as_bytes()).is_err(), "accepted source: {b}");
    }
}
#[test]
fn source_inventory_and_coordinate_limits_fail_closed() {
    let s = source();
    let bad=[
        s.replace("*END*","*LINES*\nRA COPPER 0 0 1 1 0 OTHER\nCOPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*"),
        s.replace("*ROUTE*","SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
        s.replace("300 150 4 10 0","300 150 4 10 0\n330 120 4 10 0"),
        s.replace("*END*","*PART*\n*END*"),
        s.replace("220 350 4 10 0","220.001 350 4 10 0"),
        s.replace("220 350 4 10 0","9999999999999999 350 4 10 0"),
        s.replace("220 350 4 10 0","6000000 350 4 10 0"),
        s.replace("220 350 4 10 0","NaN 350 4 10 0"),
        s.replace("300 150 4 10 0","300 150 4 10 0 SIMPLEVIA"),
    ];
    for b in bad {
        assert!(
            witness(b.as_bytes()).is_err(),
            "accepted unsupported source: {b}"
        );
    }
}
#[test]
fn sha_selection_and_decimal_source_lexemes_are_bound() {
    let s = source();
    let decimal = s.replace("80 250 1 10 0", "80.00 250 1 10 0");
    assert_ne!(witness(SRC).unwrap(), witness(decimal.as_bytes()).unwrap());
    assert!(shared::encode_oblique(SRC, "OTHER", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 2], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 4], [1, 0]).is_err());
}
