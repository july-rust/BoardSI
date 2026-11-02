#[allow(dead_code)]
#[path = "../src/bin/boardsi-d008-dual-layer-via-area.rs"]
mod d008;
const SOURCE: &[u8] = include_bytes!("fixtures/d008_dual_layer_via_area_v94.asc");
fn swap(from: &str, to: &str) -> Vec<u8> {
    // Windows checkout can use CRLF for .asc; normalize only for mutated fixtures.
    let normalized = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("\r\n", "\n");
    assert!(normalized.contains(from), "fixture mutation anchor missing");
    normalized.replacen(from, to, 1).into_bytes()
}
#[test]
fn dual_layer_source_only_candidate() {
    let wire = d008::encode(SOURCE, ["RA", "RB"], "NET_A", [1, 4], 0).unwrap();
    assert!(wire.contains("OVERALL|SOURCE_DUAL_LAYER_VIA_PAD_AREA_CANDIDATE"));
    assert_eq!(wire.matches("CONTACT|").count(), 2);
    assert!(wire.contains("P009_UNSUPPORTED|SI_NOT_RUN|0|0"));
}
#[test]
fn reject_bad_selection_or_layer_net() {
    assert!(d008::encode(SOURCE, ["RA", "RB"], "NET_A", [4, 1], 0).is_err());
    assert!(d008::encode(SOURCE, ["RA", "RB"], "BAD", [1, 4], 0).is_err());
    assert!(d008::encode(SOURCE, ["RA", "RB"], "NET_A", [1, 2], 0).is_err());
    let wrong = swap("RB COPPER 0 0 1 1 0 NET_A", "RB COPPER 0 0 1 1 0 OTHER");
    assert!(d008::encode(&wrong, ["RA", "RB"], "NET_A", [1, 4], 0).is_err());
}
#[test]
fn reject_missing_conductive_bottom_layer() {
    let missing = swap("0 35 R", "-1 35 R");
    assert!(d008::encode(&missing, ["RA", "RB"], "NET_A", [1, 4], 0).is_err());
}
#[test]
fn boundary_is_not_candidate() {
    let boundary = swap(
        "100 200\n200 200\n200 300\n100 300\n100 200",
        "135 200\n235 200\n235 300\n135 300\n135 200",
    );
    let wire = d008::encode(&boundary, ["RA", "RB"], "NET_A", [1, 4], 0).unwrap();
    assert!(
        wire.contains("OVERALL|SOURCE_DUAL_LAYER_NOT_VERIFIED"),
        "boundary witness: {wire}"
    );
}

#[test]
fn owned_bottom_copcut_blocks_pad_contact() {
    let source = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("\r\n", "\n");
    let source = source.replacen(
        "RB COPPER 0 0 1 1 0 NET_A",
        "RB COPPER 0 0 2 1 0 NET_A",
        1,
    );
    let source = source.replacen(
        "*ROUTE*\n",
        "COPCUT 5 0.01 0 4\n140 240\n160 240\n160 260\n140 260\n140 240\n*ROUTE*\n",
        1,
    );
    let wire = d008::encode(source.as_bytes(), ["RA", "RB"], "NET_A", [1, 4], 0).unwrap();
    assert!(wire.contains("CONTACT|1|SOURCE_COPCUT_RISK"));
    assert!(wire.contains("OVERALL|SOURCE_DUAL_LAYER_NOT_VERIFIED"));
}
