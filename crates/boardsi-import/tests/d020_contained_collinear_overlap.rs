#[path = "../src/d015_t_junction.rs"]
mod shared;
const SRC: &[u8] = include_bytes!("fixtures/d020_contained_collinear_overlap_v94.asc");
const T: &[u8] = include_bytes!("fixtures/d015_t_junction_v94.asc");
const O: &[u8] = include_bytes!("fixtures/d019_oblique_t_junction_v94.asc");

fn witness(bytes: &[u8]) -> Result<String, &'static str> {
    shared::encode_contained_overlap(bytes, "NET_A", [1, 4], [0, 1])
}
fn source() -> String {
    String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n")
}

#[test]
fn strict_containment_yields_source_candidate_only() {
    let wire = witness(SRC).unwrap();
    assert!(wire.starts_with("BOARDSI_D020_CONTAINED_COLLINEAR_OVERLAP_V1|"));
    assert!(wire.contains("SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(wire.contains("OVERLAP|MAIN_TRACE|1|CONTAINED_TRACE|3|START|200|250|END|250|250"));
    assert!(wire.contains("EDGE|4|5|7|SOURCE_COLLINEAR_CONTAINED_OVERLAP_CANDIDATE|4"));
    assert!(wire.contains("COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"));
    assert!(wire.contains("OVERALL|COLLINEAR_CONTAINED_OVERLAP_SOURCE_CANDIDATE_ONLY"));
    assert_eq!(wire.lines().filter(|line| line.starts_with("NODE|")).count(), 8);
    assert_eq!(wire.lines().filter(|line| line.starts_with("EDGE|")).count(), 5);
    assert_eq!(wire, witness(SRC).unwrap());
}
#[test]
fn reversed_containment_and_source_lexical_sha() {
    let s = source();
    let reverse = s.replace("200 250 4 10 0\n250 250 4 10 0",
                            "250 250 4 10 0\n200 250 4 10 0");
    let wire = witness(reverse.as_bytes()).unwrap();
    assert!(wire.contains("START|200|250|END|250|250"));
    let decimal = s.replace("80 250 1 10 0", "80.00 250 1 10 0");
    assert_ne!(witness(SRC).unwrap(), witness(decimal.as_bytes()).unwrap());
}
#[test]
fn old_t_x_and_oblique_modes_are_disjoint() {
    assert!(shared::encode(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(witness(T).is_err());
    assert!(witness(O).is_err());
    assert!(shared::encode(T, "NET_A", [1, 4], [0, 1]).is_ok());
    assert!(shared::encode_oblique(O, "NET_A", [1, 4], [0, 1]).is_ok());
}
#[test]
fn one_tick_miss_and_noncontained_or_ambiguous_overlap_fail_closed() {
    let s = source();
    for hostile in [
        s.replace("200 250 4 10 0", "200 251 4 10 0"),
        s.replace("250 250 4 10 0", "250 251 4 10 0"),
        s.replace("200 250 4 10 0", "150 250 4 10 0"),
        s.replace("250 250 4 10 0", "350 250 4 10 0"),
        s.replace("200 250 4 10 0", "100 250 4 10 0"),
        s.replace("200 250 4 10 0", "250 250 4 10 0"),
        s.replace("200 250 4 10 0", "150 250 4 10 0")
            .replace("250 250 4 10 0", "350 250 4 10 0"),
        s.replace("200 250 4 10 0", "200 250 1 10 0"),
        s.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        s.replace("200 250 4 10 0", "6000000 250 4 10 0"),
        s.replace("200 250 4 10 0", "200.001 250 4 10 0"),
        s.replace("200 250 4 10 0", "200 250 4 0 0"),
        s.replace("250 250 4 10 0", "250 250 4 10 0\n300 250 4 10 0"),
        s.replace("*END*", "*LINES*\nRA COPPER 0 0 1 1 0 OTHER\nCOPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*"),
        s.replace("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
    ] {
        assert!(witness(hostile.as_bytes()).is_err(), "accepted hostile: {hostile}");
    }
}
#[test]
fn selection_and_source_bindings_remain_strict() {
    assert!(shared::encode_contained_overlap(SRC, "OTHER", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_contained_overlap(SRC, "NET_A", [1, 2], [0, 1]).is_err());
    assert!(shared::encode_contained_overlap(SRC, "NET_A", [1, 4], [1, 0]).is_err());
}
