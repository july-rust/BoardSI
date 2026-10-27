//! Public native process-level regression: opt-in evidence must appear
//! AFTER an unchanged complete V1-V14 artwork stdout, including the distinct
//! early-return serializer for source COPPER/COPCUT/COPCCO curved V10-V14.
use std::path::PathBuf;
use std::process::Command;

fn invoke(fixture: &PathBuf, opt_in: bool) -> std::process::Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_boardsi-import-artwork"));
    process.arg(fixture);
    if opt_in {
        process.arg("--contact-records-v1");
    }
    process.output().expect("native importer binary")
}

#[test]
fn opt_in_suffix_preserves_old_artwork_for_every_copper_wire_version() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (filename, expected_version) in [
        ("pads_copper_filled_v94.asc", "V10"),
        ("pads_copper_cutout_v94.asc", "V11"),
        ("pads_copper_circular_void_v94.asc", "V12"),
        ("pads_curved_copper_candidate_v94.asc", "V13"),
        ("pads_curved_copper_polygon_cutout_v94.asc", "V14"),
    ] {
        let fixture = root.join(filename);
        let old = invoke(&fixture, false);
        assert!(
            old.status.success(),
            "{filename} default mode failed: {}",
            String::from_utf8_lossy(&old.stderr)
        );
        let extended = invoke(&fixture, true);
        assert!(
            extended.status.success(),
            "{filename} opt-in mode failed: {}",
            String::from_utf8_lossy(&extended.stderr)
        );
        assert!(
            old.stdout.starts_with(b"BOARDSI_"),
            "{filename} invalid old transport"
        );
        let first_line = String::from_utf8_lossy(&old.stdout)
            .lines()
            .next()
            .unwrap()
            .to_owned();
        assert!(
            first_line.contains(expected_version),
            "{filename} unexpected version {first_line}"
        );
        assert!(
            old.stdout.ends_with(b"SI_NOT_RUN\n"),
            "{filename} malformed old END"
        );
        assert!(
            extended.stdout.starts_with(&old.stdout),
            "{filename} legacy transport changed"
        );
        let suffix = &extended.stdout[old.stdout.len()..];
        assert!(
            suffix.starts_with(b"BOARDSI_CONTACT_RECORDS_V1|"),
            "{filename} missing requested suffix"
        );
        let text = String::from_utf8_lossy(suffix);
        let header = text.lines().next().unwrap();
        let fields: Vec<_> = header.split('|').collect();
        assert_eq!(fields.len(), 4, "{filename} malformed opt-in header");
        let count: usize = fields[1].parse().expect("decimal record count");
        assert_eq!(fields[2], "LOCAL_ENDPOINT_ONLY");
        assert_eq!(fields[3].len(), 64, "{filename} missing source hash");
        assert_eq!(
            text.lines().count(),
            count + 2,
            "{filename} truncated record suffix"
        );
        assert!(text.ends_with(&format!("BOARDSI_CONTACT_END_V1|{count}\n")));
        assert_eq!(
            extended.stderr, old.stderr,
            "{filename} default audit changed"
        );
    }
}

#[test]
fn b003_canonical_interior_edges_do_not_break_legacy_qt_audit_counts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (filename, expected_local, expected_barrels, expected_records) in [
        ("b003_t_junction_v94.asc", 0usize, 0usize, 0usize),
        ("b003_via_interior_v94.asc", 1usize, 0usize, 1usize),
    ] {
        let fixture = root.join(filename);
        let default = invoke(&fixture, false);
        assert!(
            default.status.success(),
            "{filename} import failure: {}",
            String::from_utf8_lossy(&default.stderr)
        );
        let extended = invoke(&fixture, true);
        assert!(
            extended.status.success(),
            "{filename} opt-in import failure"
        );
        assert!(extended.stdout.starts_with(&default.stdout));
        assert_eq!(extended.stderr, default.stderr);
        let audit = String::from_utf8_lossy(&default.stderr);
        assert!(
            audit.contains(&format!(
                "BOARDSI_CONTACT_AUDIT_V1|{expected_local}|{expected_barrels}|"
            )),
            "{filename} legacy stderr counters differ from wire: {audit}"
        );
        let suffix = &extended.stdout[default.stdout.len()..];
        let text = String::from_utf8_lossy(suffix);
        assert!(text.starts_with(&format!(
            "BOARDSI_CONTACT_RECORDS_V1|{expected_records}|LOCAL_ENDPOINT_ONLY|"
        )));
        assert!(text.ends_with(&format!("BOARDSI_CONTACT_END_V1|{expected_records}\n")));
        assert_eq!(text.lines().count(), expected_records + 2);
    }
}
