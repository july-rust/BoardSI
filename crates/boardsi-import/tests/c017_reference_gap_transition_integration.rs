//! C_017: bounded source-bound V1/V2/V3/V4 cross-version regression.
//! Public synthetic PADS V9.4 MILS only. Never P009 or physical SI.
use boardsi_import::{
    reference_gap_intervals, reference_risk_wire, reference_route_v3, reference_route_v4,
};

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const SINGLE_AREA: &str = "*LINES*\nREF COPPER 100 200 2 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 2\n20 20\n40 20\n40 40\n20 40\n20 20\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const TWO_AREAS: &str = "*LINES*\nRA COPPER 100 200 2 1 0 NET_REFA\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 2\n40 40\n60 40\n60 60\n40 60\n40 40\nRB COPPER 100 200 1 1 0 NET_REFB\nCOPCLS 5 0.01 0 3\n0 0\n100 0\n100 100\n0 100\n0 0\n";

fn single_source(last: i32) -> Vec<u8> {
    format!(
        "{HEADER}{SINGLE_AREA}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n110 230 1 10 0\n150 230 1 10 0\n{last} 230 1 10 0\n*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n*END*\n"
    )
    .into_bytes()
}
fn via_source(last: i32) -> Vec<u8> {
    format!(
        "{HEADER}{VIA}{TWO_AREAS}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n{last} 250 4 10 0\n*SIGNAL* NET_REFA\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n*SIGNAL* NET_REFB\nR3.1 R4.2\n300 400 3 10 0\n350 400 3 10 0\n*END*\n"
    )
    .into_bytes()
}
fn source_sha(wire: &str) -> &str {
    wire.lines().next().unwrap().split('|').nth(1).unwrap()
}

#[test]
fn single_layer_v1_v2_v3_same_source_and_exact_gap_ancestry() {
    let bytes = single_source(170);
    let v1 = reference_risk_wire::encode(&bytes, &[0], "NET_REF", &[2]).unwrap();
    let v2_first = reference_gap_intervals::encode(&bytes, 0, "NET_REF", 2).unwrap();
    let v2_second = reference_gap_intervals::encode(&bytes, 1, "NET_REF", 2).unwrap();
    let v3 = reference_route_v3::encode(&bytes, &[0, 1], "NET_REF", 2).unwrap();
    assert!(v1.starts_with("BOARDSI_REFERENCE_RISK_V1|"));
    assert!(v2_first.starts_with("BOARDSI_REFERENCE_GAP_V2|"));
    assert!(v3.starts_with("BOARDSI_REFERENCE_ROUTE_V3|"));
    assert_eq!(source_sha(&v1), source_sha(&v2_first));
    assert_eq!(source_sha(&v2_first), source_sha(&v2_second));
    assert_eq!(source_sha(&v2_first), source_sha(&v3));
    assert!(v1.contains("REFERENCE_SPLIT_RISK"), "{v1}");
    assert!(v2_first.contains("REFERENCE_SPLIT_RISK"), "{v2_first}");
    assert!(v3.contains("REFERENCE_SPLIT_RISK"), "{v3}");
    assert!(v3.contains(&v2_first) && v3.contains(&v2_second));
    assert!(v3.contains("|UNSUPPORTED|NOT_RUN|0"));
    assert!(v3.ends_with("SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}

#[test]
fn changed_source_invalidates_all_single_layer_receipt_hashes() {
    let old = single_source(170);
    let updated = single_source(171);
    let v1_old = reference_risk_wire::encode(&old, &[0], "NET_REF", &[2]).unwrap();
    let v1_new = reference_risk_wire::encode(&updated, &[0], "NET_REF", &[2]).unwrap();
    let v2_old = reference_gap_intervals::encode(&old, 0, "NET_REF", 2).unwrap();
    let v2_new = reference_gap_intervals::encode(&updated, 0, "NET_REF", 2).unwrap();
    let v3_old = reference_route_v3::encode(&old, &[0, 1], "NET_REF", 2).unwrap();
    let v3_new = reference_route_v3::encode(&updated, &[0, 1], "NET_REF", 2).unwrap();
    assert_ne!(source_sha(&v1_old), source_sha(&v1_new));
    assert_ne!(source_sha(&v2_old), source_sha(&v2_new));
    assert_ne!(source_sha(&v3_old), source_sha(&v3_new));
    assert_eq!(source_sha(&v1_new), source_sha(&v3_new));
    assert_eq!(source_sha(&v2_new), source_sha(&v3_new));
}

#[test]
fn plated_two_layer_v4_embeds_both_source_bound_v2_receipts() {
    let bytes = via_source(170);
    let left = reference_gap_intervals::encode(&bytes, 0, "NET_REFA", 2).unwrap();
    let right = reference_gap_intervals::encode(&bytes, 1, "NET_REFB", 3).unwrap();
    let v4 =
        reference_route_v4::encode(&bytes, &[0, 1], 1, ["NET_REFA", "NET_REFB"], [2, 3]).unwrap();
    assert!(v4.starts_with("BOARDSI_REFERENCE_ROUTE_V4|"));
    assert_eq!(source_sha(&v4), source_sha(&left));
    assert_eq!(source_sha(&v4), source_sha(&right));
    assert!(v4.contains(&left) && v4.contains(&right));
    assert!(v4.contains("REFERENCE_TRANSFER_UNVERIFIED"));
    assert!(v4.contains("REFERENCE_SPLIT_RISK"), "{v4}");
    assert!(v4.contains("|UNSUPPORTED|NOT_RUN|0|"));
    assert!(v4.ends_with("END|2|1|1|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}

#[test]
fn revised_via_source_or_unsupported_reference_never_promotes_to_si_pass() {
    let old = via_source(170);
    let changed = via_source(171);
    let old_v4 =
        reference_route_v4::encode(&old, &[0, 1], 1, ["NET_REFA", "NET_REFB"], [2, 3]).unwrap();
    let new_v4 =
        reference_route_v4::encode(&changed, &[0, 1], 1, ["NET_REFA", "NET_REFB"], [2, 3]).unwrap();
    assert_ne!(source_sha(&old_v4), source_sha(&new_v4));
    let unsupported =
        reference_route_v4::encode(&old, &[0, 1], 1, ["MISSING_REF", "NET_REFB"], [2, 3]).unwrap();
    assert!(unsupported.contains("|UNSUPPORTED|NOT_RUN|0|"));
    assert!(unsupported.contains("|UNSUPPORTED\n") || unsupported.contains("|UNSUPPORTED|"));
    assert!(!unsupported.contains("PHYSICAL_PASS"));
}
