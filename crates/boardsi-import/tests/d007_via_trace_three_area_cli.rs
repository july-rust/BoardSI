//! D_007 source-only Rust integration regression.
#[allow(dead_code)]
#[path = "../src/bin/boardsi-d007-via-trace-three-area.rs"]
mod d007;
const SOURCE: &[u8] = include_bytes!("fixtures/d007_via_trace_three_area_v94.asc");
#[test]
fn one_via_one_trace_three_areas() {
    let wire = d007::encode(SOURCE, ["RA", "RB", "RC"], "NET_A", 1, 0).unwrap();
    assert!(
        wire.contains("LINK|SOURCE_VIA_TRACE_AREA_CANDIDATE|0|1|0|1|0|1"),
        "{wire}"
    );
    assert!(wire.contains("PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN"));
}
