#[path = "../src/d016_x_junction.rs"]
mod d016;
const SRC: &[u8] = include_bytes!("fixtures/d016_x_junction_v94.asc");

fn witness(data: &[u8]) -> Result<String, &'static str> {
    d016::encode(data, "NET_A", [1, 4], [0, 1])
}
fn source() -> String {
    String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n")
}
#[test]
fn strict_interior_x_contact_is_only_a_layer_local_candidate() {
    let w=witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D016_X_JUNCTION_V1|"));
    assert!(w.contains("PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert_eq!(w.lines().filter(|s|s.starts_with("VIA|")).count(),2);
    assert_eq!(w.lines().filter(|s|s.starts_with("TRACE|")).count(),4);
    assert_eq!(w.lines().filter(|s|s.starts_with("NODE|")).count(),8);
    assert_eq!(w.lines().filter(|s|s.starts_with("EDGE|")).count(),5);
    assert!(w.contains("EDGE|4|5|7|SOURCE_INTERIOR_X_CONTACT_CANDIDATE|4"));
    assert!(w.contains("COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"));
    assert!(w.contains("OVERALL|X_JUNCTION_SOURCE_CANDIDATE_ONLY"));
    assert_eq!(w,witness(SRC).unwrap());
}
#[test]
fn source_sha_binding_and_selected_vias_fail_closed() {
    let s=source();
    let modified=s.replace("U3.3 U4.4","U3.3  U4.4");
    assert_ne!(witness(SRC).unwrap(),witness(modified.as_bytes()).unwrap());
    assert!(d016::encode(SRC,"OTHER",[1,4],[0,1]).is_err());
    assert!(d016::encode(SRC,"NET_A",[1,2],[0,1]).is_err());
    assert!(d016::encode(SRC,"NET_A",[1,4],[1,0]).is_err());
}
#[test]
fn tangency_endpoint_and_legacy_t_junction_are_not_x_junctions() {
    let s=source();
    for invalid in [
        // D015 endpoint/interior, not D016 interior/interior.
        s.replace("250 150 4 10 0","250 250 4 10 0"),
        // Branch misses the main line by one exact source tick.
        s.replace("250 150 4 10 0","250 251 4 10 0"),
        // Crosses only the main endpoint rather than strict interior.
        s.replace("250 350 4 10 0","150 350 4 10 0")
         .replace("250 150 4 10 0","150 150 4 10 0"),
        // Main interior collinear overlap is not an X.
        s.replace("250 350 4 10 0","200 250 4 10 0")
         .replace("250 150 4 10 0","300 250 4 10 0"),
        // Touching an endpoint of the branch is not X.
        s.replace("250 350 4 10 0","250 250 4 10 0"),
        // Wrong layer (or cross-net) must never become an edge.
        s.replace("250 350 4 10 0","250 350 1 10 0"),
        s.replace("*SIGNAL* NET_A\nU3.3","*SIGNAL* OTHER\nU3.3"),
        // Off-perpendicular crossing is outside bounded D016 geometry.
        s.replace("250 350 4 10 0","260 350 4 10 0"),
    ] {
        assert!(witness(invalid.as_bytes()).is_err(),"accepted hostile source: {invalid}");
    }
}
#[test]
fn extra_conductors_and_unsupported_sections_are_not_silently_dropped() {
    let s=source();
    for invalid in [
        s.replace("*END*", "*LINES*\nRA COPPER 0 0 1 1 0 OTHER\nCOPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*"),
        s.replace("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
        s.replace("-2 35 R\n0 35 R", "-2 35 R\n-2 35 R"),
        s.replace("SIMPLEVIA 15 2", "SIMPLEVIA 15 3"),
        s.replace("250 150 4 10 0","250 150 4 10 0\n270 270 4 10 0"),
        s.replace("*END*","*TEXT*\n*END*"),
        s.replace("250 350 4 10 0","250 350 2 10 0"),
        s.replace("250 350 4 10 0","250 350 4 0 0"),
        s.replace("250 150 4 10 0","250 150 4 10 0 SIMPLEVIA"),
        s.replace("250 150 4 10 0","250 150 4 10 0\n*SIGNAL* NET_A\nU5.5 U6.6"),
    ] {
        assert!(witness(invalid.as_bytes()).is_err(),"accepted unmodeled source: {invalid}");
    }
}
#[test]
fn source_grid_decimal_equivalence_is_distinct_sha() {
    let s=source();
    let decimal=s.replace("80 250 1 10 0","80.00 250 1 10 0");
    let w=witness(decimal.as_bytes()).unwrap();
    assert!(w.contains("SOURCE_INTERIOR_X_CONTACT_CANDIDATE"));
    assert_ne!(w,witness(SRC).unwrap());
    for bad in [
        s.replace("80 250 1 10 0","80.001 250 1 10 0"),
        s.replace("80 250 1 10 0","NaN 250 1 10 0"),
        s.replace("80 250 1 10 0","999999999999999999999 250 1 10 0"),
    ] {
        assert!(witness(bad.as_bytes()).is_err());
    }
}
