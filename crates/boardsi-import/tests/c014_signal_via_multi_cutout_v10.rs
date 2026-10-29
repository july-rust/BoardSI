//! Public C_014 synthetic Source-only V10, preserving valid legacy receipts.
use boardsi_import::{reference_route_v8, reference_route_v9, reference_route_v10};
const HEADER: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const SIGNAL: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n";
const REFERENCE: &str = "*SIGNAL* NET_REF\nR1.1 R2.2\n140 280 2 10 0\n180 280 3 10 0 SIMPLEVIA\n190 280 3 10 0\n";
fn rect(a: i32, b: i32, c: i32, d: i32, layer: i32) -> String {
    format!("COPCUT 5 0.01 0 {layer}\n{a} {b}\n{c} {b}\n{c} {d}\n{a} {d}\n{a} {b}\n")
}
fn copper(count_a: usize, count_b: usize) -> String {
    let corners = [(5,5,12,12), (20,5,27,12), (73,5,80,12),
                   (87,5,94,12), (5,80,12,87), (20,80,27,87), (5,88,12,95)];
    let mut text = String::from("*LINES*\n");
    for (name, layer, count) in [("RA", 2, count_a), ("RB", 3, count_b)] {
        text.push_str(&format!("{name} COPPER 100 200 {} 1 0 NET_REF\nCOPCLS 5 0.01 0 {layer}\n0 0\n100 0\n100 100\n0 100\n0 0\n", 1+count));
        if count != 0 { text.push_str(&rect(30,30,70,70,layer)); }
        for &(a,b,c,d) in corners.iter().take(count.saturating_sub(1)) {
            text.push_str(&rect(a,b,c,d,layer));
        }
    }
    text
}
fn asc(a: usize, b: usize) -> Vec<u8> {
    format!("{HEADER}{VIA}{}{SIGNAL}{REFERENCE}*END*\n", copper(a,b)).into_bytes()
}
fn v10(bytes: &[u8]) -> Result<String, &'static str> {
    reference_route_v10::encode(bytes, &[0,1], 1, ["NET_REF","NET_REF"], [2,3], 1)
}
fn v8(bytes: &[u8]) -> Result<String, &'static str> {
    reference_route_v8::encode(bytes, &[0,1], 1, ["NET_REF","NET_REF"], [2,3], 1)
}
fn legacy(wire: &str) -> (String, String) {
    let lines: Vec<_> = wire.lines().collect();
    let start = lines.iter().position(|l| l.starts_with("LEGACY|")).unwrap();
    let fields: Vec<_> = lines[start].split('|').collect();
    let count: usize = fields[2].parse().unwrap();
    (fields[1].to_owned(), format!("{}\n", lines[start+1..start+1+count].join("\n")))
}
#[test]
fn zero_one_and_multiple_keep_correct_legacy() {
    for (a,b) in [(0,0), (1,1), (2,2), (4,8), (8,1)] {
        let bytes = asc(a,b);
        let wire = v10(&bytes).unwrap();
        assert!(wire.starts_with("BOARDSI_REFERENCE_ROUTE_V10|"));
        assert_eq!(wire.lines().filter(|l| l.starts_with("HOLE|")).count(), a+b);
        let (kind, embedded) = legacy(&wire);
        if a<=1 && b<=1 {
            assert_eq!(kind, "V9");
            assert_eq!(embedded, reference_route_v9::encode(&bytes,&[0,1],1,["NET_REF","NET_REF"],[2,3],1).unwrap());
        } else {
            assert_eq!(kind, "V8");
            assert_eq!(embedded, v8(&bytes).unwrap());
        }
        assert!(wire.lines().next().unwrap().contains("|SOURCE_CUTOUT_"));
    }
}
#[test]
fn tangency_and_overlap_fail_safe() {
    let original = String::from_utf8(asc(2,2)).unwrap();
    let tangent = original.replace("30 30\n70 30\n70 70\n30 70\n30 30",
        "32.5 32.5\n67.5 32.5\n67.5 67.5\n32.5 67.5\n32.5 32.5");
    assert!(v10(tangent.as_bytes()).unwrap().contains("|SOURCE_CUTOUT_TANGENCY_RISK|"));
    let overlap = tangent.replace("32.5 32.5", "32.51 32.5").replace("32.5 67.5", "32.51 67.5");
    assert!(v10(overlap.as_bytes()).unwrap().contains("|SOURCE_REFERENCE_COPPER_OVERLAP_RISK|"));
}
#[test]
fn malformed_and_ambiguous_source_rejected() {
    let base = String::from_utf8(asc(2,2)).unwrap();
    let same = base.replace("COPCUT 5 0.01 0 2\n5 5\n12 5\n12 12\n5 12\n5 5",
        "COPCUT 5 0.01 0 2\n30 30\n70 30\n70 70\n30 70\n30 30");
    assert!(v10(same.as_bytes()).is_err());
    let wrong = base.replacen("COPCUT 5 0.01 0 2", "COPCUT 5 0.01 0 4", 1);
    assert!(v10(wrong.as_bytes()).is_err());
    let nonrect = base.replace("5 12\n5 5", "4 12\n5 5");
    assert!(v10(nonrect.as_bytes()).is_err());
    let net = base.replacen("NET_REF", "OTHER", 1);
    assert!(v10(net.as_bytes()).is_err());
    let too_many = base.replace("RA COPPER 100 200 3 1 0", "RA COPPER 100 200 10 1 0");
    assert!(v10(too_many.as_bytes()).is_err());
    assert_ne!(v10(base.as_bytes()).unwrap(), v10(base.replace("30 30", "31 30").as_bytes()).unwrap_err_or_default());
}
trait ErrorString {
    fn unwrap_err_or_default(self) -> String;
}
impl ErrorString for Result<String, &'static str> {
    fn unwrap_err_or_default(self) -> String { self.unwrap_or_else(|e| e.to_string()) }
}
