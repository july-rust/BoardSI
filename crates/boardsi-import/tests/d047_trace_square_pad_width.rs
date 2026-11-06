//! D_047 source-exact finite-width trace to square SMD pad tests.
use boardsi_import::d047_trace_square_pad_width;
const SOURCE: &str = include_str!("fixtures/d047_trace_square_pad_width_v94.asc");

fn audit(source: &str) -> Result<String, &'static str> {
    d047_trace_square_pad_width::encode(source.as_bytes())
}

#[test]
fn local_square_pad_contact_is_not_global_connectivity() {
    let receipt = audit(SOURCE).expect("supported synthetic SOURCE");
    assert!(receipt.starts_with("BOARDSI_D047_TRACE_SQUARE_PAD_WIDTH_V1|"));
    assert!(receipt.contains("C|0|U1|NET_A|1|5|4\n"));
    assert!(receipt.contains("C|1|U2|NET_A|1|4|3\n"));
    assert_eq!(receipt.matches("\nP|").count(), 4);
    assert_eq!(receipt.matches("\nT|").count(), 5);
    assert_eq!(receipt.matches("\nE|").count(), 7);
    assert!(receipt.contains("E|6|1|8|POSITIVE_FINITE_WIDTH_TRACE_PAD_CONTACT\n"));
    assert!(receipt.contains("COUNT|NODES|9|EDGES|7|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED"));
    assert!(receipt.ends_with("END|SOURCE_LOCAL_TRACE_PAD_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n"));
}

#[test]
fn corner_round_cap_reverse_net_header_crlf_tab_are_qualified() {
    let source = SOURCE.replace("\r\n", "\n");
    let base = "217 390 1 8 0\n217 410 1 8 0";
    let corner = "217 417 1 8 0\n225 425 1 8 0";
    assert_ne!(source.replace(base, corner), source);
    assert!(audit(&source.replace(base, corner)).is_ok());
    assert!(audit(&source.replace(base, "217 410 1 8 0\n217 390 1 8 0")).is_ok());
    assert!(audit(&source.replace("NET_A", "NET_B")).is_ok());
    assert!(audit(&source.replace('\n', "\r\n")).is_ok());
    assert!(audit(&source.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A")).is_ok());
    for header in [
        "!PADS-POWERPCB-V9.4-MILS-CP1252!",
        "!PADS-POWERPCB-V9.4-MILS-250L!",
    ] {
        assert!(audit(&source.replace("!PADS-POWERPCB-V9.4-MILS!", header)).is_ok());
    }
}

#[test]
fn tangency_gap_aabb_owner_layer_shape_and_hostile_sources_fail_closed() {
    let s = SOURCE.replace("\r\n", "\n");
    let base = "217 390 1 8 0\n217 410 1 8 0";
    let cases = [
        s.replace(base, "219 390 1 8 0\n219 410 1 8 0"),
        s.replace(base, "220 390 1 8 0\n220 410 1 8 0"),
        s.replace(base, "216 419 1 8 0\n224 427 1 8 0"),
        s.replace(base, "215 390 1 8 0\n215 410 1 8 0"),
        s.replace("U1.2 U2.2", "U1.1 U2.2"),
        s.replace("217 390 1 8 0", "217 390 2 8 0"),
        s.replace("217 390 1 8 0", "217 390 1 0 0"),
        s.replace("217 390 1 8 0", "217 390 1 20 0"),
        s.replace("PAD 2 3\n-2 30 S 0", "PAD 2 3\n-2 30 R 0"),
        s.replace("U2 TYPEA 400 300", "U2 TYPEA 260 300"),
        s.replace(base, "217 390 1 8 0\n217 390 1 8 0"),
        s.replace(&format!("U1.2 U2.2\n{base}\n"), ""),
        s.replace("*END*", &format!("U1.2 U2.2\n{base}\n*END*")),
        s.replace("*END*", "*VIA*\n*END*"),
        s.replace("*END*", "*END*\nFORGED"),
    ];
    for (i, candidate) in cases.iter().enumerate() {
        assert_ne!(candidate, &s, "inert mutation {i}");
        assert!(audit(candidate).is_err(), "hostile SOURCE case {i}");
    }
}
