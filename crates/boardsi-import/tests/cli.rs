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
