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
    let source = SOURCE.replace("\r\n", "\n");
    let round = "222 285 1 8 0\n222 315 1 8 0";
    let square = "217 390 1 8 0\n217 410 1 8 0";
    let variants = [
        source.replace(round, "217 316 1 8 0\n220 319 1 8 0"),
        source.replace(square, "217 417 1 8 0\n225 425 1 8 0"),
        source.replace(round, "222 315 1 8 0\n222 285 1 8 0"),
        source.replace(square, "217 410 1 8 0\n217 390 1 8 0"),
        source.replace("NET_A", "NET_B"),
        source.replace('\n', "\r\n"),
        source.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A"),
        source.replace("222 285 1 8 0", "+222 285 1 8.00 128"),
        source.replace("222 285 1 8 0", "000222 285 1 8 0"),
        source.replace("222 315 1 8 0", "222 315 65 8 0"),
        source.replace("217 390 1 8 0", "+217 390 1 8.00 1024"),
        source.replace("217 410 1 8 0", "217 410 65 8 0"),
        source.replace(
            "!PADS-POWERPCB-V9.4-MILS!",
            "!PADS-POWERPCB-V9.4-MILS-CP1252!",
        ),
    ];
    for (i, modified) in variants.iter().enumerate() {
        assert_ne!(modified, &source, "inert supported variant {i}");
        assert!(audit(modified).is_ok(), "valid variant {i}");
    }
}

#[test]
fn either_failed_contact_or_hostile_provenance_fails_entire_graph() {
    let source = SOURCE.replace("\r\n", "\n");
    let round = "222 285 1 8 0\n222 315 1 8 0";
    let square = "217 390 1 8 0\n217 410 1 8 0";
    let cases = [
        source.replace(round, "224 285 1 8 0\n224 315 1 8 0"),
        source.replace(round, "224.01 285 1 8 0\n224.01 315 1 8 0"),
        source.replace(round, "222 322 1 8 0\n230 330 1 8 0"),
        source.replace(square, "219 390 1 8 0\n219 410 1 8 0"),
        source.replace(square, "219.01 390 1 8 0\n219.01 410 1 8 0"),
        source.replace(square, "216 419 1 8 0\n224 427 1 8 0"),
        source.replace(round, square),
        source.replace(square, round),
        source.replace("U1.1 U2.1\n222", "U1.2 U2.1\n222"),
        source.replace("U1.2 U2.2\n217", "U1.1 U2.2\n217"),
        source.replace("222 285 1 8 0", "222 285 2 8 0"),
        source.replace("217 390 1 8 0", "217 390 2 8 0"),
        source.replace("217 390 1 8 0", "217 390 1 10 0"),
        source.replace("222 285 1 8 0", "222 285 1 8.001 0"),
        source.replace("222 285 1 8 0", "222 285 65 8 0"),
        source.replace("217 390 1 8 0", "217 390 1 8 16"),
        source.replace("222 285 1 8 0", "222 285 1 8 4096"),
        source.replace("PAD 1 3\n-2 40 R 0", "PAD 1 3\n-2 40 S 0"),
        source.replace("PAD 2 3\n-2 30 S 0", "PAD 2 3\n-2 30 R 0"),
        source.replace("U2 TYPEA 400 300", "U2 TYPEA 260 300"),
        source.replace("NET_A", "NET_C"),
        source.replace(&format!("U1.1 U2.1\n{round}\n"), ""),
        source.replace(&format!("U1.2 U2.2\n{square}\n"), ""),
        source.replace("*END*", &format!("U1.2 U2.2\n{square}\n*END*")),
        source.replace("*END*", "*VIA*\n*END*"),
        source.replace("*END*", "*END*\nFORGED"),
    ];
    for (i, modified) in cases.iter().enumerate() {
        assert_ne!(modified, &source, "inert hostile case {i}");
        assert!(
            audit(modified).is_err(),
            "hostile case {i} unexpectedly accepted"
        );
    }
}
