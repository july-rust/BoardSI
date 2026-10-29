//! C_009 V5: public synthetic PADS source-contact regression. NEVER P009.
use boardsi_import::reference_route_v5::encode;

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const COPPER: &str =
    "*LINES*\nRA COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nRB COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 3\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const SIGNAL: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n";
const REFERENCE: &str = "*SIGNAL* NET_REF\nR1.1 R2.2\n30 50 2 10 0\n50 50 3 10 0 SIMPLEVIA\n70 50 3 10 0\n";
fn source(copper: &str, signal: &str, reference: &str) -> Vec<u8> {
    format!("{HEADER}{VIA}{copper}{signal}{reference}*END*\n").into_bytes()
}
fn audit(bytes: &[u8], reference_via: usize) -> Result<String, &'static str> {
    encode(bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], reference_via)
}
#[test]
fn accepted_candidate_contains_two_exact_copper_contacts_and_v4_unmodified() {
    let bytes = source(COPPER, SIGNAL, REFERENCE);
    let v5 = audit(&bytes, 1).unwrap();
    assert!(v5.starts_with("BOARDSI_REFERENCE_ROUTE_V5|"), "{v5}");
    assert!(v5.contains("|VERIFIED_SOURCE_STITCH_CANDIDATE|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|"));
    assert!(v5.contains("\nCONTACT|2|") && v5.contains("\nCONTACT|3|"), "{v5}");
    assert!(v5.contains("\nBOARDSI_REFERENCE_ROUTE_V4|"), "{v5}");
    assert!(v5.ends_with("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
    assert!(v5.contains("|0.052"), "distance must remain nonzero/display-only: {v5}");
    let legacy = boardsi_import::reference_route_v4::encode(
        &bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3]
    ).unwrap();
    assert!(v5.contains(&legacy), "V4 record changed");
}
#[test]
fn never_select_signal_via_or_unknown_via_as_reference() {
    let bytes = source(COPPER, SIGNAL, REFERENCE);
    assert!(audit(&bytes, 0).is_err());
    assert!(audit(&bytes, 2).is_err());
}
#[test]
fn fail_closed_on_missing_or_wrong_reference_net() {
    let wrong = REFERENCE.replace("NET_REF", "NET_OTHER");
    assert!(audit(&source(COPPER, SIGNAL, &wrong), 1).is_err());
    assert!(encode(&source(COPPER, SIGNAL, REFERENCE), &[0, 1], 1,
                   ["NET_REF", "NET_OTHER"], [2, 3], 1).is_err());
}
#[test]
fn fail_closed_if_one_layer_has_no_contact_or_via_is_outside_copper() {
    let missing = COPPER.replace("RB COPPER 100 200 1 1 0 NET_REF", 
        "RB COPPER 100 200 1 1 0 NET_OTHER");
    assert!(audit(&source(&missing, SIGNAL, REFERENCE), 1).is_err());
    let moved = REFERENCE.replace("50 50 3", "150 250 3");
    assert!(audit(&source(COPPER, SIGNAL, &moved), 1).is_err());
}
#[test]
fn fail_closed_for_reference_cutout_under_via_centre() {
    let cutout = COPPER.replace("RA COPPER 100 200 1 1 0", 
        "RA COPPER 100 200 2 1 0")
        .replace("RB COPPER", 
            "COPCUT 5 0.01 0 2\n40 40\n60 40\n60 60\n40 60\n40 40\nRB COPPER");
    assert!(audit(&source(&cutout, SIGNAL, REFERENCE), 1).is_err());
}
#[test]
fn changes_to_source_bytes_change_v5_receipt_and_never_enable_solver() {
    let bytes = source(COPPER, SIGNAL, REFERENCE);
    let a = audit(&bytes, 1).unwrap();
    let other = source(COPPER, &SIGNAL.replace("130 250", "131 250"), REFERENCE);
    let b = audit(&other, 1).unwrap();
    assert_ne!(a.lines().next(), b.lines().next());
    assert!(a.contains("|UNSUPPORTED|NOT_RUN|0|"));
    assert!(b.contains("|UNSUPPORTED|NOT_RUN|0|"));
}
