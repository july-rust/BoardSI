//! D_041 local source-bridge regressions; not global electrical Open/Short.
use boardsi_import::d041_local_bridge;

const SOURCE: &str = include_str!("fixtures/d041_local_bridge_v94.asc");
fn check(source: &str) -> Result<String, &'static str> {
    d041_local_bridge::encode(source.as_bytes())
}

#[test]
fn exact_bridge_is_one_source_local_component() {
    let receipt = check(SOURCE).unwrap();
    assert!(receipt.contains("C|0|U1+U2|NET_A|1|9|8\n"));
    assert_eq!(receipt.matches("\nP|").count(), 4);
    assert_eq!(receipt.matches("\nT|").count(), 5);
    assert_eq!(receipt.matches("\nE|").count(), 8);
    assert!(receipt.contains("E|6|4|8|EXACT_BRIDGE_JUNCTION_LOCAL_COPPER\n"));
    assert!(receipt.contains("E|7|8|6|EXACT_BRIDGE_JUNCTION_LOCAL_COPPER\n"));
    assert!(
        receipt.contains("COUNT|NODES|9|EDGES|8|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED")
    );
    assert!(receipt.ends_with("END|SOURCE_LOCAL_BRIDGE_ONLY_NO_GLOBAL_PASS\n"));
    assert_eq!(check(SOURCE).unwrap(), receipt);
}

#[test]
fn header_direction_and_same_net_lexical_variants() {
    let base = SOURCE.replace("\r\n", "\n");
    let original = check(&base).unwrap();
    let reversed = base.replace(
        "200 350 1 10 0\n400 350 1 10 0",
        "400 350 1 10 0\n200 350 1 10 0",
    );
    assert_ne!(reversed, base);
    assert_eq!(
        original.split_once('\n').unwrap().1,
        check(&reversed).unwrap().split_once('\n').unwrap().1
    );
    for header in [
        "!PADS-POWERPCB-V9.4-MILS-CP1252!",
        "!PADS-POWERPCB-V9.4-MILS-250L!",
        "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!",
    ] {
        let variant = base.replace("!PADS-POWERPCB-V9.4-MILS!", header);
        assert_eq!(
            original.split_once('\n').unwrap().1,
            check(&variant).unwrap().split_once('\n').unwrap().1
        );
    }
    let crlf = base.replace("\n", "\r\n");
    assert!(check(&crlf).is_ok());
    let tokenized = base
        .replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A")
        .replace("U1.1 U2.1\n", "U1.1\tU2.1\n");
    assert!(check(&tokenized).is_ok());
    let net_b = base.replace("NET_A", "NET_B");
    assert!(check(&net_b).unwrap().contains("C|0|U1+U2|NET_B|1|9|8\n"));
    let tabs = base.replace(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN",
        "!PADS-POWERPCB-V9.4-MILS!\tDESIGN",
    );
    assert!(check(&tabs).is_ok());
}

#[test]
fn absent_misaligned_wrong_net_or_unsupported_bridge_fails_closed() {
    let base = SOURCE.replace("\r\n", "\n");
    let failures = [
        base.replace("400 350 1 10 0\n*END*", "400 350.01 1 10 0\n*END*"),
        base.replace(
            "200 350 1 10 0\n400 350 1 10 0\n*END*",
            "200 350 2 10 0\n400 350 2 10 0\n*END*",
        ),
        base.replace(
            "200 350 1 10 0\n400 350 1 10 0\n*END*",
            "200 350 1 11 0\n400 350 1 11 0\n*END*",
        ),
        base.replace("U1.1 U2.1\n200 350", "U1.2 U2.1\n200 350"),
        base.replace("U1.1 U2.1\n200 350 1 10 0\n400 350 1 10 0\n", ""),
        base.replace("*END*", "U1.1 U2.1\n200 350 1 10 0\n400 350 1 10 0\n*END*"),
        base.replace("U2 TYPEA 400 300", "U2 TYPEA 200 300"),
        base.replace("U2 TYPEA 400 300", "U2 TYPEA 400 300.01"),
        base.replace("*ROUTE*", "*VIA*\nSIMPLEVIA 10 2\n*ROUTE*"),
        base.replace("*ROUTE*", "*LINES*\nRA COPCLS 0 0 1 1 0 NET_A\n*ROUTE*"),
        base.replace("U1.1 U2.1", "U1.1 U1.2"),
        base.replace("U1.1 U2.1", "*SIGNAL* NET_B\nU1.1 U2.1"),
        base.replace(
            "U2.1 U2.2\n400 300 1 10 0",
            "*SIGNAL* NET_B\nU2.1 U2.2\n400 300 1 10 0",
        ),
        base.replace("*END*", "*END*\n!PADS-POWERPCB-V9.4-MILS! DESIGN"),
    ];
    for (i, changed) in failures.iter().enumerate() {
        assert_ne!(changed, &base, "mutation {i} made no change");
        assert!(check(changed).is_err(), "mutation {i} bypassed fail closed");
    }
}
