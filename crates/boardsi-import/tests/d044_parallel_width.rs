//! D_044 SOURCE-local finite-width parallel trace positive overlap, not electrical SI.
use boardsi_import::d044_parallel_width;

const SOURCE: &str = include_str!("fixtures/d044_parallel_width_v94.asc");

fn audit(s: &str) -> Result<String, &'static str> {
    d044_parallel_width::encode(s.as_bytes())
}

#[test]
fn finite_width_connects_two_local_components_without_centerline_coincidence() {
    let receipt = audit(SOURCE).expect("bounded synthetic source");
    assert!(receipt.starts_with("BOARDSI_D044_PARALLEL_WIDTH_CONTACT_V1|"));
    assert!(receipt.contains("C|0|U1+U2|NET_A|1|9|10\n"));
    assert_eq!(receipt.matches("\nP|").count(), 4);
    assert_eq!(receipt.matches("\nT|").count(), 5);
    assert_eq!(receipt.matches("\nE|").count(), 10);
    assert!(receipt.contains("E|9|7|8|POSITIVE_FINITE_WIDTH_PARALLEL_CONTACT\n"));
    assert!(receipt.contains("COUNT|NODES|9|EDGES|10|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED"));
    assert!(receipt.ends_with("END|SOURCE_LOCAL_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n"));
    assert_eq!(receipt, audit(SOURCE).unwrap());
}

#[test]
fn reversed_headers_nets_and_crlf_are_validated() {
    let reverse = SOURCE.replace(
        "230 349 1 52 0\n230 351 1 52 0",
        "230 351 1 52 0\n230 349 1 52 0",
    );
    assert_ne!(reverse, SOURCE);
    // Geometry identity is preserved even though the source digest changes.
    let normal = audit(SOURCE).unwrap();
    let flipped = audit(&reverse).unwrap();
    assert_eq!(normal.split_once('\n').unwrap().1.replace("23000|34900|23000|35100", "23000|35100|23000|34900"), flipped.split_once('\n').unwrap().1);
    assert!(audit(&SOURCE.replace('\n', "\r\n")).is_ok());
    assert!(audit(&SOURCE.replace("NET_A", "NET_B")).is_ok());
    for header in [
        "!PADS-POWERPCB-V9.4-MILS-CP1252!",
        "!PADS-POWERPCB-V9.4-MILS-250L!",
    ] {
        assert!(audit(&SOURCE.replace("!PADS-POWERPCB-V9.4-MILS!", header)).is_ok());
    }
}

#[test]
fn boundary_gap_layer_owner_and_invalid_source_fail_closed() {
    let fail = [
        SOURCE.replace("1 52 0", "1 50 0"),
        SOURCE.replace("1 52 0", "1 48 0"),
        SOURCE.replace("1 52 0", "2 52 0"),
        SOURCE.replace("230 349 1 52 0", "230 349 1 0 0"),
        SOURCE.replace("U1.1 U2.1", "U1.1 U1.2"),
        SOURCE.replace("230 349 1 52 0", "200 349 1 52 0"),
        SOURCE.replace("230 349 1 52 0", "230 300 1 52 0"),
        SOURCE.replace("230 349 1 52 0", "230 351 1 52 0"),
        SOURCE.replace("U1.1 U2.1\n230 349 1 52 0\n230 351 1 52 0\n", ""),
        SOURCE.replace("*END*", "U1.1 U2.1\n230 349 1 52 0\n230 351 1 52 0\n*END*"),
        SOURCE.replace("U2 TYPEA 260 300", "U2 TYPEA 200 300"),
        SOURCE.replace("*END*", "*VIA*\n*END*"),
        SOURCE.replace("*END*", "*END*\nUNSUPPORTED"),
    ];
    for (index, source) in fail.iter().enumerate() {
        assert_ne!(source, SOURCE, "inert mutation {index}");
        assert!(audit(source).is_err(), "accepted hostile variant {index}");
    }
}
