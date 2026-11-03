#[path = "../src/d015_t_junction.rs"]
mod d015;
const SRC: &[u8] = include_bytes!("fixtures/d015_t_junction_v94.asc");

fn witness(data: &[u8]) -> Result<String, &'static str> {
    d015::encode(data, "NET_A", [1, 4], [0, 1])
}
#[test]
fn strict_same_layer_branch_without_barrel_promotion() {
    let w = witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D015_T_JUNCTION_V1|"));
    assert!(w.contains("PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert_eq!(w.lines().filter(|s| s.starts_with("VIA|")).count(), 2);
    assert_eq!(w.lines().filter(|s| s.starts_with("TRACE|")).count(), 4);
    assert_eq!(w.lines().filter(|s| s.starts_with("NODE|")).count(), 8);
    assert_eq!(w.lines().filter(|s| s.starts_with("EDGE|")).count(), 5);
    assert!(w.contains("EDGE|4|5|7|SOURCE_INTERIOR_T_CONTACT_CANDIDATE|4"));
    assert!(w.contains(
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert_eq!(w, witness(SRC).unwrap());
}
#[test]
fn stale_source_identity_and_wrong_selection_fail_closed() {
    let source = String::from_utf8(SRC.to_vec()).unwrap();
    let whitespace = source.replace("U3.3 U4.4", "U3.3  U4.4");
    assert_ne!(
        witness(SRC).unwrap(),
        witness(whitespace.as_bytes()).unwrap()
    );
    assert!(d015::encode(SRC, "OTHER", [1, 4], [0, 1]).is_err());
    assert!(d015::encode(SRC, "NET_A", [1, 2], [0, 1]).is_err());
    assert!(d015::encode(SRC, "NET_A", [1, 4], [1, 0]).is_err());
}
#[test]
fn same_layer_perpendicular_interior_only() {
    let text = String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n");
    // A one-source-tick miss, an overrun crossing, a main endpoint,
    // a parallel contact and an off-layer branch cannot qualify.
    for invalid in [
        text.replace("250 250 4 10 0", "250 251 4 10 0"),
        text.replace("250 250 4 10 0", "250 150 4 10 0"),
        text.replace("250 250 4 10 0", "150 250 4 10 0"),
        text.replace("250 350 4 10 0", "250 350 1 10 0"),
        text.replace("250 350 4 10 0", "200 250 4 10 0"),
        text.replace("250 350 4 10 0", "150 250 4 10 0"),
    ] {
        assert!(
            witness(invalid.as_bytes()).is_err(),
            "accepted hostile fixture: {invalid}"
        );
    }
}
#[test]
fn extra_ambiguous_and_unmodeled_source_fail_closed() {
    let text = String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n");
    for invalid in [
        text.replace("SIMPLEVIA 15 2", "SIMPLEVIA 15 3"),
        text.replace("0 35 R", "0 10 R"),
        text.replace("250 350 4 10 0", "250 350 2 10 0"),
        text.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        text.replace("420 250 1 10 0", "350 250 1 10 0"),
        text.replace("250 250 4 10 0", "250 250 4 10 0\n270 270 4 10 0"),
        text.replace("250 350 4 10 0", "150 250 4 10 0 SIMPLEVIA"),
    ] {
        assert!(
            witness(invalid.as_bytes()).is_err(),
            "accepted hostile fixture: {invalid}"
        );
    }
}
 
#[test]
fn independent_source_sections_and_duplicate_via_definition_fail_closed() {
    let source = String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n");
    let copper = source.replace("*END*",
        "*LINES*\nRA COPPER 0 0 1 1 0 OTHER\nCOPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*");
    assert!(witness(copper.as_bytes()).is_err());
    let duplicate = source.replace("*ROUTE*",
        "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*");
    assert!(witness(duplicate.as_bytes()).is_err());
    let unknown = source.replace("*END*", "*TEXT*\n*END*");
    assert!(witness(unknown.as_bytes()).is_err());
    let duplicate_pad = source.replace("-2 35 R\n0 35 R", "-2 35 R\n-2 35 R");
    assert!(witness(duplicate_pad.as_bytes()).is_err());
}
 
#[test]
fn decimal_mils_source_is_equivalent_geometry_but_distinct_sha() {
    let source = String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n");
    let decimal = source.replace("80 250 1 10 0", "80.00 250 1 10 0");
    let receipt = witness(decimal.as_bytes()).unwrap();
    assert!(receipt.contains("COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3"));
    assert_ne!(receipt, witness(source.as_bytes()).unwrap());
    let unsupported = source.replace("80 250 1 10 0", "80.001 250 1 10 0");
    assert!(witness(unsupported.as_bytes()).is_err());
}
