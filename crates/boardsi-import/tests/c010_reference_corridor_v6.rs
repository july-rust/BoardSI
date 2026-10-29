//! C_010 public synthetic PADS corridor vs Reference Stitch source Area identity.
//! Every result remains SOURCE_GEOMETRY_ONLY, never real P009/physical SI.
use boardsi_import::{reference_route_v5, reference_route_v6};
const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const COPPER: &str = "*LINES*\nRA COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nRB COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 3\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const SIGNAL: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n";
const REFERENCE: &str =
    "*SIGNAL* NET_REF\nR1.1 R2.2\n140 280 2 10 0\n180 280 3 10 0 SIMPLEVIA\n190 280 3 10 0\n";
fn source(copper: &str, signal: &str) -> Vec<u8> {
    format!("{HEADER}{VIA}{copper}{signal}{REFERENCE}*END*\n").into_bytes()
}
fn v6(bytes: &[u8]) -> Result<String, &'static str> {
    reference_route_v6::encode(bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1)
}
#[test]
fn valid_same_area_source_candidate_embeds_exact_original_v5() {
    let bytes = source(COPPER, SIGNAL);
    let result = v6(&bytes).unwrap();
    assert!(result.contains("|SAME_SOURCE_AREA_CORRIDOR_CANDIDATE|UNSUPPORTED|NOT_RUN|0|"));
    assert_eq!(
        result
            .lines()
            .filter(|l| l.starts_with("CORRIDOR|"))
            .count(),
        2
    );
    assert_eq!(
        result
            .lines()
            .filter(|l| l.ends_with("|SAME_SOURCE_AREA"))
            .count(),
        2
    );
    let legacy =
        reference_route_v5::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1).unwrap();
    assert!(result.contains(&legacy));
    assert!(result.ends_with("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
    assert!(!result.contains("physical_si=PASS"));
}
#[test]
fn disconnected_same_net_area_is_mismatch_not_a_return_path() {
    // RA strictly contains the via (180,280), while disjoint RC holds the
    // signal segment (130..150,250). SAME Net labels must not join islands.
    let copper=COPPER.replace(
        "RA COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n",
        "RA COPPER 170 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n30 0\n30 100\n0 100\n0 0\nRC COPPER 110 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n50 0\n50 100\n0 100\n0 0\n",
    );
    let result = v6(&source(&copper, SIGNAL)).unwrap();
    assert!(
        result.contains("|REFERENCE_AREA_MISMATCH|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
    assert!(result.contains("|REFERENCE_AREA_MISMATCH\n"), "{result}");
}
#[test]
fn corridor_cutout_with_via_contact_intact_is_gap() {
    let copper = COPPER
        .replace(
            "RA COPPER 100 200 1 1 0 NET_REF",
            "RA COPPER 100 200 2 1 0 NET_REF",
        )
        .replace(
            "RB COPPER",
            "COPCUT 5 0.01 0 2\n35 45\n45 45\n45 55\n35 55\n35 45\nRB COPPER",
        );
    let result = v6(&source(&copper, SIGNAL)).unwrap();
    assert!(
        result.contains("|REFERENCE_CORRIDOR_GAP|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
}
#[test]
fn touching_boundary_does_not_become_strict_corridor() {
    let boundary = SIGNAL.replace("130 250 1", "100 250 1");
    let result = v6(&source(COPPER, &boundary)).unwrap();
    assert!(
        result.contains("|REFERENCE_CORRIDOR_GAP|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
}
#[test]
fn source_changes_invalidate_v6_hash_and_do_not_reuse_v5() {
    let a = v6(&source(COPPER, SIGNAL)).unwrap();
    let b = v6(&source(COPPER, &SIGNAL.replace("130 250", "131 250"))).unwrap();
    assert_ne!(
        a.lines().next().unwrap().split('|').nth(1),
        b.lines().next().unwrap().split('|').nth(1)
    );
    assert!(a.contains("|UNSUPPORTED|NOT_RUN|0|"));
    assert!(b.contains("|UNSUPPORTED|NOT_RUN|0|"));
}
#[test]
fn wrong_reference_net_or_stitch_index_must_fail_closed() {
    let bytes = source(COPPER, SIGNAL);
    assert!(
        reference_route_v6::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_OTHER"], [2, 3], 1)
            .is_err()
    );
    assert!(
        reference_route_v6::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 0).is_err()
    );
    assert!(
        reference_route_v6::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 99).is_err()
    );
    assert!(
        reference_route_v6::encode(&bytes, &[1, 0], 1, ["NET_REF", "NET_REF"], [2, 3], 1).is_err()
    );
}
