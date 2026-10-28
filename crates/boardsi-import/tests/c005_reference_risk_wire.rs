//! C_005 exact source-to-Qt bounded risk transport regression. Public synthetic only.
use boardsi_import::reference_risk_wire::encode;

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const AREA: &str = "*LINES*\nREF COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const REF_ROUTE: &str = "*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n";
fn source(copper: &str, route: &str, via: &str) -> String {
    format!(
        "{HEADER}{via}{copper}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n{route}\n{REF_ROUTE}*END*\n"
    )
}
#[test]
fn source_covered_is_explicitly_not_a_physical_pass() {
    let bytes = source(AREA, "130 250 1 10 0\n170 250 1 10 0", "");
    let out = encode(bytes.as_bytes(), &[0], "NET_REF", &[2]).unwrap();
    assert!(out.contains("|SOURCE_GEOMETRY_ONLY|"));
    assert!(out.contains("|0|UNSUPPORTED|NOT_RUN|0\n"));
    assert!(out.contains("T|0|1|"));
    assert!(!out.contains("\nM|"));
    assert!(out.ends_with("END|1|0|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}
#[test]
fn hole_and_gap_report_risk_and_only_representative_midpoint() {
    let copper = AREA.replace("100 200 1 1 0", "100 200 2 1 0");
    let hole = "COPCUT 5 0.01 0 2\n20 20\n40 20\n40 40\n20 40\n20 20\n";
    let bytes = source(
        &format!("{copper}{hole}"),
        "110 230 1 10 0\n170 230 1 10 0",
        "",
    );
    let out = encode(bytes.as_bytes(), &[0], "NET_REF", &[2]).unwrap();
    assert!(out.contains("|REFERENCE_SPLIT_RISK|SOURCE_REFERENCE_GAP_OR_CUTOUT|"));
    assert!(out.contains("\nM|"));
    assert!(out.ends_with("END|1|1|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}
#[test]
fn plated_transition_keeps_p009_unsupported() {
    let bytes = source(
        AREA,
        "130 250 1 10 0\n150 250 3 10 0 SIMPLEVIA\n170 250 3 10 0",
        VIA,
    );
    let out = encode(bytes.as_bytes(), &[0, 1], "NET_REF", &[2, 2]).unwrap();
    assert!(out.contains("|LAYER_TRANSITION_RISK|"));
    assert!(out.contains("|2|0|UNSUPPORTED|NOT_RUN|0\n"));
    assert!(out.contains("\nM|"));
    assert!(out.ends_with("END|2|1|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}
#[test]
fn invalid_selection_fails_without_untrusted_success() {
    let bytes = source(AREA, "130 250 1 10 0\n170 250 1 10 0", "");
    for (ids, layers, ref_name) in [
        (vec![], vec![], "NET_REF"),
        (vec![0, 0], vec![2, 2], "NET_REF"),
        (vec![usize::MAX], vec![2], "NET_REF"),
        (vec![0], vec![0], "NET_REF"),
        (vec![0], vec![999], "NET_REF"),
        (vec![0], vec![2], "BAD|REF"),
    ] {
        assert!(encode(bytes.as_bytes(), &ids, ref_name, &layers).is_err());
    }
}
#[test]
fn missing_reference_and_wrong_layer_are_unsupported_not_safe() {
    let bytes = source(AREA, "130 250 1 10 0\n170 250 1 10 0", "");
    for (name, layer) in [("INVALID_REF", 2), ("NET_REF", 4), ("NET_SIG", 2)] {
        let result = encode(bytes.as_bytes(), &[0], name, &[layer]).unwrap();
        assert!(result.contains("|UNSUPPORTED|"));
        assert!(!result.contains("\nM|"));
    }
}
