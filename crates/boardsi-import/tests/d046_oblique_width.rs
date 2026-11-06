//! D_046 SOURCE-local, oblique finite-width copper contact qualification.
use boardsi_import::d046_oblique_width;
const SOURCE: &str = include_str!("fixtures/d046_oblique_width_v94.asc");

fn audit(s: &str) -> Result<String, &'static str> {
    d046_oblique_width::encode(s.as_bytes())
}

#[test]
fn exact_oblique_bridge_connects_without_centerline_crossing() {
    let receipt = audit(SOURCE).expect("bounded synthetic source");
    assert!(receipt.starts_with("BOARDSI_D046_OBLIQUE_WIDTH_CONTACT_V1|"));
    assert!(receipt.contains("C|0|U1+U2|NET_A|1|9|10\n"));
    assert_eq!(receipt.matches("\nP|").count(), 4);
    assert_eq!(receipt.matches("\nT|").count(), 5);
    assert_eq!(receipt.matches("\nE|").count(), 10);
    assert!(receipt.contains("E|9|7|8|POSITIVE_FINITE_WIDTH_OBLIQUE_CONTACT\n"));
    assert!(receipt
        .contains("COUNT|NODES|9|EDGES|10|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED"));
    assert!(receipt.ends_with("END|SOURCE_LOCAL_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n"));
}

#[test]
fn direction_net_headers_crlf_and_tab_pass() {
    let source = SOURCE.replace("\r\n", "\n");
    let reversed = source.replace(
        "224 350 1 42 0\n236 357 1 42 0",
        "236 357 1 42 0\n224 350 1 42 0",
    );
    assert_ne!(reversed, source);
    assert!(audit(&reversed).is_ok());
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
fn boundary_corner_gap_owner_and_hostile_sources_fail_closed() {
    let source = SOURCE.replace("\r\n", "\n");
    let witness = "224 350 1 42 0\n236 357 1 42 0";
    let cases = [
        source.replace("1 42 0", "1 40 0"),
        source.replace("1 42 0", "1 38 0"),
        source.replace(witness, "224 363 1 42 0\n236 370 1 42 0"),
        source.replace("224 350 1 42 0", "224 350 2 42 0"),
        source.replace("U1.1 U2.1", "U1.1 U1.2"),
        source.replace("224 350 1 42 0", "224 350 1 0 0"),
        source.replace(witness, "200 350 1 42 0\n236 357 1 42 0"),
        source.replace(witness, "220 350 1 42 0\n260 350 1 42 0"),
        source.replace(witness, "224 350 1 42 0\n224 350 1 42 0"),
        source.replace(&format!("U1.1 U2.1\n{witness}\n"), ""),
        source.replace("*END*", &format!("U1.1 U2.1\n{witness}\n*END*")),
        source.replace("U2 TYPEA 260 300", "U2 TYPEA 200 300"),
        source.replace("*END*", "*VIA*\n*END*"),
        source.replace("*END*", "*END*\nUNSUPPORTED"),
    ];
    for (i, item) in cases.iter().enumerate() {
        assert_ne!(item, &source, "inert mutation {i}");
        assert!(audit(item).is_err(), "accepted hostile variant {i}");
    }
}
