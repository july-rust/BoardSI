//! Public synthetic, bounded C_012 V8 Signal Via / Reference Stitch Via Source pad clearance.
//! No electrical return-current or solver qualification.
use boardsi_import::{reference_route_v7, reference_route_v8};

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
fn v8(bytes: &[u8]) -> Result<String, &'static str> {
    reference_route_v8::encode(bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1)
}
#[test]
fn positive_separation_contains_exact_v7_receipt_and_source_identity() {
    let bytes = source(COPPER, VIA, SIGNAL);
    let wire = v8(&bytes).unwrap();
    let v7 = reference_route_v7::encode(
        &bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1
    ).unwrap();
    let head: Vec<_> = wire.lines().next().unwrap().split('|').collect();
    assert_eq!(head.len(), 13);
    assert_eq!(head[0], "BOARDSI_REFERENCE_ROUTE_V8");
    assert_eq!(head[2], "SOURCE_PAD_SEPARATE_OBSERVED");
    assert_eq!(head[3..7], ["UNSUPPORTED", "NOT_RUN", "0", "REFERENCE_TRANSFER_UNVERIFIED"]);
    assert_eq!(head[10], "1"); // Stitch Via source index.
    assert_eq!(head[11], "0"); // Independently determined Signal Via.
    assert_eq!(head[12], "PAD_FOOTPRINT_CONTAINED_CANDIDATE");
    assert_eq!(wire.lines().filter(|l| l.starts_with("CLEARANCE|")).count(), 2);
    assert!(wire.contains("CLEARANCE|2|0|1|15000|25000|18000|28000|3500|3500|1500|1500|SOURCE_PAD_SEPARATE_OBSERVED"));
    assert!(wire.contains(&v7));
    assert!(wire.ends_with("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
    assert!(!wire.contains("physical_si=PASS"));
}
#[test]
fn tangent_overlap_and_one_source_tick_change_are_distinct() {
    let tangent = REFERENCE.replace(
        "180 280 3 10 0 SIMPLEVIA", "185 250 3 10 0 SIMPLEVIA"
    );
    let bytes = format!("{HEADER}{VIA}{COPPER}{SIGNAL}{tangent}*END*\n").into_bytes();
    let result = v8(&bytes).unwrap();
    assert!(result.lines().next().unwrap().contains("|SOURCE_PAD_TANGENT_RISK|"), "{result}");
    assert_eq!(result.lines().filter(|l| l.ends_with("|SOURCE_PAD_TANGENT_RISK")).count(), 2);

    let overlap = tangent.replace("185 250 3 10 0 SIMPLEVIA", "184 250 3 10 0 SIMPLEVIA");
    let overlap_bytes = format!("{HEADER}{VIA}{COPPER}{SIGNAL}{overlap}*END*\n").into_bytes();
    let result = v8(&overlap_bytes).unwrap();
    assert!(result.lines().next().unwrap().contains("|SOURCE_PAD_OVERLAP_RISK|"), "{result}");

    let separate = tangent.replace("185 250 3 10 0 SIMPLEVIA", "185.01 250 3 10 0 SIMPLEVIA");
    let separate_bytes = format!("{HEADER}{VIA}{COPPER}{SIGNAL}{separate}*END*\n").into_bytes();
    let result = v8(&separate_bytes).unwrap();
    assert!(result.lines().next().unwrap().contains("|SOURCE_PAD_SEPARATE_OBSERVED|"), "{result}");
}
#[test]
fn invalid_source_padstack_net_and_reference_index_fail_closed() {
    let bytes=source(COPPER,VIA,SIGNAL);
    assert!(reference_route_v8::encode(
        &bytes, &[0,1],1,["NET_REF","NET_REF"],[2,3],0).is_err());
    let missing_middle=source(COPPER,"*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n",SIGNAL);
    assert!(v8(&missing_middle).is_err());
    let invalid=source(COPPER,&VIA.replace("-1 35 R","-1 35 S"),SIGNAL);
    assert!(v8(&invalid).is_err());
    let other=source(COPPER,VIA,&SIGNAL.replace("*SIGNAL* NET_SIG","*SIGNAL* NET_REF"));
    assert!(v8(&other).is_err());
}
#[test]
fn source_mutation_invalidates_entire_versioned_receipt() {
    let a=v8(&source(COPPER,VIA,SIGNAL)).unwrap();
    let b=v8(&source(COPPER,VIA,&SIGNAL.replace("130 250","131 250"))).unwrap();
    assert_ne!(a.lines().next().unwrap().split('|').nth(1),
               b.lines().next().unwrap().split('|').nth(1));
}
