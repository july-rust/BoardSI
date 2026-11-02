#[allow(dead_code)]
#[path = "../src/bin/boardsi-d010-two-trace-four-area.rs"]
mod d010;
const SOURCE: &[u8] = include_bytes!("fixtures/d010_two_trace_four_area_v94.asc");
fn swap(from: &str, to: &str) -> Vec<u8> {
    let source = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("\r\n", "\n");
    assert!(source.contains(from), "missing fixture anchor: {from}");
    source.replacen(from, to, 1).into_bytes()
}
fn classify(bytes: &[u8]) -> Result<String, &'static str> {
    d010::encode(bytes, ["RA", "RB", "RC", "RD"], "NET_A", [1, 4], 0)
}
#[test]
fn two_layer_two_trace_four_area_source_candidate() {
    let wire = classify(SOURCE).unwrap();
    assert!(
        wire.contains("OVERALL|SOURCE_DUAL_LAYER_TWO_TRACE_FOUR_AREA_CANDIDATE"),
        "{wire}"
    );
    assert_eq!(wire.matches("CONTACT|").count(), 4);
    assert_eq!(wire.matches("TRACE|").count(), 2);
    assert_eq!(wire.matches("LINK|").count(), 2);
    assert!(
        wire.contains("LINK|0|SOURCE_VIA_TRACE_TARGET_AREA_CANDIDATE|1|0|0|2|1"),
        "{wire}"
    );
    assert!(
        wire.contains("LINK|1|SOURCE_VIA_TRACE_TARGET_AREA_CANDIDATE|0|1|0|3|1"),
        "{wire}"
    );
    assert!(wire.contains("PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0"));
}
#[test]
fn wrong_net_layers_pad_and_route_fail_closed() {
    assert!(d010::encode(SOURCE, ["RA", "RB", "RC", "RD"], "OTHER", [1, 4], 0).is_err());
    assert!(d010::encode(SOURCE, ["RA", "RB", "RC", "RD"], "NET_A", [4, 1], 0).is_err());
    assert!(d010::encode(SOURCE, ["RA", "RB", "RC", "RD"], "NET_A", [1, 2], 0).is_err());
    assert!(d010::encode(SOURCE, ["RA", "RA", "RC", "RD"], "NET_A", [1, 4], 0).is_err());
    assert!(classify(&swap("0 35 R", "-1 35 R")).is_err());
    assert!(classify(&swap("330 280 4 10 0", "330 280 1 10 0")).is_err());
    assert!(classify(&swap(
        "150 250 1 10 0 SIMPLEVIA\n330 280 4 10 0",
        "150 250 1 10 0 SIMPLEVIA"
    ))
    .is_err());
}
#[test]
fn each_destination_boundary_is_not_candidate() {
    for (from, to) in [
        ("330 280 1 10 0", "280 280 1 10 0"),
        ("330 280 4 10 0", "280 280 4 10 0"),
    ] {
        let wire = classify(&swap(from, to)).unwrap();
        assert!(
            wire.contains("SOURCE_DUAL_LAYER_TWO_TRACE_NOT_VERIFIED"),
            "{wire}"
        );
    }
}
#[test]
fn cutout_on_first_destination_blocks_qualified_path() {
    let raw = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("\r\n", "\n");
    let changed=raw.replacen("RC COPPER 0 0 1 1 0 NET_A",
                             "RC COPPER 0 0 2 1 0 NET_A",1)
        .replacen("RD COPPER 0 0 1 1 0 NET_A",
                  "COPCUT 5 0.01 0 1\n320 270\n340 270\n340 290\n320 290\n320 270\nRD COPPER 0 0 1 1 0 NET_A",1);
    let wire = classify(changed.as_bytes()).unwrap();
    assert!(wire.contains("LINK|0|SOURCE_COPCUT_RISK"), "{wire}");
    assert!(wire.contains("SOURCE_DUAL_LAYER_TWO_TRACE_NOT_VERIFIED"));
}
#[test]
fn via_area_boundary_is_not_candidate() {
    let changed = swap(
        "100 200\n200 200\n200 300\n100 300\n100 200",
        "135 200\n235 200\n235 300\n135 300\n135 200",
    );
    let wire = classify(&changed).unwrap();
    assert!(wire.contains("SOURCE_DUAL_LAYER_TWO_TRACE_NOT_VERIFIED"));
}
#[test]
fn overflow_is_rejected_without_panicking() {
    let changed = swap("330 280 4 10 0", "9223372036854775807 280 4 10 0");
    assert!(
        !classify(&changed).is_ok_and(
            |wire| wire.contains("OVERALL|SOURCE_DUAL_LAYER_TWO_TRACE_FOUR_AREA_CANDIDATE")
        )
    );
}
