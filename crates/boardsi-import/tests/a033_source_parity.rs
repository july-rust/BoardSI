//! A_033: independent, hand-authored source-grid oracles for the actual local
//! PADS V9.4 MILS CLI. This is NOT a vendor-export or GPU frame certification.
//! Exact complete primitive sets catch unexpected extra, omitted, or shifted art.
use boardsi_import::digest::sha256;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const NM_PER_MIL: i64 = 25_400;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn import(path: &Path, contacts: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_boardsi-import-artwork"));
    cmd.arg(path);
    if contacts {
        cmd.arg("--contact-records-v1");
    }
    cmd.output().expect("start local native PADS importer")
}

fn success(path: &Path, contacts: bool) -> Output {
    let out = import(path, contacts);
    assert!(
        out.status.success(),
        "import of public fixture failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn primitive_set(wire: &str, kind: &str) -> BTreeSet<String> {
    wire.lines()
        .filter(|row| row.starts_with(kind))
        .map(str::to_owned)
        .collect()
}

fn assert_only_rows(wire: &str, expected: &[&str], kind: &str) {
    let actual = primitive_set(wire, kind);
    assert_eq!(
        actual.len(),
        wire.lines().filter(|l| l.starts_with(kind)).count(),
        "duplicate native transport records"
    );
    let oracle: BTreeSet<String> = expected.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(
        actual, oracle,
        "every source-authored primitive must appear exactly once"
    );
}

#[test]
fn complete_straight_route_rows_match_hand_calculated_source_grid() {
    let path = fixture("pads_route_straight_v94.asc");
    let source = fs::read(&path).expect("synthetic fixture");
    assert!(source
        .windows(b"100 200 1 10 0".len())
        .any(|w| w == b"100 200 1 10 0"));
    assert_eq!(100 * NM_PER_MIL, 2_540_000);
    let wire = String::from_utf8(success(&path, false).stdout).expect("ASCII IPC");
    assert_eq!(
        wire.lines().next(),
        Some("BOARDSI_P2A_PARTIAL_V1|V9.4-MILS|4")
    );
    assert_only_rows(
        &wire,
        &[
            "T|1|NET_A|2540000|5080000|7620000|5080000|254000",
            "T|1|NET_A|7620000|5080000|7620000|10160000|254000",
            "T|4|NET_B|25400000|7620000|38100000|7620000|304800",
        ],
        "T|",
    );
    assert_eq!(
        wire.lines()
            .filter(|row| matches!(
                row.split('|').next(),
                Some("A" | "V" | "P" | "G" | "H" | "J" | "B" | "C" | "O" | "Q")
            ))
            .count(),
        0,
        "a route-only fixture must not invent other source artwork"
    );
    assert_eq!(wire.lines().filter(|row| row.starts_with("L|")).count(), 4);
    assert!(wire.ends_with("END|3|0|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn complete_bottom_mirror_pad_rows_match_manual_board_axis_oracle() {
    // Independent expected board coordinates and orthogonal rotations are
    // calculated from the small authored decal and placements, not from
    // importer snapshots, renderer triangles, or Canonical output.
    let path = fixture("pads_bottom_mirror_v94.asc");
    let wire = String::from_utf8(success(&path, false).stdout).expect("ASCII IPC");
    assert_eq!(
        wire.lines().next(),
        Some("BOARDSI_P2E_PARTIAL_V4|V9.4-MILS|4")
    );
    assert_only_rows(
        &wire,
        &[
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
        ],
        "P|",
    );
    assert_eq!(
        wire.lines()
            .filter(|row| matches!(
                row.split('|').next(),
                Some("T" | "A" | "V" | "G" | "H" | "J" | "Q" | "O" | "B" | "C")
            ))
            .count(),
        0,
        "pad-only source must not acquire phantom copper or traces"
    );
    assert!(wire.ends_with("END|0|0|10|CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn route_source_hash_and_local_contact_suffix_are_explicit_not_certified_short() {
    let path = fixture("pads_route_straight_v94.asc");
    let source = fs::read(&path).expect("public synthetic fixture");
    let legacy = success(&path, false);
    let extended = success(&path, true);
    assert!(extended.stdout.starts_with(&legacy.stdout));
    assert_eq!(
        extended.stderr, legacy.stderr,
        "opt-in cannot change the legacy audit"
    );
    let suffix = String::from_utf8(extended.stdout[legacy.stdout.len()..].to_vec())
        .expect("strict ASCII suffix");
    let mut expected_hash = String::with_capacity(64);
    for b in sha256(&source) {
        use std::fmt::Write as _;
        write!(&mut expected_hash, "{b:02x}").expect("hex text");
    }
    let mut rows = suffix.lines();
    let header = rows.next().expect("suffix header");
    let header_parts: Vec<_> = header.split('|').collect();
    assert_eq!(header_parts.len(), 4);
    assert_eq!(header_parts[0], "BOARDSI_CONTACT_RECORDS_V1");
    assert_eq!(header_parts[2], "LOCAL_ENDPOINT_ONLY");
    assert_eq!(
        header_parts[3], expected_hash,
        "suffix must bind original source bytes"
    );
    let count: usize = header_parts[1].parse().expect("bounded decimal count");
    let records: Vec<_> = rows.collect();
    assert_eq!(
        records.len(),
        count + 1,
        "record count and trailer are mandatory"
    );
    assert_eq!(
        records.last().copied(),
        Some(format!("BOARDSI_CONTACT_END_V1|{count}").as_str())
    );
    // Only adjacent *distinct* NET_A source traces touch at exactly
    // (300 mil, 200 mil); NET_B geometry must remain disconnected.
    assert_eq!(count, 1);
    assert!(
        records[0].starts_with("E|S|"),
        "local same-net endpoint only"
    );
    assert!(records[0].ends_with("|7620000|5080000"));
    assert!(!suffix.contains("\nX|C|"), "no cross-net short inference");
}

#[test]
fn mutated_source_must_not_pass_as_original_or_succeed_on_invalid_geometry() {
    let source = fs::read_to_string(fixture("pads_route_straight_v94.asc")).unwrap();
    let temp = std::env::temp_dir().join(format!("boardsi_a033_route_{}.asc", std::process::id()));
    // File is created from a PUBLIC synthetic fixture only, never a real board.
    let moved = source.replacen("100 200 1 10 0", "101 200 1 10 0", 1);
    assert_ne!(moved, source);
    fs::write(&temp, moved).unwrap();
    let shifted = success(&temp, false);
    let shifted_wire = String::from_utf8(shifted.stdout).unwrap();
    assert!(
        !shifted_wire.contains("T|1|NET_A|2540000|5080000|7620000|5080000|254000"),
        "the original Golden row must not survive a moved source endpoint"
    );
    assert!(shifted_wire.contains("T|1|NET_A|2565400|5080000|7620000|5080000|254000"));

    for bad in [
        source.replacen("MAXLAYER 4", "MAXLAYER 65", 1),
        source.replacen("100 200 1 10 0", "999999 200 1 10 0", 1),
        source.replacen("!PADS-POWERPCB-V9.4-MILS!", "!PADS-POWERPCB-V9.5-MILS!", 1),
    ] {
        assert_ne!(bad, source, "mutation must change input");
        fs::write(&temp, bad).unwrap();
        let failed = import(&temp, false);
        assert!(!failed.status.success(), "hostile input was accepted");
        assert!(
            failed.stdout.is_empty(),
            "failed import emitted usable partial artwork"
        );
    }
    fs::remove_file(&temp).unwrap();
}
