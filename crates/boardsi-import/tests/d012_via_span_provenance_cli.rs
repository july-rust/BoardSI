#[allow(dead_code)]
#[path = "../src/bin/boardsi-d012-via-span-provenance.rs"]
mod d012;
const SOURCE: &[u8] = include_bytes!("fixtures/d010_two_trace_four_area_v94.asc");
fn witness(input: &[u8]) -> Result<String, &'static str> {
    d012::encode(input, ["RA", "RB", "RC", "RD"], "NET_A", [1, 4], 0)
}
#[test]
fn source_span_is_not_plating_or_electrical_proof() {
    let w = witness(SOURCE).unwrap();
    assert!(w.starts_with("BOARDSI_D012_VIA_SPAN_PROVENANCE_V1|"));
    assert!(
        w.contains("PLATING_IMPORTER_ASSUMED|BARREL_NOT_VERIFIED|NO_CROSS_LAYER_ELECTRICAL_EDGE")
    );
    let via = w
        .lines()
        .find(|line| line.starts_with("VIA|0|"))
        .and_then(|line| line.strip_prefix("VIA|0|"))
        .unwrap();
    let expected_span = format!("SPAN|0|{via}|1|4|PLATING_IMPORTER_ASSUMED");
    assert!(w.lines().any(|line| line == expected_span));
    assert_eq!(w.lines().filter(|s| s.starts_with("NODE|")).count(), 8);
    assert_eq!(w.lines().filter(|s| s.starts_with("EDGE|")).count(), 6);
    assert_eq!(w, witness(SOURCE).unwrap());
    assert!(w.ends_with("END|NO_ELECTRICAL_BARREL_EDGE\n"));
}
#[test]
fn changed_source_changes_sha_but_preserves_source_graph() {
    let edited = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("U1.1 U2.2", "U1.1  U2.2");
    let original = witness(SOURCE).unwrap();
    let updated = witness(edited.as_bytes()).unwrap();
    assert_ne!(updated, original);
    assert_eq!(
        updated
            .lines()
            .filter(|s| s.starts_with("EDGE|"))
            .collect::<Vec<_>>(),
        original
            .lines()
            .filter(|s| s.starts_with("EDGE|"))
            .collect::<Vec<_>>()
    );
}
#[test]
fn fail_closed_on_bad_net_span_geometry_and_ambiguous_contact() {
    assert!(d012::encode(SOURCE, ["RA", "RB", "RC", "RD"], "WRONG", [1, 4], 0).is_err());
    assert!(d012::encode(SOURCE, ["RA", "RB", "RC", "RD"], "NET_A", [1, 2], 0).is_err());
    assert!(d012::encode(SOURCE, ["RA", "RB", "RC", "RD"], "NET_A", [1, 4], 1).is_err());
    let text = String::from_utf8(SOURCE.to_vec()).unwrap();
    let boundary = text.replacen("330 280 4 10 0", "280 280 4 10 0", 1);
    assert!(witness(boundary.as_bytes()).is_err());
}
