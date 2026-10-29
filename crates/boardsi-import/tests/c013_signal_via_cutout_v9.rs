//! Public synthetic V9 source-geometry only. Never physical DRC or return current.
use boardsi_import::{reference_route_v8, reference_route_v9};

const HEADER: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const SIGNAL: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n";
const REFERENCE: &str = "*SIGNAL* NET_REF\nR1.1 R2.2\n140 280 2 10 0\n180 280 3 10 0 SIMPLEVIA\n190 280 3 10 0\n";

fn copper(cutout: Option<(&str, &str, &str, &str)>) -> String {
    let mut s = "*LINES*\n".to_string();
    for (name, layer) in [("RA", 2), ("RB", 3)] {
        let parts = if cutout.is_some() { 2 } else { 1 };
        s.push_str(&format!("{name} COPPER 100 200 {parts} 1 0 NET_REF\nCOPCLS 5 0.01 0 {layer}\n0 0\n100 0\n100 100\n0 100\n0 0\n"));
        if let Some((left, lower, right, upper)) = cutout {
            s.push_str(&format!("COPCUT 5 0.01 0 {layer}\n{left} {lower}\n{right} {lower}\n{right} {upper}\n{left} {upper}\n{left} {lower}\n"));
        }
    }
    s
}
fn asc(copper: &str) -> Vec<u8> {
    format!("{HEADER}{VIA}{copper}{SIGNAL}{REFERENCE}*END*\n").into_bytes()
}
fn v9(src: &[u8]) -> Result<String, &'static str> {
    reference_route_v9::encode(src, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1)
}
fn v8(src: &[u8]) -> Result<String, &'static str> {
    reference_route_v8::encode(src, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1)
}
fn embedded_v8(wire: &str) -> String {
    let lines: Vec<_> = wire.lines().collect();
    let fields: Vec<_> = lines[3].split('|').collect();
    assert_eq!(fields[0], "V8");
    let n: usize = fields[1].parse().unwrap();
    format!("{}\n", lines[4..4+n].join("\n"))
}
#[test]
fn cutout_positive_and_exact_v8_receipt() {
    let bytes = asc(&copper(Some(("30","30","70","70"))));
    let wire = v9(&bytes).unwrap();
    let header: Vec<_> = wire.lines().next().unwrap().split('|').collect();
    assert_eq!(header.len(), 12);
    assert_eq!(header[0], "BOARDSI_REFERENCE_ROUTE_V9");
    assert_eq!(header[2], "SOURCE_CUTOUT_GAP_OBSERVED");
    assert_eq!(&header[3..7], &["UNSUPPORTED","NOT_RUN","0","REFERENCE_TRANSFER_UNVERIFIED"]);
    assert_eq!(wire.lines().filter(|l| l.starts_with("CUTOUT|")).count(), 2);
    assert_eq!(embedded_v8(&wire), v8(&bytes).unwrap());
    assert!(wire.ends_with("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}
#[test]
fn tangent_and_one_tick_shift_to_overlap() {
    let tangent = asc(&copper(Some(("32.5","32.5","67.5","67.5"))));
    let wire = v9(&tangent).unwrap();
    assert!(wire.lines().next().unwrap().contains("|SOURCE_CUTOUT_TANGENCY_RISK|"),"{wire}");
    let shifted = asc(&copper(Some(("32.51","32.5","67.51","67.5"))));
    let wire = v9(&shifted).unwrap();
    assert!(wire.lines().next().unwrap().contains("|SOURCE_REFERENCE_COPPER_OVERLAP_RISK|"),"{wire}");
}
#[test]
fn absent_cutout_is_a_warning_not_a_pass() {
    let bytes=asc(&copper(None));
    let wire=v9(&bytes).unwrap();
    assert!(wire.lines().next().unwrap().contains("|SOURCE_CUTOUT_MISSING_RISK|"));
    assert_eq!(embedded_v8(&wire),v8(&bytes).unwrap());
}
#[test]
fn invalid_ownership_unsupported_geometry_and_mutation_fail_closed() {
    let base=copper(Some(("30","30","70","70")));
    let missing_ownership=base.replace("RA COPPER 100 200 2 1 0", "RA COPPER 100 200 1 1 0");
    assert!(v9(&asc(&missing_ownership)).is_err());
    let wrong_layer=base.replacen("COPCUT 5 0.01 0 2", "COPCUT 5 0.01 0 4",1);
    assert!(v9(&asc(&wrong_layer)).is_err());
    let wrong_net=base.replacen("0 NET_REF", "0 NET_OTHER",1);
    assert!(v9(&asc(&wrong_net)).is_err());
    let malformed=base.replace("30 30\n70 30\n70 70\n30 70", "30 30\n70 30\n65 70\n30 70");
    assert!(v9(&asc(&malformed)).is_err());
    let a=v9(&asc(&base)).unwrap();
    let b=v9(&asc(&base.replace("30 30", "31 30"))).err();
    assert!(b.is_some()); // Invalid rectangle must not silently become geometrically accepted.
    assert!(!a.contains("solver_ready=true"));
}
