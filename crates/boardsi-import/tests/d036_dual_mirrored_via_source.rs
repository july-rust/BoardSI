// D_036 is SOURCE geometry only; no fabricated plated barrel or electrical proof.
#[path = "../src/d015_t_junction.rs"]
#[allow(dead_code)]
mod d015_core;
#[path = "../src/d036_dual_mirrored_via.rs"]
#[allow(dead_code)]
mod d036_core;

const FIXTURE: &str = include_str!("fixtures/d036_dual_mirrored_via_endpoint_v94.asc");

fn qualify(src: &str) -> Result<String, &'static str> {
    d036_core::encode(src.as_bytes(), "NET_A", [1, 4], [0, 1])
}

#[test]
fn d036_source_dual_signed_negative_graph_and_sha() {
    let wire = qualify(FIXTURE).expect("exact dual Via geometry");
    let sha: String = boardsi_import::digest::sha256(FIXTURE.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert!(wire.starts_with(&format!(
        "BOARDSI_D036_DUAL_MIRRORED_VIA_ENDPOINT_CONTACT_V1|{sha}|SOURCE_GEOMETRY_ONLY"
    )));
    assert!(wire.contains("BARREL_ELECTRICAL_NOT_VERIFIED"));
    assert!(wire.contains("CROSS_LAYER_ELECTRICAL_NOT_PROVEN"));
    assert!(wire.contains("SOLVER_READY_FALSE"));
    assert_eq!(wire.matches("NODE|").count(), 9);
    assert_eq!(wire.matches("EDGE|").count(), 8);
    assert!(wire.contains(
        "COUNT|NODES|9|LAYER_LOCAL_EDGES|8|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|VIA|A|X|15000|Y|25000"));
    assert!(wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|4|VIA|B|X|35000|Y|25000"));
    assert!(wire.contains("EDGE|4|5|7|SOURCE_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.contains("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.contains("EDGE|6|5|8|SOURCE_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.contains("EDGE|7|2|8|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
}

#[test]
fn d036_reverse_branch_and_lexical_sha_independent() {
    let reversed_a = FIXTURE.replace(
        "150 250 4 10 0\n30 160 4 10 0",
        "30 160 4 10 0\n150 250 4 10 0",
    );
    let reversed_b = reversed_a.replace(
        "350 250 4 10 0\n470 160 4 10 0",
        "470 160 4 10 0\n350 250 4 10 0",
    );
    let reversed_wire = qualify(&reversed_b).expect("both branch records can reverse");
    assert_eq!(reversed_wire.matches("NODE|").count(), 9);
    assert_eq!(reversed_wire.matches("EDGE|").count(), 8);
    assert!(reversed_wire.contains("LOCAL_COMPONENTS|3"));
    let lexical = FIXTURE.replace("80 250 1 10 0", "80.00 250 1 10 0");
    let lexical_wire = qualify(&lexical).expect("exact decimal lexical source");
    assert_ne!(qualify(FIXTURE).unwrap(), lexical_wire, "SHA must bind original Source bytes");
    assert!(lexical_wire.contains("LOCAL_COMPONENTS|3"));
    assert!(d015_core::encode_non45_oblique_mirrored_via_a_endpoint_contact(
        FIXTURE.as_bytes(), "NET_A", [1, 4], [0, 1]
    ).is_err());
    assert!(d015_core::encode_non45_oblique_mirrored_via_b_endpoint_contact(
        FIXTURE.as_bytes(), "NET_A", [1, 4], [0, 1]
    ).is_err());
}

#[test]
fn d036_hostile_signed_geometry_source_and_selection_fail_closed() {
    let hostiles = [
        FIXTURE.replace("30 160 4 10 0", "30 160.01 4 10 0"),
        FIXTURE.replace("470 160 4 10 0", "470 160.01 4 10 0"),
        FIXTURE.replace("30 160 4 10 0", "30 340 4 10 0"),
        FIXTURE.replace("470 160 4 10 0", "470 340 4 10 0"),
        FIXTURE.replace("30 160 4 10 0", "60 160 4 10 0"),
        FIXTURE.replace("470 160 4 10 0", "440 160 4 10 0"),
        FIXTURE.replace("150 250 4 10 0\n30 160", "150.01 250 4 10 0\n30 160"),
        FIXTURE.replace("350 250 4 10 0\n470 160", "350.01 250 4 10 0\n470 160"),
        FIXTURE.replace("470 160 4 10 0", "470 160 1 10 0"),
        FIXTURE.replace("*SIGNAL* NET_A\nU5.5", "*SIGNAL* OTHER\nU5.5"),
        FIXTURE.replace("*END*", "*LINES*\nCOPCLS 5 0.01 0 4\n*END*"),
        FIXTURE.replace("SIMPLEVIA 15 2", "SIMPLEVIA 15 2\nSIMPLEVIA 15 2"),
        FIXTURE.replace("350 250 4 10 0\n470 160 4 10 0", "150 250 4 10 0\n470 160 4 10 0"),
        FIXTURE.replace("350 250 4 10 0\n470 160 4 10 0", "350 250 4 10 0\n30 160 4 10 0"),
    ];
    for hostile in hostiles {
        assert_ne!(hostile, FIXTURE, "hostile mutation must be effective");
        assert!(qualify(&hostile).is_err(), "hostile SOURCE unexpectedly accepted: {hostile}");
    }
    assert!(d036_core::encode(FIXTURE.as_bytes(), "OTHER", [1, 4], [0, 1]).is_err());
    assert!(d036_core::encode(FIXTURE.as_bytes(), "NET_A", [1, 2], [0, 1]).is_err());
    assert!(d036_core::encode(FIXTURE.as_bytes(), "NET_A", [1, 4], [1, 0]).is_err());
    assert!(d036_core::encode(FIXTURE.as_bytes(), "NET_A", [1, 4], [0, 0]).is_err());
}
