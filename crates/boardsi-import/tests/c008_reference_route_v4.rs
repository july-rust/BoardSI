//! C_008 public synthetic source-only plated transition regression (no P009).
use boardsi_import::reference_route_v4::encode;

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const AREAS: &str = "*LINES*\nRA COPPER 100 200 1 1 0 NET_REFA\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nRB COPPER 100 200 1 1 0 NET_REFB\nCOPCLS 5 0.01 0 3\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const REF_ROUTES: &str = "*SIGNAL* NET_REFA\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n*SIGNAL* NET_REFB\nR3.1 R4.2\n300 400 3 10 0\n350 400 3 10 0\n";
const GOOD: &str = "130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0";
fn source(route: &str, extras: &str, area: &str) -> Vec<u8> {
    format!(
        "{HEADER}{VIA}{area}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n{route}\n{extras}{REF_ROUTES}*END*\n"
    )
    .into_bytes()
}
fn audit(bytes: &[u8], indices: &[usize], split: usize) -> Result<String, &'static str> {
    encode(bytes, indices, split, ["NET_REFA", "NET_REFB"], [2, 3])
}
#[test]
fn accepted_two_layer_route_has_source_via_and_independent_v2_receipts() {
    let out = audit(&source(GOOD, "", AREAS), &[0, 1], 1).unwrap();
    assert!(out.starts_with("BOARDSI_REFERENCE_ROUTE_V4|"), "{out}");
    assert!(out.contains("|REFERENCE_TRANSFER_UNVERIFIED|"), "{out}");
    assert!(out.contains("\nVIA|0|"), "{out}");
    assert!(out.contains("SEG|0|") && out.contains("SEG|1|"), "{out}");
    assert!(out.contains("BOARDSI_REFERENCE_GAP_V2|"), "{out}");
    assert!(
        out.ends_with("END|2|0|1|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"),
        "{out}"
    );
    assert!(out.contains("|UNSUPPORTED|NOT_RUN|0|"), "{out}");
}
#[test]
fn source_cutout_produces_layer_a_gap_and_still_unverified_transition() {
    let first = AREAS.replace("RA COPPER 100 200 1 1 0", "RA COPPER 100 200 2 1 0");
    let extra = "COPCUT 5 0.01 0 2\n40 40\n60 40\n60 60\n40 60\n40 40\n";
    let modified = first.replace("RB COPPER", &format!("{extra}RB COPPER"));
    let out = audit(&source(GOOD, "", &modified), &[0, 1], 1).unwrap();
    assert!(out.contains("|REFERENCE_SPLIT_RISK"), "{out}");
    assert!(out.lines().any(|line| line.starts_with("I|")), "{out}");
}
#[test]
fn requires_one_explicit_connected_plated_via_at_transition() {
    let missing = GOOD.replace(" SIMPLEVIA", "");
    assert!(audit(&source(&missing, "", AREAS), &[0, 1], 1).is_err());
    // A changed VIA vertex also moves its adjacent trace endpoint, and is
    // therefore still connected. Select an actually disjoint same-net trace.
    let other = "*SIGNAL* NET_SIG\nU3.1 U4.2\n190 280 4 10 0\n210 280 4 10 0\n";
    assert!(audit(&source(GOOD, other, AREAS), &[0, 2], 1).is_err());
}
#[test]
fn invalid_split_duplicate_reversed_and_wrong_layers_rejected() {
    let data = source(GOOD, "", AREAS);
    for (indices, split) in [
        (vec![0, 1], 0),
        (vec![0, 1], 2),
        (vec![0, 0], 1),
        (vec![1, 0], 1),
        (vec![0, 999], 1),
    ] {
        assert!(
            audit(&data, &indices, split).is_err(),
            "{indices:?}/{split}"
        );
    }
}
#[test]
fn selected_route_rejects_unselected_contact_and_same_net_arc() {
    let branch = "*SIGNAL* NET_SIG\nU3.1 U4.2\n135 250 1 10 0\n135 270 1 10 0\n";
    assert!(audit(&source(GOOD, branch, AREAS), &[0, 1], 1).is_err());
}
#[test]
fn third_conductive_layer_branch_at_selected_via_is_unsupported() {
    // SIMPLEVIA declares conductive pads on 1/2/3/4; layer 2 branch
    // at the source via centre must not be silently treated as unbranched.
    let branch = "*SIGNAL* NET_SIG\nU3.1 U4.2\n150 250 2 10 0\n150 270 2 10 0\n";
    let result = audit(&source(GOOD, branch, AREAS), &[0, 1], 1);
    assert_eq!(result, Err("UNSELECTED_VIA_BRANCH"));
}
#[test]
fn cross_net_reference_or_source_change_never_promotes_physical_status() {
    let data = source(GOOD, "", AREAS);
    let out = encode(&data, &[0, 1], 1, ["NET_REFA", "NET_SIG"], [2, 3]).unwrap();
    assert!(out.contains("|UNSUPPORTED|"), "{out}");
    let changed = source(
        "131 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0",
        "",
        AREAS,
    );
    let other = audit(&changed, &[0, 1], 1).unwrap();
    assert_ne!(
        audit(&data, &[0, 1], 1).unwrap().lines().next(),
        other.lines().next()
    );
}
#[test]
fn v2_and_v3_legacy_audits_remain_unchanged() {
    let data = source(GOOD, "", AREAS);
    let before = boardsi_import::reference_gap_intervals::encode(&data, 0, "NET_REFA", 2).unwrap();
    let _ = audit(&data, &[0, 1], 1).unwrap();
    let after = boardsi_import::reference_gap_intervals::encode(&data, 0, "NET_REFA", 2).unwrap();
    assert_eq!(before, after);
}
