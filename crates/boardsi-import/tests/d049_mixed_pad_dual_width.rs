//! D_049 strict two-contact SOURCE composition and adversarial gate.
use boardsi_import::d049_mixed_pad_dual_width;
const SOURCE: &str = include_str!("fixtures/d049_mixed_round_square_dual_width_v94.asc");

fn audit(source: &str) -> Result<String, &'static str> {
    d049_mixed_pad_dual_width::encode(source.as_bytes())
}

#[test]
fn combined_pad_contacts_are_one_source_bound_graph() {
    let wire = audit(SOURCE).expect("valid two-contact fixture");
    assert!(wire.starts_with("BOARDSI_D049_MIXED_PAD_DUAL_WIDTH_V1|"));
    assert!(wire.contains("C|0|U1|NET_A|1|6|5\n"));
    assert!(wire.contains("C|1|U2|NET_A|1|4|3\n"));
    assert_eq!(wire.matches("\nP|").count(), 4);
    assert_eq!(wire.matches("\nT|").count(), 6);
    assert_eq!(wire.matches("\nE|").count(), 8);
    assert!(wire.contains("E|6|0|8|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_CONTACT\n"));
    assert!(wire.contains("E|7|1|9|POSITIVE_FINITE_WIDTH_TRACE_PAD_CONTACT\n"));
    assert!(
        wire.contains("COUNT|NODES|10|EDGES|8|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED")
    );
    assert!(wire.ends_with("END|SOURCE_LOCAL_MIXED_PAD_DUAL_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n"));
}

#[test]
fn independent_cap_direction_net_and_source_variants() {
    let round = "222 285 1 8 0\n222 315 1 8 0";
    let square = "217 390 1 8 0\n217 410 1 8 0";
    let variants = [
        SOURCE.replace(round, "217 316 1 8 0\n220 319 1 8 0"),
        SOURCE.replace(square, "217 417 1 8 0\n225 425 1 8 0"),
        SOURCE.replace(round, "222 315 1 8 0\n222 285 1 8 0"),
        SOURCE.replace(square, "217 410 1 8 0\n217 390 1 8 0"),
        SOURCE.replace("NET_A", "NET_B"),
        SOURCE.replace('\n', "\r\n"),
        SOURCE.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A"),
        SOURCE.replace("222 285 1 8 0", "+222 285 1 8.00 128"),
        SOURCE.replace("222 315 1 8 0", "222 315 65 8 0"),
        SOURCE.replace("217 390 1 8 0", "+217 390 1 8.00 1024"),
        SOURCE.replace("217 410 1 8 0", "217 410 65 8 0"),
        SOURCE.replace(
            "!PADS-POWERPCB-V9.4-MILS!",
            "!PADS-POWERPCB-V9.4-MILS-CP1252!",
        ),
    ];
    for (i, modified) in variants.iter().enumerate() {
        assert_ne!(modified, SOURCE, "inert supported variant {i}");
        assert!(audit(modified).is_ok(), "valid variant {i}");
    }
}

#[test]
fn either_failed_contact_or_hostile_provenance_fails_entire_graph() {
    let round = "222 285 1 8 0\n222 315 1 8 0";
    let square = "217 390 1 8 0\n217 410 1 8 0";
    let cases = [
        SOURCE.replace(round, "224 285 1 8 0\n224 315 1 8 0"),
        SOURCE.replace(round, "224.01 285 1 8 0\n224.01 315 1 8 0"),
        SOURCE.replace(round, "222 322 1 8 0\n230 330 1 8 0"),
        SOURCE.replace(square, "219 390 1 8 0\n219 410 1 8 0"),
        SOURCE.replace(square, "219.01 390 1 8 0\n219.01 410 1 8 0"),
        SOURCE.replace(square, "216 419 1 8 0\n224 427 1 8 0"),
        SOURCE.replace(round, square),
        SOURCE.replace(square, round),
        SOURCE.replace("U1.1 U2.1\n222", "U1.2 U2.1\n222"),
        SOURCE.replace("U1.2 U2.2\n217", "U1.1 U2.2\n217"),
        SOURCE.replace("222 285 1 8 0", "222 285 2 8 0"),
        SOURCE.replace("217 390 1 8 0", "217 390 2 8 0"),
        SOURCE.replace("217 390 1 8 0", "217 390 1 10 0"),
        SOURCE.replace("222 285 1 8 0", "222 285 1 8.001 0"),
        SOURCE.replace("222 285 1 8 0", "222 285 65 8 0"),
        SOURCE.replace("217 390 1 8 0", "217 390 1 8 16"),
        SOURCE.replace("222 285 1 8 0", "222 285 1 8 4096"),
        SOURCE.replace("PAD 1 3\n-2 40 R 0", "PAD 1 3\n-2 40 S 0"),
        SOURCE.replace("PAD 2 3\n-2 30 S 0", "PAD 2 3\n-2 30 R 0"),
        SOURCE.replace("U2 TYPEA 400 300", "U2 TYPEA 260 300"),
        SOURCE.replace("NET_A", "NET_C"),
        SOURCE.replace(&format!("U1.1 U2.1\n{round}\n"), ""),
        SOURCE.replace(&format!("U1.2 U2.2\n{square}\n"), ""),
        SOURCE.replace("*END*", &format!("U1.2 U2.2\n{square}\n*END*")),
        SOURCE.replace("*END*", "*VIA*\n*END*"),
        SOURCE.replace("*END*", "*END*\nFORGED"),
    ];
    for (i, modified) in cases.iter().enumerate() {
        assert_ne!(modified, SOURCE, "inert hostile case {i}");
        assert!(
            audit(modified).is_err(),
            "hostile case {i} unexpectedly accepted"
        );
    }
}
