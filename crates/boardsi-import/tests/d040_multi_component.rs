//! D_040 bounded same-layer disconnected-component source validation.
use boardsi_import::d040_multi_component;

const SOURCE: &str = include_str!("fixtures/d040_multi_component_v94.asc");
fn check(s: &str) -> Result<String, &'static str> {
    d040_multi_component::encode(s.as_bytes())
}
fn mixed(s: &str) -> String {
    let marker = "U2.1 U2.2\n400 300 1 10 0";
    assert!(s.contains(marker));
    s.replacen(marker, "*SIGNAL* NET_B\nU2.1 U2.2\n400 300 1 10 0", 1)
}

#[test]
fn same_net_two_disconnected_components_do_not_merge() {
    let out = check(SOURCE).expect("source-owned isolated same-Net paths");
    assert!(out.contains("C|0|U1|NET_A|1|4|3\n"));
    assert!(out.contains("C|1|U2|NET_A|1|4|3\n"));
    assert_eq!(out.matches("\nP|").count(), 4);
    assert_eq!(out.matches("\nT|").count(), 4);
    assert_eq!(out.matches("\nE|").count(), 6);
    assert!(out.contains("COUNT|NODES|8|EDGES|6|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED"));
    assert!(out.ends_with("END|LOCAL_COPPER_COMPONENTS_ONLY_NO_GLOBAL_PASS\n"));
    assert_eq!(out, check(SOURCE).unwrap());
}
#[test]
fn mixed_net_components_are_isolated_not_short_or_open_certificates() {
    let changed = mixed(SOURCE);
    let out = check(&changed).expect("two distinct source-owned nets");
    assert!(out.contains("C|0|U1|NET_A|1|4|3\n"));
    assert!(out.contains("C|1|U2|NET_B|1|4|3\n"));
    assert!(out.contains("T|2|1|NET_B|"));
    assert!(out.contains("|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"));
    assert_ne!(out.lines().next(), check(SOURCE).unwrap().lines().next());
}
#[test]
fn supported_headers_reversal_and_lexical_changes_preserve_only_topology() {
    let base = SOURCE.replace("\r\n", "\n");
    let original = check(SOURCE).unwrap();
    for header in [
        "!PADS-POWERPCB-V9.4-MILS-CP1252!",
        "!PADS-POWERPCB-V9.4-MILS-250L!",
        "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!",
    ] {
        let var = base.replace("!PADS-POWERPCB-V9.4-MILS!", header);
        let wire = check(&var).unwrap();
        assert_eq!(
            wire.split_once('\n').unwrap().1,
            original.split_once('\n').unwrap().1
        );
    }
    let rev = base.replace(
        "400 300 1 10 0\n400 350 1 10 0",
        "400 350 1 10 0\n400 300 1 10 0",
    );
    assert_ne!(rev, base);
    assert_eq!(
        check(&rev).unwrap().split_once('\n').unwrap().1,
        original.split_once('\n').unwrap().1
    );
    let tabbed = base.replace(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN",
        "!PADS-POWERPCB-V9.4-MILS!\tDESIGN",
    );
    assert!(check(&tabbed).is_ok());
    assert_ne!(
        original.lines().next(),
        check(&tabbed).unwrap().lines().next()
    );
}
#[test]
fn gap_owner_layer_cross_net_collision_and_unsupported_source_fail_closed() {
    let base = SOURCE.replace("\r\n", "\n");
    let bad = [
        base.replace(
            "400 400 1 10 0\n400 350 1 10 0",
            "400 400 1 10 0\n400 350.01 1 10 0",
        ),
        base.replace("U2 TYPEA 400 300", "U2 TYPEA 200 300"),
        base.replace("U2 TYPEA 400 300", "U2 TYPEA 200.01 300"),
        base.replace("400 350 1 10 0", "400 350 2 10 0"),
        base.replace("400 350 1 10 0", "400 350 1 11 0"),
        base.replace("U2.1 U2.2", "U1.1 U1.2"),
        base.replace(
            "U2 TYPEA 400 300 90 U N 0 -1 0 -1 0",
            "U1 TYPEA 400 300 90 U N 0 -1 0 -1 0",
        ),
        base.replace("*ROUTE*", "*VIA*\nSIMPLEVIA 10 2\n*ROUTE*"),
        base.replace("*ROUTE*", "*LINES*\nRA COPCLS 0 0 1 1 0 NET_A\n*ROUTE*"),
        base.replace("*PART*\n", "*PART*\nU3 TYPEA 500 300 90 U N 0 -1 0 -1 0\n"),
        base.replace("*END*", "*SIGNAL* NET_B\n*END*"),
        base.replace("*PARTTYPE*", "D2 I 1000 1000 0 2 2 0 0\n*PARTTYPE*"),
        base.replace("T100 0 0 0 2", "T101 0 0 0 2"),
    ];
    for (i, s) in bad.iter().enumerate() {
        assert_ne!(s, &base, "mutation {i} did not change SOURCE");
        assert!(check(s).is_err(), "mutation {i} bypassed Fail-Closed");
    }
    let collision = mixed(&base).replace("U2 TYPEA 400 300", "U2 TYPEA 200 300");
    assert!(
        check(&collision).is_err(),
        "cross Net collision wrongly passed"
    );
}
