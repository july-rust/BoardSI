//! Launch the actual read-only binary; a header-only fixture is never full PCB import.
use std::process::Command;

#[test]
fn cli_distinguishes_pads_signature_from_non_pcb_asc() {
    let binary = env!("CARGO_BIN_EXE_boardsi-import-probe");
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    let pads = Command::new(binary)
        .arg(fixtures.join("pads_header_only.asc"))
        .output()
        .expect("PADS probe executable must start");
    assert!(pads.status.success());
    assert_eq!(
        String::from_utf8(pads.stdout).unwrap().trim(),
        "PADS_PCB_ASCII_SIGNATURE_ONLY; geometry=NOT_PARSED; si=NOT_RUN"
    );

    let unknown = Command::new(binary)
        .arg(fixtures.join("not_a_pcb.asc"))
        .output()
        .expect("PADS probe executable must start");
    assert_eq!(unknown.status.code(), Some(3));
    assert_eq!(
        String::from_utf8(unknown.stdout).unwrap().trim(),
        "UNKNOWN_OR_UNSUPPORTED; pcb_import=UNSUPPORTED"
    );
}

#[test]
fn artwork_cli_uses_exact_nm_transport_for_known_public_geometry() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_route_straight_v94.asc");
    let output = Command::new(bin)
        .arg(&fixture)
        .output()
        .expect("restricted PADS artwork importer must start");
    assert!(
        output.status.success(),
        "artwork importer exit was {:?}: {:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let message = String::from_utf8(output.stdout).expect("ASCII transport");
    assert!(message.starts_with("BOARDSI_P2A_PARTIAL_V1|V9.4-MILS|4\n"));
    // Source points (100,200) -> (300,200), width 10 mil;
    // independent fixed reference values: 2_540_000, 5_080_000,
    // 7_620_000 and 254_000 nanometres.
    assert!(message.contains("T|1|NET_A|2540000|5080000|7620000|5080000|254000\n"));
    assert!(message.contains("T|4|NET_B|25400000|7620000|38100000|7620000|304800\n"));
    assert!(message.ends_with("END|3|0|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn p2c_cli_emits_explicit_unverified_net_and_exact_simple_pad_geometry() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_component_simple_v94.asc");
    let output = Command::new(bin)
        .arg(file)
        .output()
        .expect("P2-C CLI executable");
    assert!(
        output.status.success(),
        "{:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let msg = String::from_utf8(output.stdout).expect("bounded ASCII");
    assert!(msg.starts_with("BOARDSI_P2C_PARTIAL_V2|V9.4-MILS|4\n"));
    assert!(msg.contains("P|U1|1|1|R|5080000|7620000|1016000|1016000|NET_UNVERIFIED\n"));
    assert!(msg.contains("P|U1|2|1|S|5080000|10160000|762000|762000|NET_UNVERIFIED\n"));
    assert!(msg.ends_with("END|1|0|2|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn p2d_cli_pad_only_board_has_independent_mm_geometry() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_rf_rectangle_v94.asc");
    let output = Command::new(bin).arg(&fixture).output().expect("P2D CLI");
    assert!(
        output.status.success(),
        "{:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let msg = String::from_utf8(output.stdout).unwrap();
    assert!(msg.starts_with("BOARDSI_P2D_PARTIAL_V3|V9.4-MILS|4\n"));
    let pads: Vec<_> = msg.lines().filter(|line| line.starts_with("P|")).collect();
    assert_eq!(pads.len(), 8);
    let expected = [
        "P|U0|1|1|RF|5080000|7620000|2032000|762000|NET_UNVERIFIED",
        "P|U0|2|1|RF|7620000|7620000|762000|2032000|NET_UNVERIFIED",
        "P|U1|1|1|RF|10160000|7620000|762000|2032000|NET_UNVERIFIED",
        "P|U1|2|1|RF|10160000|10160000|2032000|762000|NET_UNVERIFIED",
        "P|U2|1|1|RF|15240000|7620000|2032000|762000|NET_UNVERIFIED",
        "P|U2|2|1|RF|12700000|7620000|762000|2032000|NET_UNVERIFIED",
        "P|U3|1|1|RF|20320000|7620000|762000|2032000|NET_UNVERIFIED",
        "P|U3|2|1|RF|20320000|5080000|2032000|762000|NET_UNVERIFIED",
    ];
    for pad in expected {
        assert!(pads.contains(&pad), "missing expected: {pad}");
    }
    assert!(msg.ends_with("END|0|0|8|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn v1_transport_remains_unchanged_when_unsupported_parts_emit_no_pads() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let baseline = include_str!("fixtures/pads_route_straight_v94.asc");
    assert!(baseline.contains("*ROUTE*"));
    let with_rejected_part = baseline.replace(
        "*ROUTE*",
        "*PART*\nU1 UNKNOWN_TYPE 200 300 90 U N 0 -1 0 -1 0\n*ROUTE*",
    );
    let temp_path = std::env::temp_dir().join(format!(
        "boardsi_v1_unsupported_part_{}.asc",
        std::process::id()
    ));
    std::fs::write(&temp_path, with_rejected_part).expect("write synthetic fixture");
    let output = Command::new(bin)
        .arg(&temp_path)
        .output()
        .expect("PADS artwork importer must start");
    let _ = std::fs::remove_file(&temp_path);
    assert!(
        output.status.success(),
        "importer: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let message = String::from_utf8(output.stdout).expect("UTF-8 transport");
    assert!(message.starts_with("BOARDSI_P2A_PARTIAL_V1|V9.4-MILS|4\n"));
    assert!(!message.contains("\nP|"));
    assert!(message.ends_with("END|3|0|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn p2e_cli_bottom_mirror_top_mixed_explicit_side_and_no_net() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_bottom_mirror_v94.asc");
    let output = Command::new(bin).arg(&fixture).output().expect("P2E CLI");
    assert!(
        output.status.success(),
        "{:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let msg = String::from_utf8(output.stdout).unwrap();
    assert!(msg.starts_with("BOARDSI_P2E_PARTIAL_V4|V9.4-MILS|4\n"));
    let pads: Vec<_> = msg.lines().filter(|s| s.starts_with("P|")).collect();
    assert_eq!(pads.len(), 10);
    let expected = [
        "P|U0|1|4|RF|5080000|7620000|2032000|762000|NET_UNVERIFIED|B|0",
        "P|U0|2|4|RF|2540000|7620000|762000|2032000|NET_UNVERIFIED|B|0",
        "P|U1|1|4|RF|10160000|7620000|762000|2032000|NET_UNVERIFIED|B|90",
        "P|U1|2|4|RF|10160000|10160000|2032000|762000|NET_UNVERIFIED|B|90",
        "P|U2|1|4|RF|15240000|7620000|2032000|762000|NET_UNVERIFIED|B|180",
        "P|U2|2|4|RF|17780000|7620000|762000|2032000|NET_UNVERIFIED|B|180",
        "P|U3|1|4|RF|20320000|7620000|762000|2032000|NET_UNVERIFIED|B|270",
        "P|U3|2|4|RF|20320000|5080000|2032000|762000|NET_UNVERIFIED|B|270",
        "P|U4|1|1|RF|25400000|7620000|2032000|762000|NET_UNVERIFIED|T|0",
        "P|U4|2|1|RF|27940000|7620000|762000|2032000|NET_UNVERIFIED|T|0",
    ];
    for want in expected {
        assert!(pads.contains(&want), "missing {want}");
    }
    assert!(msg.ends_with("END|0|0|10|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn p2g_cli_emits_exact_non_copper_board_and_cutout_edges() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_board_outline_cutout_v94.asc");
    let output = Command::new(bin).arg(fixture).output().expect("P2G CLI");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let msg = String::from_utf8(output.stdout).unwrap();
    assert!(msg.starts_with("BOARDSI_P2G_PARTIAL_V6|V9.4-MILS|4\n"));
    assert!(msg.contains("O|OUTER|MAIN_BOARD|0|2540000|5080000|12700000|5080000\n"));
    assert!(msg.contains("O|CUT|INNER_CUT|0|5080000|7620000|7620000|7620000\n"));
    assert_eq!(msg.lines().filter(|s| s.starts_with("O|")).count(), 8);
    assert!(msg.ends_with("END|1|0|0|0|1|1|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn p2g_cli_refuses_lines_get_instead_of_silently_losing_board_boundary() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let original = include_str!("fixtures/pads_board_outline_cutout_v94.asc");
    // A valid routed board with a library reference inside *LINES* must never
    // succeed while silently dropping an explicit BOARD perimeter.
    let with_get = original.replace(
        "MAIN_BOARD BOARD",
        "*GET* UNVERIFIED_LIBRARY_OUTLINE\nMAIN_BOARD BOARD",
    );
    let path = std::env::temp_dir().join(format!(
        "boardsi_p2g_get_fails_closed_{}.asc",
        std::process::id()
    ));
    std::fs::write(&path, with_get).expect("write public synthetic board");
    let result = Command::new(bin)
        .arg(&path)
        .output()
        .expect("run P2G importer");
    let _ = std::fs::remove_file(&path);
    assert_eq!(result.status.code(), Some(3));
    assert_eq!(
        String::from_utf8_lossy(&result.stderr).trim(),
        "P2A_IMPORT_ERROR:UNSUPPORTED_OUTLINE_GET"
    );
}

#[test]
fn p2h_cli_emits_exact_analytical_circles_with_versioned_end() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_board_circle_cutout_v94.asc");
    let output = Command::new(bin).arg(fixture).output().expect("P2H CLI");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let msg = String::from_utf8(output.stdout).unwrap();
    assert!(msg.starts_with("BOARDSI_P2H_PARTIAL_V7|V9.4-MILS|4\n"));
    assert!(msg.contains("C|OUTER|MAIN_CIRCLE|7620000|5080000|5080000\n"));
    assert!(msg.contains("C|CUT|INNER_CIRCLE|7620000|5080000|1270000\n"));
    assert_eq!(msg.lines().filter(|s| s.starts_with("C|")).count(), 2);
    assert_eq!(msg.lines().filter(|s| s.starts_with("O|")).count(), 0);
    assert!(msg.ends_with("END|0|0|0|0|1|1|2|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn a018_mixed_line_arc_outline_remains_fail_closed_until_full_geometry_and_ipc() {
    let bin = env!("CARGO_BIN_EXE_boardsi-import-artwork");
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pads_board_mixed_quarter_arc_v94.asc");
    let output = Command::new(bin)
        .arg(source)
        .output()
        .expect("restricted PADS artwork importer must start");
    // PADS arc source is now independently preflighted but not promoted to
    // incomplete board geometry or a misleading V7 straight-edge transport.
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim(),
        "P2A_IMPORT_ERROR:UNSUPPORTED_OUTLINE_ARC_OR_POINT"
    );
}
