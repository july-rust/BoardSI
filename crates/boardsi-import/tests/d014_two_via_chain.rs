#[path = "../src/d014_two_via_chain.rs"]
mod d014;
const SRC: &[u8] = include_bytes!("fixtures/d014_two_via_chain_v94.asc");
fn witness(s: &[u8]) -> Result<String, &'static str> {
    d014::encode(s, "NET_A", [1, 4], [0, 1])
}
#[test]
fn source_two_via_chain_without_electrical_promotion() {
    let w = witness(SRC).unwrap();
    assert!(w.starts_with("BOARDSI_D014_TWO_VIA_CHAIN_V1|"));
    assert!(w.contains("PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE"));
    assert_eq!(w.lines().filter(|r| r.starts_with("VIA|")).count(), 2);
    assert_eq!(w.lines().filter(|r| r.starts_with("TRACE|")).count(), 3);
    assert_eq!(w.lines().filter(|r| r.starts_with("NODE|")).count(), 7);
    assert_eq!(w.lines().filter(|r| r.starts_with("EDGE|")).count(), 4);
    assert!(w.contains("LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0"));
    assert!(w.ends_with("END|FAIL_CLOSED_NO_BARREL_PROOF\n"));
    assert_eq!(w, witness(SRC).unwrap());
}
#[test]
fn sha_and_selection_fail_closed() {
    let source = String::from_utf8(SRC.to_vec()).unwrap();
    let edited = source.replace("U1.1 U2.2", "U1.1  U2.2");
    assert_ne!(witness(SRC).unwrap(), witness(edited.as_bytes()).unwrap());
    assert!(d014::encode(SRC, "OTHER", [1, 4], [0, 1]).is_err());
    assert!(d014::encode(SRC, "NET_A", [1, 2], [0, 1]).is_err());
    assert!(d014::encode(SRC, "NET_A", [1, 4], [1, 0]).is_err());
    assert!(d014::encode(SRC, "NET_A", [1, 4], [0, 2]).is_err());
}
#[test]
fn missing_extra_and_mismatched_contacts_fail_closed() {
    let source = String::from_utf8(SRC.to_vec())
        .unwrap()
        .replace("\r\n", "\n");
    assert!(witness(
        source
            .replace("350 250 1 10 0 SIMPLEVIA", "350 250 1 10 0")
            .as_bytes()
    )
    .is_err());
    assert!(witness(
        source
            .replace(
                "350 250 1 10 0 SIMPLEVIA",
                "350 250 1 10 0 SIMPLEVIA\n360 250 4 10 0 SIMPLEVIA"
            )
            .as_bytes()
    )
    .is_err());
    assert!(witness(
        source
            .replace("350 250 1 10 0 SIMPLEVIA", "350 250 2 10 0 SIMPLEVIA")
            .as_bytes()
    )
    .is_err());
    assert!(witness(source.replace("0 35 R", "0 10 R").as_bytes()).is_err());
    assert!(witness(
        source
            .replace("SIMPLEVIA 15 2", "SIMPLEVIA 15 3")
            .as_bytes()
    )
    .is_err());
    assert!(witness(
        source
            .replace("*SIGNAL* NET_A", "*SIGNAL* OTHER")
            .as_bytes()
    )
    .is_err());
    assert!(witness(
        source
            .replace("150 250 4 10 0 SIMPLEVIA", "150 250 4 10 0 UNKNOWN")
            .as_bytes()
    )
    .is_err());
}
