//! Public synthetic C_006 rational Source Reference gap regression; no physical SI.
use boardsi_import::reference_gap_intervals::encode;
const HEADER:&str="!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const AREA:&str="*LINES*\nREF COPPER 100 200 2 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const CUT:&str="COPCUT 5 0.01 0 2\n20 20\n40 20\n40 40\n20 40\n20 20\n";
const CUT2:&str="COPCUT 5 0.01 0 2\n50 20\n60 20\n60 40\n50 40\n50 20\n";
fn source(area:&str,route:&str)->String {
    format!("{HEADER}{area}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n{route}\n*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n*END*\n")
}
fn audit(area:&str,route:&str)->String {
    let bytes=source(area,route);
    encode(bytes.as_bytes(),0,"NET_REF",2).unwrap()
}
#[test]
fn covered_is_geometry_only() {
    let s=audit(AREA,"130 250 1 10 0\n170 250 1 10 0");
    assert!(s.contains("|SOURCE_GEOMETRY_ONLY|"));
    assert!(s.contains("END|1|0|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL"));
    assert!(!s.contains("\nI|"));
}
#[test]
fn whole_uncovered_is_exact_full_interval() {
    let s=audit(AREA,"210 230 1 10 0\n260 230 1 10 0");
    assert!(s.contains("|REFERENCE_SPLIT_RISK|"));
    assert!(s.contains("\nI|0|1|1|1\n"));
}
#[test]
fn one_cutout_returns_exact_fractions() {
    let s=audit(&format!("{AREA}{CUT}"),"110 230 1 10 0\n170 230 1 10 0");
    assert!(s.contains("\nI|1|6|1|2\n"),"{s}");
    assert!(s.ends_with("END|1|1|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}
#[test]
fn multiple_cutouts_return_ordered_distinct_gaps() {
    let s=audit(&format!("{AREA}{CUT}{CUT2}"),"110 230 1 10 0\n170 230 1 10 0");
    assert!(s.contains("\nI|1|6|1|2\nI|2|3|5|6\n"),"{s}");
    assert!(s.contains("END|1|2|"));
}
#[test]
fn partial_outer_reference_uses_actual_crossing() {
    let s=audit(AREA,"90 250 1 10 0\n140 250 1 10 0");
    assert!(s.contains("\nI|0|1|1|5\n"),"{s}");
}
#[test]
fn diagonal_crossing_uses_exact_rational_parameter() {
    let s=audit(&format!("{AREA}{CUT}"),"110 222 1 10 0\n170 248 1 10 0");
    assert!(s.contains("\nI|1|6|1|2\n"),"{s}");
}
#[test]
fn collinear_boundary_is_unsupported_no_forged_precision() {
    let s=audit(AREA,"110 200 1 10 0\n170 200 1 10 0");
    assert!(s.contains("|UNSUPPORTED|"),"{s}");
    assert!(!s.contains("\nI|"));
}
#[test]
fn unknown_reference_or_layer_is_not_a_positive_result() {
    let bytes=source(AREA,"130 250 1 10 0\n170 250 1 10 0");
    for (name,layer) in [("WRONG",2),("NET_REF",4),("NET_SIG",2)] {
        let result=encode(bytes.as_bytes(),0,name,layer).unwrap();
        assert!(result.contains("|UNSUPPORTED|"),"{result}");
        assert!(!result.contains("\nI|"));
    }
}
#[test]
fn original_v1_remains_unchanged_in_opt_in_v2() {
    let bytes=source(&format!("{AREA}{CUT}"),"110 230 1 10 0\n170 230 1 10 0");
    let v1=boardsi_import::reference_risk_wire::encode(bytes.as_bytes(),&[0],"NET_REF",&[2]).unwrap();
    let v2=encode(bytes.as_bytes(),0,"NET_REF",2).unwrap();
    assert!(v1.starts_with("BOARDSI_REFERENCE_RISK_V1|"));
    assert!(v2.starts_with("BOARDSI_REFERENCE_GAP_V2|"));
    assert!(v1.contains("\nM|") && v2.contains("\nI|"));
    assert!(v2.contains("|0|UNSUPPORTED|NOT_RUN|0\n"));
}
