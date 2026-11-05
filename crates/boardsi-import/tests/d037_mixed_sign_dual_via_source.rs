// D_037 is SOURCE geometry only; no fabricated plated barrel or electrical proof.
use std::fmt::Write;
#[path = "../src/d015_t_junction.rs"]
#[allow(dead_code)]
mod d015_core;
#[path = "../src/d036_dual_mirrored_via.rs"]
#[allow(dead_code)]
mod d036_core;
#[path = "../src/d037_mixed_sign_dual_via.rs"]
#[allow(dead_code)]
mod d037_core;

const FIXTURE: &str = include_str!("fixtures/d037_mixed_sign_dual_via_endpoint_v94.asc");

fn qualify(src: &str) -> Result<String, &'static str> {
    d037_core::encode(src.as_bytes(), "NET_A", [1, 4], [0, 1])
}

#[test]
fn d037_source_dual_signed_negative_graph_and_sha() {
    let wire = qualify(FIXTURE).expect("exact dual Via geometry");
    let mut sha = String::new();
    for byte in boardsi_import::digest::sha256(FIXTURE.as_bytes()) {
        write!(&mut sha, "{byte:02x}").expect("SHA-256 hex");
    }
    assert!(wire.starts_with(&format!(
        "BOARDSI_D037_DUAL_MIXED_SIGN_VIA_ENDPOINT_CONTACT_V1|{sha}|SOURCE_GEOMETRY_ONLY"
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
    assert!(wire
        .contains("EDGE|4|5|7|SOURCE_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.contains("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire
        .contains("EDGE|6|5|8|SOURCE_NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.contains("EDGE|7|2|8|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"));
    assert!(wire.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
}

#[test]
fn d037_reverse_branch_and_lexical_sha_independent() {
    // Normalize CRLF checkout fixtures so reverse mutation is real on Windows.
    let normalized = FIXTURE.replace("\r\n", "\n");
    let reversed_a = normalized.replace(
        "150 250 4 10 0\n30 160 4 10 0",
        "30 160 4 10 0\n150 250 4 10 0",
    );
    let reversed_b = reversed_a.replace(
        "350 250 4 10 0\n470 340 4 10 0",
        "470 340 4 10 0\n350 250 4 10 0",
    );
    let reversed_wire = qualify(&reversed_b).expect("both branch records can reverse");
    assert_eq!(reversed_wire.matches("NODE|").count(), 9);
    assert_eq!(reversed_wire.matches("EDGE|").count(), 8);
    assert!(reversed_wire.contains("LOCAL_COMPONENTS|3"));
    assert_ne!(reversed_a, normalized);
    assert_ne!(reversed_b, reversed_a);
    assert!(d036_core::encode(FIXTURE.as_bytes(), "NET_A", [1, 4], [0, 1]).is_err());
    let lexical = normalized.replace("80 250 1 10 0", "80.00 250 1 10 0");
    assert_ne!(lexical, normalized);
    let lexical_wire = qualify(&lexical).expect("exact decimal lexical source");
    assert_ne!(
        qualify(&normalized).unwrap(),
        lexical_wire,
        "SHA must bind original Source bytes"
    );
    assert!(lexical_wire.contains("LOCAL_COMPONENTS|3"));
    assert!(
        d015_core::encode_non45_oblique_mirrored_via_a_endpoint_contact(
            FIXTURE.as_bytes(),
            "NET_A",
            [1, 4],
            [0, 1]
        )
        .is_err()
    );
    assert!(
        d015_core::encode_non45_oblique_mirrored_via_b_endpoint_contact(
            FIXTURE.as_bytes(),
            "NET_A",
            [1, 4],
            [0, 1]
        )
        .is_err()
    );
}

#[test]
fn d037_rotated_board_remains_valid_with_disjoint_layer1_arms() {
    // Exact 180-degree rotation around (250,250) mil preserves both signed
    // exterior 4:3 ratios, but main direction is reversed. No L1 false contact.
    let base = FIXTURE.replace("\r\n", "\n");
    let rotated = base
        .replace(
            "80 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n350 250 1 10 0 SIMPLEVIA\n420 250 1 10 0",
            "420 250 1 10 0\n350 250 4 10 0 SIMPLEVIA\n150 250 1 10 0 SIMPLEVIA\n80 250 1 10 0",
        )
        // Distinct placeholders prevent the first rotated branch from being
        // accidentally substituted again by the second replacement.
        .replace("150 250 4 10 0\n30 160 4 10 0", "__D037_ROT_A__")
        .replace("350 250 4 10 0\n470 340 4 10 0", "__D037_ROT_B__")
        .replace("__D037_ROT_A__", "350 250 4 10 0\n470 340 4 10 0")
        .replace("__D037_ROT_B__", "150 250 4 10 0\n30 160 4 10 0");
    assert_ne!(rotated, base);
    assert!(rotated.contains("350 250 4 10 0\n470 340 4 10 0"));
    assert!(rotated.contains("150 250 4 10 0\n30 160 4 10 0"));
    let wire = qualify(&rotated).expect("rotated disjoint L1 geometry qualifies");
    assert!(wire.contains(
        "COUNT|NODES|9|LAYER_LOCAL_EDGES|8|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"
    ));
    assert!(wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|VIA|A|X|35000|Y|25000"));
    assert!(wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|4|VIA|B|X|15000|Y|25000"));
}

#[test]
fn d037_hostile_signed_geometry_source_and_selection_fail_closed() {
    let normalized = FIXTURE.replace("\r\n", "\n");
    let source = normalized.as_str();
    let hostiles = [
        source.replace("30 160 4 10 0", "30 160.01 4 10 0"),
        source.replace("470 340 4 10 0", "470 340.01 4 10 0"),
        source.replace("30 160 4 10 0", "30 340 4 10 0"),
        source.replace("470 340 4 10 0", "470 160 4 10 0"),
        source.replace("30 160 4 10 0", "60 160 4 10 0"),
        source.replace("470 340 4 10 0", "440 340 4 10 0"),
        source.replace("150 250 4 10 0\n30 160", "150.01 250 4 10 0\n30 160"),
        source.replace("350 250 4 10 0\n470 340", "350.01 250 4 10 0\n470 340"),
        source.replace("470 340 4 10 0", "470 340 1 10 0"),
        // Codex P2: formerly missed crossing of two nonhorizontal Layer-1 arms.
        source
            .replace("80 250 1 10 0", "350 450 1 10 0")
            .replace("420 250 1 10 0", "150 450 1 10 0"),
        source.replace("*SIGNAL* NET_A\nU5.5", "*SIGNAL* OTHER\nU5.5"),
        source.replace("*END*", "*LINES*\nCOPCLS 5 0.01 0 4\n*END*"),
        source.replace("SIMPLEVIA 15 2", "SIMPLEVIA 15 2\nSIMPLEVIA 15 2"),
        source.replace(
            "350 250 4 10 0\n470 340 4 10 0",
            "150 250 4 10 0\n470 340 4 10 0",
        ),
        source.replace(
            "350 250 4 10 0\n470 340 4 10 0",
            "350 250 4 10 0\n30 160 4 10 0",
        ),
    ];
    for hostile in hostiles {
        assert_ne!(hostile, source, "hostile mutation must be effective");
        assert!(
            qualify(&hostile).is_err(),
            "hostile SOURCE unexpectedly accepted: {hostile}"
        );
    }
    assert!(d037_core::encode(FIXTURE.as_bytes(), "OTHER", [1, 4], [0, 1]).is_err());
    assert!(d037_core::encode(FIXTURE.as_bytes(), "NET_A", [1, 2], [0, 1]).is_err());
    assert!(d037_core::encode(FIXTURE.as_bytes(), "NET_A", [1, 4], [1, 0]).is_err());
    assert!(d037_core::encode(FIXTURE.as_bytes(), "NET_A", [1, 4], [0, 0]).is_err());
}
