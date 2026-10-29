//! Public synthetic, bounded C_011 V7 footprint vs Source Copper risks.
//! No electrical return-current or solver qualification.
use boardsi_import::{reference_route_v6, reference_route_v7};

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const COPPER: &str = "*LINES*\nRA COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nRB COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 3\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const SIGNAL: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n";
const REFERENCE: &str =
    "*SIGNAL* NET_REF\nR1.1 R2.2\n140 280 2 10 0\n180 280 3 10 0 SIMPLEVIA\n190 280 3 10 0\n";
fn source(copper: &str, via: &str, signal: &str) -> Vec<u8> {
    format!("{HEADER}{via}{copper}{signal}{REFERENCE}*END*\n").into_bytes()
}
fn v7(bytes: &[u8]) -> Result<String, &'static str> {
    reference_route_v7::encode(bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1)
}
#[test]
fn round_pad_entirely_inside_both_copper_areas_embeds_unchanged_v6() {
    let bytes = source(COPPER, VIA, SIGNAL);
    let result = v7(&bytes).unwrap();
    assert!(
        result.contains("|PAD_FOOTPRINT_CONTAINED_CANDIDATE|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
    assert_eq!(
        result
            .lines()
            .filter(|l| l.ends_with("|PAD_FOOTPRINT_CONTAINED_CANDIDATE"))
            .count(),
        2
    );
    assert!(result.contains("|3500|1500|"), "{result}");
    let old =
        reference_route_v6::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1).unwrap();
    assert!(result.contains(&old));
    assert!(result.ends_with("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
    assert!(!result.contains("physical_si=PASS"));
}
#[test]
fn centre_inside_but_pad_overhangs_copper_outer() {
    // Right boundary at 195 mil, centre x180 is inside but radius=17.5 mil.
    let smaller = COPPER.replacen("100 0\n100 100", "95 0\n95 100", 1);
    let result = v7(&source(&smaller, VIA, SIGNAL)).unwrap();
    assert!(
        result.contains("|PAD_BOUNDARY_RISK|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
    assert_eq!(
        result
            .lines()
            .filter(|l| l.ends_with("|PAD_BOUNDARY_RISK"))
            .count(),
        1
    );
}
#[test]
fn exact_tangent_to_copper_outer_is_risk_not_success() {
    let tangent = COPPER.replacen("100 0\n100 100", "97.5 0\n97.5 100", 1);
    let result = v7(&source(&tangent, VIA, SIGNAL)).unwrap();
    assert!(
        result.contains("|PAD_BOUNDARY_RISK|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
}
#[test]
fn cutout_near_via_rim_rejects_footprint_but_via_centre_remains_supported() {
    let hole = COPPER
        .replacen(
            "RA COPPER 100 200 1 1 0 NET_REF",
            "RA COPPER 100 200 2 1 0 NET_REF",
            1,
        )
        .replacen(
            "RB COPPER",
            "COPCUT 5 0.01 0 2\n65 65\n75 65\n75 95\n65 95\n65 65\nRB COPPER",
            1,
        );
    let result = v7(&source(&hole, VIA, SIGNAL)).unwrap();
    assert!(
        result.contains("|PAD_CUTOUT_RISK|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
}
#[test]
fn different_source_sha_and_invalid_reference_via_are_fail_closed() {
    let a = v7(&source(COPPER, VIA, SIGNAL)).unwrap();
    let b = v7(&source(COPPER, VIA, &SIGNAL.replace("130 250", "131 250"))).unwrap();
    assert_ne!(
        a.lines().next().unwrap().split('|').nth(1),
        b.lines().next().unwrap().split('|').nth(1)
    );
    let bytes = source(COPPER, VIA, SIGNAL);
    assert!(
        reference_route_v7::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 0).is_err()
    );
    assert!(
        reference_route_v7::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_OTHER"], [2, 3], 1)
            .is_err()
    );
}
#[test]
fn invalid_or_unsupported_padstack_never_becomes_valid_footprint() {
    let invalid = source(COPPER, &VIA.replace("-1 35 R", "-1 15 R"), SIGNAL);
    assert!(v7(&invalid).is_err());
    let no_middle = source(COPPER, "*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n", SIGNAL);
    assert!(v7(&no_middle).is_err());
    let non_round = source(COPPER, &VIA.replace("-1 35 R", "-1 35 S"), SIGNAL);
    assert!(v7(&non_round).is_err());
}
#[test]
fn same_net_different_copper_islands_cannot_qualify() {
    let disconnected = COPPER.replacen(
      "RA COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n",
      "RA COPPER 170 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n30 0\n30 100\n0 100\n0 0\nRC COPPER 110 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n50 0\n50 100\n0 100\n0 0\n",1);
    let result = v7(&source(&disconnected, VIA, SIGNAL)).unwrap();
    assert!(
        result.contains("|V6_CORRIDOR_RISK|UNSUPPORTED|NOT_RUN|0|"),
        "{result}"
    );
}
