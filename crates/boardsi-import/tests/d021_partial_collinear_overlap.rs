#[path = "../src/d015_t_junction.rs"]
mod shared;

const SRC: &[u8] = include_bytes!("fixtures/d021_partial_collinear_overlap_v94.asc");
const CONTAINED: &[u8] = include_bytes!("fixtures/d020_contained_collinear_overlap_v94.asc");
const T: &[u8] = include_bytes!("fixtures/d015_t_junction_v94.asc");
const O: &[u8] = include_bytes!("fixtures/d019_oblique_t_junction_v94.asc");

fn witness(bytes: &[u8]) -> Result<String, &'static str> {
    shared::encode_partial_overlap(bytes, "NET_A", [1, 4], [0, 1])
}

fn source() -> String {
    String::from_utf8(SRC.to_vec()).unwrap().replace("\r\n", "\n")
}

#[test]
fn exact_positive_length_partial_source_candidate_only() {
    let wire = witness(SRC).unwrap();
    assert!(wire.starts_with("BOARDSI_D021_PARTIAL_COLLINEAR_OVERLAP_V1|"));
    assert!(wire.contains("SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(wire.contains("OVERLAP|MAIN_TRACE|1|PARTIAL_TRACE|3|START|30000|25000|END|35000|25000"));
    assert!(wire.contains("EDGE|4|5|7|SOURCE_COLLINEAR_PARTIAL_OVERLAP_CANDIDATE|4"));
    assert!(wire.contains("COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"));
    assert!(wire.contains("OVERALL|COLLINEAR_PARTIAL_OVERLAP_SOURCE_CANDIDATE_ONLY"));
    assert_eq!(wire.lines().filter(|line| line.starts_with("NODE|")).count(), 8);
    assert_eq!(wire.lines().filter(|line| line.starts_with("EDGE|")).count(), 5);
    assert_eq!(wire, witness(SRC).unwrap());
}

#[test]
fn reverse_and_partial_on_opposite_end_are_direction_invariant() {
    let s = source();
    let reversed = s.replace(
        "300 250 4 10 0\n400 250 4 10 0",
        "400 250 4 10 0\n300 250 4 10 0",
    );
    assert!(witness(reversed.as_bytes()).unwrap().contains(
        "START|30000|25000|END|35000|25000"
    ));
    let opposite = s.replace("300 250 4 10 0", "100 250 4 10 0")
        .replace("400 250 4 10 0", "200 250 4 10 0");
    assert!(witness(opposite.as_bytes()).unwrap().contains(
        "START|15000|25000|END|20000|25000"
    ));
    let decimal = s.replace("80 250 1 10 0", "80.00 250 1 10 0");
    assert_ne!(witness(SRC).unwrap(), witness(decimal.as_bytes()).unwrap());
}

#[test]
fn existing_modes_are_disjoint() {
    assert!(shared::encode(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_oblique(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_contained_overlap(SRC, "NET_A", [1, 4], [0, 1]).is_err());
    assert!(witness(T).is_err());
    assert!(witness(O).is_err());
    assert!(witness(CONTAINED).is_err());
    assert!(shared::encode(T, "NET_A", [1, 4], [0, 1]).is_ok());
    assert!(shared::encode_oblique(O, "NET_A", [1, 4], [0, 1]).is_ok());
    assert!(shared::encode_contained_overlap(CONTAINED, "NET_A", [1, 4], [0, 1]).is_ok());
}

#[test]
fn hostile_source_inventory_and_geometry_fail_closed() {
    let s = source();
    for hostile in [
        s.replace("300 250 4 10 0", "300 251 4 10 0"),
        s.replace("400 250 4 10 0", "400 251 4 10 0"),
        s.replace("300 250 4 10 0", "350 250 4 10 0"),
        s.replace("300 250 4 10 0", "351 250 4 10 0"),
        s.replace("300 250 4 10 0", "400 250 4 10 0"),
        s.replace("400 250 4 10 0", "350 250 4 10 0"),
        s.replace("400 250 4 10 0", "250 250 4 10 0"),
        s.replace("300 250 4 10 0", "150 250 4 10 0")
            .replace("400 250 4 10 0", "350 250 4 10 0"),
        s.replace("300 250 4 10 0", "300 250 1 10 0"),
        s.replace("*SIGNAL* NET_A\nU3.3", "*SIGNAL* OTHER\nU3.3"),
        s.replace("300 250 4 10 0", "6000000 250 4 10 0"),
        s.replace("300 250 4 10 0", "300.001 250 4 10 0"),
        s.replace("300 250 4 10 0", "300 250 4 0 0"),
        s.replace("400 250 4 10 0", "400 250 4 10 0\n410 250 4 10 0"),
        s.replace("*END*", "*LINES*\nRA COPPER 0 0 1 1 0 OTHER\nCOPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*"),
        s.replace("*ROUTE*", "SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*"),
    ] {
        assert!(witness(hostile.as_bytes()).is_err(), "accepted hostile: {hostile}");
    }
}

#[test]
fn selection_remains_fail_closed() {
    assert!(shared::encode_partial_overlap(SRC, "OTHER", [1, 4], [0, 1]).is_err());
    assert!(shared::encode_partial_overlap(SRC, "NET_A", [1, 2], [0, 1]).is_err());
    assert!(shared::encode_partial_overlap(SRC, "NET_A", [1, 4], [1, 0]).is_err());
}
