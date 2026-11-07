//! D_050 SOURCE-bounded positive area bridge regression; not global electrical proof.
use boardsi_import::d050_mixed_pad_width_bridge;
const SOURCE: &str = include_str!("fixtures/d050_mixed_pad_finite_width_bridge_v94.asc");
fn audit(s: &str) -> Result<String, &'static str> {
    d050_mixed_pad_width_bridge::encode(s.as_bytes())
}
const BRIDGE: &str = "U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0";
#[test]
fn one_exact_source_graph_with_all_mixed_pad_contacts() {
    let wire = audit(SOURCE).expect("valid bounded D050");
    assert!(wire.starts_with("BOARDSI_D050_MIXED_PAD_WIDTH_BRIDGE_V1|"));
    assert!(wire.contains("C|0|U1_U2|NET_A|1|11|10\n"));
    assert_eq!(wire.matches("\nC|").count(), 1);
    assert_eq!(wire.matches("\nP|").count(), 4);
    assert_eq!(wire.matches("\nT|").count(), 7);
    assert_eq!(wire.matches("\nE|").count(), 10);
    assert!(wire.contains("T|6|1|NET_A|22600|30000|37800|30000|800\n"));
    assert!(wire.contains("E|8|8|10|POSITIVE_FINITE_WIDTH_TRACE_TRACE_BRIDGE\n"));
    assert!(wire.contains("E|9|2|10|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_BRIDGE\n"));
    assert!(wire.contains("COUNT|NODES|11|EDGES|10|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"));
    assert!(wire.ends_with("END|SOURCE_LOCAL_MIXED_PAD_FINITE_WIDTH_BRIDGE_ONLY_NO_GLOBAL_PASS\n"));
}
#[test]
fn supported_lexical_orientation_net_and_one_tick_positive_contacts() {
    let variants = [
        SOURCE.replace(BRIDGE, "U1.1 U2.1\n378 300 1 8 0\n226 300 1 8 0"),
        SOURCE.replace("NET_A", "NET_B"),
        SOURCE.replace('\n', "\r\n"),
        SOURCE.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A"),
        SOURCE.replace(BRIDGE, "U1.1 U2.1\n+226 300 1 8.00 128\n378 300 65 8 0"),
        SOURCE.replace(BRIDGE, "U1.1 U2.1\n000226 300 1 8 0\n0378 300 1 8 0"),
        SOURCE.replace("226 300 1 8 0", "229.99 300 1 8 0"),
        SOURCE.replace("378 300 1 8 0", "376.01 300 1 8 0"),
        SOURCE.replace("226 300 1 8 0", "224.01 300 1 8 0"),
        SOURCE.replace("378 300 1 8 0", "379.99 300 1 8 0"),
    ];
    for (i, v) in variants.iter().enumerate() {
        assert_ne!(v, SOURCE, "inert positive test {i}");
        assert!(audit(v).is_ok(), "supported variant {i} failed");
    }
}
#[test]
fn tangency_gap_cross_net_ambiguity_and_forged_inventory_fail_closed() {
    let cases = [
        SOURCE.replace("226 300 1 8 0", "230 300 1 8 0"),
        SOURCE.replace("226 300 1 8 0", "230.01 300 1 8 0"),
        SOURCE.replace("226 300 1 8 0", "224 300 1 8 0"),
        SOURCE.replace("378 300 1 8 0", "376 300 1 8 0"),
        SOURCE.replace("378 300 1 8 0", "375.99 300 1 8 0"),
        SOURCE.replace("378 300 1 8 0", "380 300 1 8 0"),
        SOURCE.replace("226 300 1 8 0", "226 322 1 8 0")
            .replace("378 300 1 8 0", "378 322 1 8 0"),
        SOURCE.replace("226 300 1 8 0", "226 300 2 8 0"),
        SOURCE.replace("226 300 1 8 0", "226 300 1 10 0"),
        SOURCE.replace("226 300 1 8 0", "226 300 1 8.001 0"),
        SOURCE.replace("226 300 1 8 0", "226 300 1 8 16"),
        SOURCE.replace(BRIDGE, "U1.2 U2.1\n226 300 1 8 0\n378 300 1 8 0"),
        SOURCE.replace("PAD 1 3\n-2 40 R 0", "PAD 1 3\n-2 40 S 0"),
        SOURCE.replace("PAD 2 3\n-2 30 S 0", "PAD 2 3\n-2 30 R 0"),
        SOURCE.replace("U2 TYPEA 400 300", "U2 TYPEA 398 300"),
        SOURCE.replace("NET_A", "NET_C"),
        SOURCE.replace(&format!("{BRIDGE}\n"), ""),
        SOURCE.replace("*END*", &format!("{BRIDGE}\n*END*")),
        SOURCE.replace("*END*", "U1.2 U2.2\n240 300 1 8 0\n360 300 1 8 0\n*END*"),
        SOURCE.replace("*END*", "*VIA*\n*END*"),
        SOURCE.replace("*END*", "*END*\nFORGED"),
    ];
    for (i, v) in cases.iter().enumerate() {
        assert_ne!(v, SOURCE, "inert hostile case {i}");
        assert!(audit(v).is_err(), "hostile case {i} passed");
    }
}
