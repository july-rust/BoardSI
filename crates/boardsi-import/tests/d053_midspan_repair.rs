//! D_053 SOURCE authored dual midspan repair: graph and fail-closed receipt.
use boardsi_import::d053_midspan_repair;
const BOTH: &str = include_str!("fixtures/d052_dual_bridge_midspan_break_both_v94.asc");
const INTACT: &str = include_str!("fixtures/d051_mixed_pad_dual_finite_width_loop_v94.asc");
const A: &str = "U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0";
const B: &str = "U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0";
const CUT_A: &str = "U1.1 U2.1\n226 300 1 8 0\n290 300 1 8 0\nU1.1 U2.1\n310 300 1 8 0\n378 300 1 8 0";
const CUT_B: &str = "U1.2 U2.2\n222 400 1 8 0\n290 400 1 8 0\nU1.2 U2.2\n310 400 1 8 0\n382 400 1 8 0";
const RA: &str = "U1.1 U2.1\n288 300 1 8 0\n312 300 1 8 0\n";
const RB: &str = "U1.2 U2.2\n288 400 1 8 0\n312 400 1 8 0\n";
fn insert(base: &str, repair: &str) -> String {
    base.replace("\r\n", "\n").replace("*END*", &format!("{repair}*END*"))
}
fn wire(s: &str) -> Result<String, &'static str> {
    d053_midspan_repair::encode(s.as_bytes())
}
#[test]
fn actual_source_repair_states_and_graph() {
    let both = BOTH.replace("\r\n", "\n");
    let original = INTACT.replace("\r\n", "\n");
    let cases = [
        (both.clone(), "BREAK_BOTH", "14|12|2|0|SEPARATED", 10, 12),
        (insert(&both, RA), "REPAIR_A_ONLY", "15|14|1|0|REACHABLE", 11, 14),
        (insert(&both, RB), "REPAIR_B_ONLY", "15|14|1|0|REACHABLE", 11, 14),
        (insert(&both, &format!("{RA}{RB}")), "REPAIR_BOTH", "16|16|1|1|REACHABLE", 12, 16),
        (insert(&original.replace(A, CUT_A), RA), "BREAK_A_REPAIR_A", "14|14|1|1|REACHABLE", 10, 14),
        (insert(&original.replace(B, CUT_B), RB), "BREAK_B_REPAIR_B", "14|14|1|1|REACHABLE", 10, 14),
    ];
    for (src, state, graph, tracks, edges) in cases {
        let output = wire(&src).unwrap_or_else(|e| panic!("{state}: {e}"));
        assert!(output.starts_with("BOARDSI_D053_SOURCE_MIDSPAN_REPAIR_V1|"));
        assert!(output.contains("GLOBAL_ELECTRICAL_NOT_VERIFIED"));
        assert!(output.contains(&format!("D|{state}|{graph}\n")), "{state}");
        assert_eq!(output.lines().filter(|s| s.starts_with("P|")).count(), 4);
        assert_eq!(output.lines().filter(|s| s.starts_with("T|")).count(), tracks);
        assert_eq!(output.lines().filter(|s| s.starts_with("E|")).count(), edges);
        assert!(output.ends_with("END|SOURCE_LOCAL_DUAL_BRIDGE_REPAIR_ONLY_NO_GLOBAL_PASS\n"));
    }
}
#[test]
fn reversed_orientation_net_variant_crlf_and_one_tick_positive_contact() {
    let both = BOTH.replace("\r\n", "\n");
    let full = insert(&both, &format!("{RA}{RB}"));
    let reverse = full.replace(
        RA.trim_end(),
        "U1.1 U2.1\n312 300 1 8 0\n288 300 1 8 0",
    ).replace(
        RB.trim_end(),
        "U1.2 U2.2\n312 400 1 8 0\n288 400 1 8 0",
    );
    let narrow = full.replace("288 300 1 8 0", "297.99 300 1 8 0")
        .replace("312 300 1 8 0", "302.01 300 1 8 0")
        .replace("288 400 1 8 0", "297.99 400 1 8 0")
        .replace("312 400 1 8 0", "302.01 400 1 8 0");
    for src in [
        reverse,
        full.replace("NET_A", "NET_B"),
        full.replace('\n', "\r\n"),
        narrow,
    ] {
        let out = wire(&src).unwrap_or_else(|e| panic!("positive rejected: {e}"));
        assert!(out.contains("D|REPAIR_BOTH|16|16|1|1|REACHABLE\n"));
        assert_ne!(out, wire(&full).unwrap());
    }
}
#[test]
fn missing_or_tangent_contacts_and_hostile_inventory_fail_closed() {
    let both = BOTH.replace("\r\n", "\n");
    let full = insert(&both, &format!("{RA}{RB}"));
    let hostile = [
        full.replace("288 300 1 8 0", "298 300 1 8 0"), // left tangent
        full.replace("312 300 1 8 0", "302 300 1 8 0"), // right tangent
        full.replace("288 400 1 8 0", "298.01 400 1 8 0"), // left gap
        full.replace("312 400 1 8 0", "301.99 400 1 8 0"), // right gap
        full.replace("288 300 1 8 0", "288 301 1 8 0"), // off-axis
        full.replace("312 300 1 8 0", "312 300 2 8 0"), // wrong layer
        full.replace("312 300 1 8 0", "312 300 1 9 0"), // wrong width
        full.replace("U1.1 U2.1\n288", "U1.2 U2.1\n288"), // wrong owner
        full.replace("NET_A", "NET_C"), // net oracle cannot claim unsupported net
        full.replace("PAD 2 3\n-2 30 S 0", "PAD 2 3\n-2 30 R 0"),
        full.replace("U2 TYPEA 400 300", "U2 TYPEA 398 300"),
        full.replace("*END*", &format!("{RA}*END*")), // duplicate
        full.replace("*END*", "U1.1 U2.1\n260 300 1 8 0\n265 300 1 8 0\n*END*"), // extra
        full.replace(RA, ""),
        full.replace("290 300 1 8 0", "270 300 1 8 0"),
        full.replace("*END*", "FORGED\n*END*"),
        insert(&INTACT.replace("\r\n", "\n"), RA), // repair without gap
    ];
    for (i, src) in hostile.iter().enumerate() {
        assert!(wire(src).is_err(), "hostile {i} incorrectly accepted");
    }
}
