#[allow(dead_code)]
#[path = "../src/bin/boardsi-d013-legacy-barrel-provenance.rs"]
mod d013;
const SOURCE: &[u8] = include_bytes!("fixtures/d010_two_trace_four_area_v94.asc");
fn witness(bytes: &[u8]) -> Result<String, &'static str> {
    d013::encode(bytes, ["RA", "RB", "RC", "RD"], "NET_A", [1, 4], 0)
}
#[test]
fn one_legacy_compatibility_link_is_not_electrical_proof() {
    let wire = witness(SOURCE).unwrap();
    assert!(wire.starts_with("BOARDSI_D013_LEGACY_BARREL_PROVENANCE_V1|"));
    assert!(wire.contains("|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert!(wire.contains("LEGACY|0|VIA:0:L1|VIA:0:L4|V1_COMPATIBILITY_PROJECTION|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED\n"));
    assert!(wire.contains(
        "COUNT|LEGACY_COMPAT_BARRELS|1|SOURCE_QUALIFIED_LOCAL_CONTACTS_NOT_ELECTRICAL\n"
    ));
    assert_eq!(wire.lines().filter(|s| s.starts_with("NODE|")).count(), 8);
    assert_eq!(wire.lines().filter(|s| s.starts_with("EDGE|")).count(), 6);
    assert!(wire.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
    assert_eq!(wire, witness(SOURCE).unwrap());
    assert!(wire.len() <= 72_000);
}
#[test]
fn source_sha_changes_without_promoting_legacy_link() {
    let edited = String::from_utf8(SOURCE.to_vec())
        .unwrap()
        .replace("U1.1 U2.2", "U1.1  U2.2");
    let original = witness(SOURCE).unwrap();
    let updated = witness(edited.as_bytes()).unwrap();
    assert_ne!(original, updated);
    assert!(updated.contains("BARREL_ELECTRICAL_NOT_VERIFIED"));
}
#[test]
fn fail_closed_wrong_net_layer_via_and_contact_geometry() {
    assert!(d013::encode(SOURCE, ["RA", "RB", "RC", "RD"], "WRONG", [1, 4], 0).is_err());
    assert!(d013::encode(SOURCE, ["RA", "RB", "RC", "RD"], "NET_A", [1, 2], 0).is_err());
    assert!(d013::encode(SOURCE, ["RA", "RB", "RC", "RD"], "NET_A", [1, 4], 1).is_err());
    let text = String::from_utf8(SOURCE.to_vec()).unwrap();
    let boundary = text.replacen("330 280 4 10 0", "280 280 4 10 0", 1);
    assert!(witness(boundary.as_bytes()).is_err());
}

#[test]
fn pad_containment_without_exact_via_centre_rejects_legacy_projection() {
    let source = String::from_utf8(SOURCE.to_vec()).unwrap();
    let off_centre = source.replace(
        "330 280 1 10 0\n150 250 4 10 0 SIMPLEVIA\n330 280 4 10 0",
        "330 280 1 10 0\n151 250 1 10 0\n*SIGNAL* NET_A\nU1.1 U2.2\n330 280 4 10 0\n150 250 4 10 0 SIMPLEVIA",
    );
    assert_ne!(source, off_centre);
    assert!(d013::d012::encode(
        off_centre.as_bytes(),
        ["RA", "RB", "RC", "RD"],
        "NET_A",
        [1, 4],
        0
    )
    .is_ok());
    assert!(witness(off_centre.as_bytes()).is_err());
}
