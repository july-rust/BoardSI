//! D_048: bounded SOURCE-exact round SMD pad to finite-width trace, not electrical continuity.
use boardsi_import::d048_trace_round_pad_width;
const SOURCE: &str = include_str!("fixtures/d048_trace_round_pad_width_v94.asc");

fn audit(s: &str) -> Result<String, &'static str> {
    d048_trace_round_pad_width::encode(s.as_bytes())
}

#[test]
fn positive_round_pad_candidate_has_exact_graph_and_scope() {
    let wire = audit(SOURCE).expect("supported round-pad SOURCE");
    assert!(wire.starts_with("BOARDSI_D048_TRACE_ROUND_PAD_WIDTH_V1|"));
    assert!(wire.contains("C|0|U1|NET_A|1|5|4\n"));
    assert!(wire.contains("C|1|U2|NET_A|1|4|3\n"));
    assert_eq!(wire.matches("\nP|").count(), 4);
    assert_eq!(wire.matches("\nT|").count(), 5);
    assert_eq!(wire.matches("\nE|").count(), 7);
    assert!(wire.contains("E|6|0|8|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_CONTACT\n"));
    assert!(wire.contains("COUNT|NODES|9|EDGES|7|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED"));
    assert!(wire.ends_with("END|SOURCE_LOCAL_TRACE_ROUND_PAD_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n"));
}

#[test]
fn round_cap_reverse_net_header_and_whitespace_parity() {
    let s = SOURCE.replace("\r\n", "\n");
    let base = "222 285 1 8 0\n222 315 1 8 0";
    let variants = [
        s.replace(base, "217 316 1 8 0\n220 319 1 8 0"),
        s.replace(base, "222 315 1 8 0\n222 285 1 8 0"),
        s.replace(base, "223.99 285 1 8 0\n223.99 315 1 8 0"),
        s.replace("NET_A", "NET_B"),
        s.replace('\n', "\r\n"),
        s.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A"),
        s.replace("!PADS-POWERPCB-V9.4-MILS!", "!PADS-POWERPCB-V9.4-MILS-CP1252!"),
    ];
    for (i, v) in variants.iter().enumerate() {
        assert_ne!(v, &s, "inert supported variant {i}");
        assert!(audit(v).is_ok(), "supported variant {i}");
    }
}

#[test]
fn tangent_gap_inside_aabb_wrong_shape_and_hostile_sources_fail_closed() {
    let s = SOURCE.replace("\r\n", "\n");
    let base = "222 285 1 8 0\n222 315 1 8 0";
    let cases = [
        s.replace(base, "224 285 1 8 0\n224 315 1 8 0"),
        s.replace(base, "224.01 285 1 8 0\n224.01 315 1 8 0"),
        s.replace(base, "225 285 1 8 0\n225 315 1 8 0"),
        s.replace(base, "219 285 1 8 0\n219 315 1 8 0"),
        s.replace(base, "222 322 1 8 0\n230 330 1 8 0"),
        s.replace("U1.1 U2.1", "U1.2 U2.2"),
        s.replace("222 285 1 8 0", "222 285 2 8 0"),
        s.replace("222 285 1 8 0", "222 285 1 0 0"),
        s.replace("222 285 1 8 0", "222 285 1 20 0"),
        s.replace("PAD 1 3\n-2 40 R 0", "PAD 1 3\n-2 40 S 0"),
        s.replace("U2 TYPEA 400 300", "U2 TYPEA 260 300"),
        s.replace("*SIGNAL* NET_A", "*SIGNAL* NET_C"),
        s.replace(base, "222 285 1 8 0\n222 285 1 8 0"),
        s.replace(&format!("U1.1 U2.1\n{base}\n"), ""),
        s.replace("*END*", &format!("U1.1 U2.1\n{base}\n*END*")),
        s.replace("*END*", "*VIA*\n*END*"),
        s.replace("*END*", "*END*\nFORGED"),
    ];
    for (i, candidate) in cases.iter().enumerate() {
        assert_ne!(candidate, &s, "inert hostile mutation {i}");
        assert!(audit(candidate).is_err(), "hostile round-pad case {i}");
    }
}
