//! D_043 independent, source-bound counterfactual graph qualification.
use boardsi_import::d043_local_cutset::{encode, parse_cut_id};

const SOURCE: &str = include_str!("fixtures/d042_local_closed_loop_v94.asc");

fn metrics(receipt: &str) -> &str {
    receipt
        .lines()
        .find(|line| line.starts_with("COUNT|"))
        .expect("count")
}

#[test]
fn baseline_and_each_cut_set_have_graph_derived_rank_and_reachability() {
    let cases = std::iter::once((vec![], 14, 14, 1, 1, "YES"))
        .chain((4..=9).map(|i| (vec![i], 13, 12, 1, 0, "YES")))
        .chain((5..=9).map(|i| (vec![4, i], 12, 10, 2, 0, "NO")))
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 12);
    for (cut, nodes, edges, components, cycles, reachable) in cases {
        let result = encode(SOURCE.as_bytes(), &cut).expect("valid fixture");
        assert!(result.starts_with("BOARDSI_D043_LOCAL_CUTSET_V1|"));
        assert!(result.contains("SOURCE_LOCAL_COUNTERFACTUAL_ONLY"));
        assert!(result.contains("P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED"));
        assert_eq!(
            metrics(&result),
            format!("COUNT|NODES|{nodes}|EDGES|{edges}|LOCAL_COMPONENTS|{components}|LOCAL_CYCLES|{cycles}|U1_U2_REACHABLE|{reachable}")
        );
        assert!(result.ends_with("END|SOURCE_LOCAL_DIAGNOSTIC_NO_PHYSICAL_OPEN_SHORT\n"));
        assert_eq!(result, encode(SOURCE.as_bytes(), &cut).unwrap());
    }
}

#[test]
fn invalid_cut_identifiers_sets_and_incomplete_source_are_rejected() {
    for invalid in [
        vec![0],
        vec![1],
        vec![3],
        vec![10],
        vec![4, 4],
        vec![5, 6],
        vec![6, 4],
        vec![4, 10],
        vec![4, 5, 6],
    ] {
        assert!(encode(SOURCE.as_bytes(), &invalid).is_err(), "{invalid:?}");
    }
    for token in ["T0", "T3", "T10", "T04", "4", "t4", "T5x", "T-4", "T"] {
        assert!(parse_cut_id(token).is_err(), "{token}");
    }
    for valid in 4..=9 {
        assert_eq!(parse_cut_id(&format!("T{valid}")), Ok(valid));
    }
    let source = SOURCE.replace("\r\n", "\n");
    let missing = source.replace("U1.2 U2.2\n150 250 1 10 0\n450 250 1 10 0\n", "");
    assert_ne!(source, missing);
    assert!(encode(missing.as_bytes(), &[4]).is_err());
    let wrong = source.replace("U1.2 U2.2\n200 350 1 10 0", "U1.1 U2.2\n200 350 1 10 0");
    assert_ne!(source, wrong);
    assert!(encode(wrong.as_bytes(), &[]).is_err());
}

#[test]
fn header_direction_newlines_and_net_preserve_bounded_results() {
    let src = SOURCE.replace("\r\n", "\n");
    let expected = encode(src.as_bytes(), &[4, 7]).unwrap();
    let swapped = src.replace(
        "U1.2 U2.2\n200 350 1 10 0\n150 350 1 10 0",
        "U1.2 U2.2\n150 350 1 10 0\n200 350 1 10 0",
    );
    assert_ne!(src, swapped);
    assert_eq!(
        metrics(&encode(swapped.as_bytes(), &[4, 7]).unwrap()),
        metrics(&expected)
    );
    for variant in [
        src.replace('\n', "\r\n"),
        src.replace("NET_A", "NET_B"),
        src.replace(
            "!PADS-POWERPCB-V9.4-MILS!",
            "!PADS-POWERPCB-V9.4-MILS-250L!",
        ),
    ] {
        assert_eq!(
            metrics(&encode(variant.as_bytes(), &[4, 7]).unwrap()),
            metrics(&expected)
        );
    }
}
