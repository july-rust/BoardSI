//! D_051 SOURCE-exact dual finite-width bridge / virtual cut-set regression.
use boardsi_import::d051_dual_bridge_loop;
const SOURCE: &str = include_str!("fixtures/d051_mixed_pad_dual_finite_width_loop_v94.asc");
const BRIDGE: &str = "U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0";
fn audit(s: &str) -> Result<String, &'static str> {
    d051_dual_bridge_loop::encode(s.as_bytes())
}
#[test]
fn exact_dual_bridge_loop_and_single_failure_reachability() {
    let wire = audit(SOURCE).expect("D051 valid SOURCE");
    assert!(wire.starts_with("BOARDSI_D051_DUAL_FINITE_WIDTH_LOOP_V1|"));
    assert!(wire.contains("C|0|U1_U2|NET_A|1|12|12\n"));
    assert_eq!(wire.matches("\nP|").count(), 4);
    assert_eq!(wire.matches("\nT|").count(), 8);
    assert_eq!(wire.matches("\nE|").count(), 12);
    assert_eq!(wire.matches("\nD|").count(), 4);
    assert!(wire.contains("T|7|1|NET_A|22200|40000|38200|40000|800\n"));
    assert!(wire.contains("E|10|9|11|POSITIVE_FINITE_WIDTH_TRACE_TRACE_SECOND_BRIDGE\n"));
    assert!(wire.contains("E|11|3|11|POSITIVE_FINITE_WIDTH_TRACE_SQUARE_PAD_SECOND_BRIDGE\n"));
    for expected in [
        "D|NONE|12|12|1|1|REACHABLE\n",
        "D|FIRST|11|10|1|0|REACHABLE\n",
        "D|SECOND|11|10|1|0|REACHABLE\n",
        "D|BOTH|10|8|2|0|SEPARATED\n",
        "COUNT|NODES|12|EDGES|12|LOCAL_COMPONENTS|1|CYCLE_RANK|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n",
    ] {
        assert!(wire.contains(expected), "missing {expected}");
    }
    assert!(wire.ends_with("END|SOURCE_LOCAL_DUAL_FINITE_WIDTH_REDUNDANCY_ONLY_NO_GLOBAL_PASS\n"));
}
#[test]
fn accepted_source_variants_and_exact_one_tick_boundaries() {
    let variants = [
        SOURCE.replace(BRIDGE, "U1.2 U2.2\n382 400 1 8 0\n222 400 1 8 0"),
        SOURCE.replace("NET_A", "NET_B"),
        SOURCE.replace('\n', "\r\n"),
        SOURCE.replace("*SIGNAL* NET_A", "*SIGNAL*\tNET_A"),
        SOURCE.replace("222 400 1 8 0", "+222 400 1 8.00 128"),
        SOURCE.replace("382 400 1 8 0", "382 400 65 8 0"),
        SOURCE.replace("222 400 1 8 0", "224.99 400 1 8 0"),
        SOURCE.replace("222 400 1 8 0", "219.01 400 1 8 0"),
        SOURCE.replace("382 400 1 8 0", "381.01 400 1 8 0"),
        SOURCE.replace("382 400 1 8 0", "384.99 400 1 8 0"),
    ];
    for (i, source) in variants.iter().enumerate() {
        assert_ne!(source, SOURCE, "inert positive {i}");
        assert!(audit(source).is_ok(), "positive {i}");
    }
}
#[test]
fn hostile_missing_duplicate_tangent_wrong_owner_and_stale_shape_fail_closed() {
    let cases = [
        SOURCE.replace("222 400 1 8 0", "225 400 1 8 0"),
        SOURCE.replace("222 400 1 8 0", "225.01 400 1 8 0"),
        SOURCE.replace("222 400 1 8 0", "219 400 1 8 0"),
        SOURCE.replace("382 400 1 8 0", "381 400 1 8 0"),
        SOURCE.replace("382 400 1 8 0", "380.99 400 1 8 0"),
        SOURCE.replace("382 400 1 8 0", "385 400 1 8 0"),
        SOURCE.replace("222 400 1 8 0", "222 401 1 8 0"),
        SOURCE.replace("222 400 1 8 0", "222 400 2 8 0"),
        SOURCE.replace("222 400 1 8 0", "222 400 1 10 0"),
        SOURCE.replace("222 400 1 8 0", "222 400 1 8.001 0"),
        SOURCE.replace("222 400 1 8 0", "222 400 1 8 16"),
        SOURCE.replace(BRIDGE, "U1.1 U2.2\n222 400 1 8 0\n382 400 1 8 0"),
        SOURCE.replace(BRIDGE, &format!("*SIGNAL* NET_B\n{BRIDGE}")),
        SOURCE.replace("217 390 1 8 0", "217 370 1 8 0"),
        SOURCE.replace("PAD 2 3\n-2 30 S 0", "PAD 2 3\n-2 30 R 0"),
        SOURCE.replace("U2 TYPEA 400 300", "U2 TYPEA 398 300"),
        SOURCE.replace("NET_A", "NET_C"),
        SOURCE.replace(&format!("{BRIDGE}\n"), ""),
        SOURCE.replace("*END*", &format!("{BRIDGE}\n*END*")),
        SOURCE.replace("*END*", "U1.1 U2.1\n240 320 1 8 0\n360 320 1 8 0\n*END*"),
        SOURCE.replace("226 300 1 8 0", "230 300 1 8 0"),
        SOURCE.replace("*END*", "*END*\nFORGED"),
    ];
    for (i, source) in cases.iter().enumerate() {
        assert_ne!(source, SOURCE, "inert hostile {i}");
        assert!(audit(source).is_err(), "hostile {i} passed");
    }
}
