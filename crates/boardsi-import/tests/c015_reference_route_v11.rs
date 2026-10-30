//! C_015 opt-in only; legacy must be true original-source V8/V4.
use boardsi_import::{reference_route_v11, reference_route_v4, reference_route_v8};
const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const ROUTE:&str="*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n*SIGNAL* NET_REF\nR1.1 R2.2\n140 280 2 10 0\n180 280 3 10 0 SIMPLEVIA\n190 280 3 10 0\n";
fn rect(layer: usize) -> String {
    format!("COPCUT 5 0.01 0 {layer}\n5 5\n12 5\n12 12\n5 12\n5 5\n")
}
fn circular(layer: usize) -> String {
    format!("COPCCO 2 0.01 0 {layer}\n30 50\n70 50\n")
}
fn copper(circle: bool) -> String {
    let mut s = String::from("*LINES*\n");
    for (name, layer) in [("RA", 2), ("RB", 3)] {
        s.push_str(&format!("{name} COPPER 100 200 3 1 0 NET_REF\nCOPCLS 5 0.01 0 {layer}\n0 0\n100 0\n100 100\n0 100\n0 0\n"));
        if circle {
            s.push_str(&circular(layer));
        } else {
            s.push_str(&format!(
                "COPCUT 5 0.01 0 {layer}\n30 30\n70 30\n70 70\n30 70\n30 30\n"
            ));
        }
        s.push_str(&rect(layer));
    }
    s
}
fn fixture(circle: bool) -> Vec<u8> {
    format!("{HEADER}{VIA}{}{ROUTE}*END*\n", copper(circle)).into_bytes()
}
fn v11(bytes: &[u8]) -> Result<String, &'static str> {
    reference_route_v11::encode(bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1)
}
#[test]
fn exact_source_rectangles_preserve_real_v8_legacy() {
    let bytes = fixture(false);
    let wire = v11(&bytes).unwrap();
    assert!(wire.starts_with("BOARDSI_REFERENCE_ROUTE_V11|"));
    assert!(wire.contains("|V8|V8_SOURCE_CONTACT_CANDIDATE\n"));
    assert_eq!(wire.lines().filter(|x| x.starts_with("HOLE|")).count(), 4);
    let v8 =
        reference_route_v8::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1).unwrap();
    assert!(wire.contains(&format!("LEGACY|V8|{}\n{v8}", v8.lines().count())));
}
#[test]
fn circular_source_has_genuine_legacy_v8_contact_witness() {
    let bytes = fixture(true);
    assert!(
        reference_route_v8::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3], 1).is_ok()
    );
    let wire = v11(&bytes).unwrap();
    assert!(wire.contains("|V4|SOURCE_CONTACT_UNVERIFIED\n"));
    assert_eq!(wire.lines().filter(|x| x.starts_with("HOLE|")).count(), 4);
    assert_eq!(wire.lines().filter(|x| x.contains("|C|")).count(), 2);
    assert!(wire.contains("SOURCE_CUTOUT_GAP_OBSERVED"));
    let v4 =
        reference_route_v4::encode(&bytes, &[0, 1], 1, ["NET_REF", "NET_REF"], [2, 3]).unwrap();
    assert!(wire.contains(&format!("LEGACY|V4|{}\n{v4}", v4.lines().count())));
}
#[test]
fn unsupported_circle_contact_and_geometry_fail_closed() {
    let bytes = String::from_utf8(fixture(true)).unwrap();
    let wrong_layer = bytes.replacen("COPCCO 2 0.01 0 2", "COPCCO 2 0.01 0 4", 1);
    assert!(v11(wrong_layer.as_bytes()).is_err());
    let wrong_net = bytes.replacen("NET_REF", "OTHER", 1);
    assert!(v11(wrong_net.as_bytes()).is_err());
    let overlapping = bytes.replacen("30 50\n70 50", "5 5\n45 5", 1);
    assert!(v11(overlapping.as_bytes()).is_err());
    let changed = bytes.replacen("NET_SIG", "NET_NEW", 1);
    assert_ne!(v11(bytes.as_bytes()), v11(changed.as_bytes()));
}
