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
    let output = Command::new(bin).arg(&fixture).output()
        .expect("restricted PADS artwork importer must start");
    assert!(output.status.success(),
        "artwork importer exit was {:?}: {:?}",
        output.status.code(), String::from_utf8_lossy(&output.stderr));
    let message = String::from_utf8(output.stdout).expect("ASCII transport");
    assert!(message.starts_with("BOARDSI_P2A_PARTIAL_V1|V9.4-MILS|4\n"));
    // Source points (100,200) -> (300,200), width 10 mil;
    // independent fixed reference values: 2_540_000, 5_080_000,
    // 7_620_000 and 254_000 nanometres.
    assert!(message.contains("T|1|NET_A|2540000|5080000|7620000|5080000|254000\n"));
    assert!(message.contains("T|4|NET_B|25400000|7620000|38100000|7620000|304800\n"));
    assert!(message.ends_with("END|3|0|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}
