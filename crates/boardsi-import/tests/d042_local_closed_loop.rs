//! D_042 SOURCE-only exact closed-loop regression; no physical Open/Short claim.
use boardsi_import::d042_local_closed_loop;

const SOURCE: &str = include_str!("fixtures/d042_local_closed_loop_v94.asc");

fn check(source: &str) -> Result<String, &'static str> {
    d042_local_closed_loop::encode(source.as_bytes())
}

#[test]
fn exact_detour_creates_one_local_component_and_one_cycle() {
    let receipt = check(SOURCE).expect("exact D042 synthetic fixture");
    assert!(receipt.starts_with("BOARDSI_D042_LOCAL_CLOSED_LOOP_V1|"));
    assert!(receipt.contains("\nC|0|U1+U2|NET_A|1|14|14\n"));
    assert_eq!(receipt.matches("\nP|").count(), 4);
    assert_eq!(receipt.matches("\nT|").count(), 10);
    assert_eq!(receipt.matches("\nE|").count(), 14);
    assert!(receipt.contains("E|8|4|9|EXACT_DETOUR_JUNCTION_LOCAL_COPPER\n"));
    assert!(receipt.contains("E|13|13|6|EXACT_DETOUR_JUNCTION_LOCAL_COPPER\n"));
    assert!(receipt.contains("CYCLE|LOCAL|1|INDEPENDENT_PATHS|2\n"));
    assert!(receipt.contains("COUNT|NODES|14|EDGES|14|LOCAL_COMPONENTS|1|LOCAL_CYCLES|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"));
    assert!(receipt.ends_with("END|SOURCE_LOCAL_CLOSED_LOOP_ONLY_NO_GLOBAL_PASS\n"));
    assert_eq!(receipt, check(SOURCE).expect("deterministic same SOURCE"));
}

#[test]
fn reversed_detour_header_whitespace_and_net_variants() {
    let base = SOURCE.replace("\r\n", "\n");
    let original = check(&base).expect("base");
    let reverse = base.replace(
        "U1.2 U2.2\n200 350 1 10 0\n150 350 1 10 0",
        "U1.2 U2.2\n150 350 1 10 0\n200 350 1 10 0",
    );
    assert_ne!(reverse, base);
    assert_eq!(
        original.split_once('\n').unwrap().1,
        check(&reverse).unwrap().split_once('\n').unwrap().1
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
    assert!(check(&base.replace('\n', "\r\n")).is_ok());
    let tabs = base.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A")
        .replace("U1.2 U2.2\n", "U1.2\tU2.2\n");
    assert!(check(&tabs).is_ok());
    let other_net = base.replace("NET_A", "NET_B");
    assert!(check(&other_net).unwrap().contains("C|0|U1+U2|NET_B|1|14|14"));
}

#[test]
fn missing_duplicate_gap_width_layer_owner_and_unsupported_are_fail_closed() {
    let source = SOURCE.replace("\r\n", "\n");
    let fail = [
        source.replace("400 350 1 10 0\n*END*", "400 350.01 1 10 0\n*END*"),
        source.replace("400 350 1 10 0\n*END*", "400 350 2 10 0\n*END*"),
        source.replace("400 350 1 10 0\n*END*", "400 350 1 11 0\n*END*"),
        source.replace("U1.2 U2.2\n200 350 1 10 0\n150 350 1 10 0\n", ""),
        source.replace("*END*", "U1.2 U2.2\n200 350 1 10 0\n150 350 1 10 0\n*END*"),
        source.replace("U1.2 U2.2\n200 350", "U1.1 U2.2\n200 350"),
        source.replace("150 250 1 10 0\n450 250 1 10 0",
                       "150 250 1 10 0\n450 249.99 1 10 0"),
        source.replace("U2 TYPEA 400 300", "U2 TYPEA 399.99 300"),
        source.replace("*ROUTE*", "*VIA*\nSIMPLEVIA 10 2\n*ROUTE*"),
        source.replace("*ROUTE*", "*LINES*\nRA COPCLS 0 0 1 1 0 NET_A\n*ROUTE*"),
        source.replace("U1.2 U2.2\n450 350", "*SIGNAL* NET_B\nU1.2 U2.2\n450 350"),
        source.replace("150 350 1 10 0\nU1.2 U2.2",
                       "150 351 1 10 0\nU1.2 U2.2"),
    ];
    for (i, mutated) in fail.iter().enumerate() {
        assert_ne!(mutated, &source, "mutation {i} was inert");
        assert!(check(mutated).is_err(), "mutation {i} bypassed fail-closed");
    }
}
