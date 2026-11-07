//! D_052 real SOURCE split traces and local finite-width copper gap.
use boardsi_import::d052_bridge_midspan_break;
const ORIGINAL: &str = include_str!("fixtures/d051_mixed_pad_dual_finite_width_loop_v94.asc");
const BOTH: &str = include_str!("fixtures/d052_dual_bridge_midspan_break_both_v94.asc");
const A: &str = "U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0";
const B: &str = "U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0";
const SPLIT_A: &str = "U1.1 U2.1\n226 300 1 8 0\n290 300 1 8 0\nU1.1 U2.1\n310 300 1 8 0\n378 300 1 8 0";
const SPLIT_B: &str = "U1.2 U2.2\n222 400 1 8 0\n290 400 1 8 0\nU1.2 U2.2\n310 400 1 8 0\n382 400 1 8 0";
fn audit(s: &str) -> Result<String, &'static str> {
    d052_bridge_midspan_break::encode(s.as_bytes())
}
#[test]
fn real_source_breaks_and_graph_are_independent_of_virtual_node_deletions() {
    let cases = [
        (ORIGINAL.to_owned(), "INTACT", "12|12|1|1|REACHABLE", 8),
        (ORIGINAL.replace(A, SPLIT_A), "BREAK_A", "13|12|1|0|REACHABLE", 9),
        (ORIGINAL.replace(B, SPLIT_B), "BREAK_B", "13|12|1|0|REACHABLE", 9),
        (BOTH.to_owned(), "BREAK_BOTH", "14|12|2|0|SEPARATED", 10),
    ];
    assert_eq!(ORIGINAL.replace(A, SPLIT_A).replace(B, SPLIT_B), BOTH);
    for (src, state, counts, tracks) in cases {
        let result = audit(&src).unwrap_or_else(|reason| panic!("{state}: {reason}"));
        assert!(result.starts_with("BOARDSI_D052_DUAL_BRIDGE_MIDSPAN_BREAK_V1|"));
        assert!(result.contains("GLOBAL_ELECTRICAL_NOT_VERIFIED"));
        assert!(result.contains(&format!("D|{state}|{counts}\n")));
        assert!(result.contains("E|9|2|"));
        assert!(result.contains("E|11|3|"));
        assert_eq!(result.lines().filter(|s| s.starts_with("P|")).count(), 4);
        assert_eq!(result.lines().filter(|s| s.starts_with("T|")).count(), tracks);
        assert_eq!(result.lines().filter(|s| s.starts_with("E|")).count(), 12);
        assert!(result.ends_with("END|SOURCE_LOCAL_DUAL_BRIDGE_MIDSPAN_BREAK_ONLY_NO_GLOBAL_PASS\n"));
    }
    let full = audit(BOTH).unwrap();
    assert!(full.contains("S|A|BREAK|CENTER_GAP_TICKS|2000|COPPER_GAP_TICKS|1200\n"));
    assert!(full.contains("S|B|BREAK|CENTER_GAP_TICKS|2000|COPPER_GAP_TICKS|1200\n"));
    assert!(full.contains("T|8|1|NET_A|31000|30000|37800|30000|800\n"));
    assert!(full.contains("T|9|1|NET_A|31000|40000|38200|40000|800\n"));
    assert!(full.contains("E|9|2|12|"));
    assert!(full.contains("E|11|3|13|"));
}
#[test]
fn positive_orientation_net_and_one_tick_copper_separation() {
    let reverse_a = "U1.1 U2.1\n290 300 1 8 0\n226 300 1 8 0\nU1.1 U2.1\n378 300 1 8 0\n310 300 1 8 0";
    let reverse_b = "U1.2 U2.2\n290 400 1 8 0\n222 400 1 8 0\nU1.2 U2.2\n382 400 1 8 0\n310 400 1 8 0";
    let mut cases = vec![
        BOTH.replace(SPLIT_A, reverse_a).replace(SPLIT_B, reverse_b),
        BOTH.replace("NET_A", "NET_B"),
        BOTH.replace('\n', "\r\n"),
        BOTH.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A"),
        BOTH.replace("226 300 1 8 0", "+226.00 300.00 1 8.00 0"),
        BOTH.replace("290 300 1 8 0", "300 300 1 8 0")
            .replace("310 300 1 8 0", "308.01 300 1 8 0"),
        BOTH.replace("290 400 1 8 0", "300 400 1 8 0")
            .replace("310 400 1 8 0", "308.01 400 1 8 0"),
    ];
    cases.retain(|s| s != BOTH);
    for (i, s) in cases.iter().enumerate() {
        assert!(audit(s).is_ok(), "positive {i}: {:?}", audit(s).err());
        assert_ne!(audit(s).unwrap(), audit(BOTH).unwrap());
    }
}
#[test]
fn hostile_gap_collision_ownership_inventory_and_stale_receipts() {
    let mut cases = vec![
        BOTH.replace("310 300 1 8 0", "298 300 1 8 0"), // tangent
        BOTH.replace("310 300 1 8 0", "297.99 300 1 8 0"), // overlap
        BOTH.replace("310 400 1 8 0", "298 400 1 8 0"),
        BOTH.replace("310 400 1 8 0", "297.99 400 1 8 0"),
        BOTH.replace("290 300 1 8 0", "270 300 1 8 0"),
        BOTH.replace("290 400 1 8 0", "270 400 1 8 0"),
        BOTH.replace("226 300 1 8 0", "230 300 1 8 0"),
        BOTH.replace("382 400 1 8 0", "379 400 1 8 0"),
        BOTH.replace("310 400 1 8 0", "310 401 1 8 0"),
        BOTH.replace("310 300 1 8 0", "310 300 2 8 0"),
        BOTH.replace("310 400 1 8 0", "310 400 1 10 0"),
        BOTH.replace("U1.1 U2.1\n310", "U1.2 U2.1\n310"),
        BOTH.replace("U1.2 U2.2\n310", "U1.2 U2.1\n310"),
        BOTH.replace("*SIGNAL* NET_A", "*SIGNAL* NET_C"),
        BOTH.replace("PAD 2 3\n-2 30 S 0", "PAD 2 3\n-2 30 R 0"),
        BOTH.replace("U2 TYPEA 400 300", "U2 TYPEA 398 300"),
        BOTH.replace("U1.2 U2.2\n310 400 1 8 0\n382 400 1 8 0\n", ""),
        BOTH.replace("*END*", "U1.1 U2.1\n310 300 1 8 0\n378 300 1 8 0\n*END*"),
        BOTH.replace("*END*", "U1.1 U2.1\n290 300 1 8 0\n310 300 1 8 0\n*END*"),
        BOTH.replace("*END*", "FORGED\n*END*"),
    ];
    cases.push(BOTH.replace("310 300 1 8 0", "310 300 1 8 0\nU1.1 U2.1\n320 300 1 8 0\n330 300 1 8 0"));
    for (i, source) in cases.iter().enumerate() {
        assert_ne!(source, BOTH, "inert negative {i}");
        assert!(audit(source).is_err(), "hostile {i} silently accepted");
    }
    // Different valid bytes imply different SHA-bound diagnostics.
    let old = audit(BOTH).unwrap();
    let newer = audit(&BOTH.replace("NET_A", "NET_B")).unwrap();
    assert_ne!(old, newer);
}
