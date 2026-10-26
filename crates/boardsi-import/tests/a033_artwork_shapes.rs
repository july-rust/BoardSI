//! A_033: independently authored closed-outline and source-owned copper/hole
//! point-set oracles. A display triangulation is never the physical oracle.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

const NM_PER_MIL: i64 = 25_400;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn import(name: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_boardsi-import-artwork"))
        .arg(fixture(name))
        .output()
        .expect("run native PADS artwork importer");
    assert!(
        output.status.success(),
        "synthetic source rejected: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("ASCII geometry transport")
}

// This oracle uses original authored MIL locations, not decoded IPC values.
fn line(kind: &str, name: &str, n: usize, a: [i64; 2], b: [i64; 2]) -> String {
    format!(
        "O|{kind}|{name}|{n}|{}|{}|{}|{}",
        a[0] * NM_PER_MIL,
        a[1] * NM_PER_MIL,
        b[0] * NM_PER_MIL,
        b[1] * NM_PER_MIL
    )
}

fn polygon(tag: &str, vertices_mil: &[[i64; 2]]) -> String {
    let mut record = format!("{tag}|{}", vertices_mil.len());
    for [x, y] in vertices_mil {
        record.push_str(&format!("|{}|{}", x * NM_PER_MIL, y * NM_PER_MIL));
    }
    record
}

#[test]
fn source_board_outline_and_explicit_cutout_have_exact_four_edges_each() {
    let wire = import("pads_board_outline_cutout_v94.asc");
    assert_eq!(
        wire.lines().next(),
        Some("BOARDSI_P2G_PARTIAL_V6|V9.4-MILS|4")
    );
    let actual: BTreeSet<String> = wire
        .lines()
        .filter(|row| row.starts_with("O|"))
        .map(str::to_owned)
        .collect();
    let outer = [[100, 200], [500, 200], [500, 500], [100, 500], [100, 200]];
    let cut = [[200, 300], [300, 300], [300, 400], [200, 400], [200, 300]];
    let mut expected = BTreeSet::new();
    for (kind, name, vertices) in [("OUTER", "MAIN_BOARD", outer), ("CUT", "INNER_CUT", cut)] {
        for (n, pair) in vertices.windows(2).enumerate() {
            expected.insert(line(kind, name, n, pair[0], pair[1]));
        }
    }
    assert_eq!(actual.len(), 8);
    assert_eq!(wire.lines().filter(|row| row.starts_with("O|")).count(), 8);
    assert_eq!(
        actual, expected,
        "source outline/hole vertices must remain exact"
    );
    assert_eq!(wire.lines().filter(|row| row.starts_with("G|")).count(), 0);
    assert_eq!(wire.lines().filter(|row| row.starts_with("H|")).count(), 0);
    assert!(wire.ends_with("CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}

#[test]
fn source_owned_copper_polygon_has_exact_two_rectangular_holes() {
    let wire = import("pads_copper_cutout_v94.asc");
    assert_eq!(
        wire.lines().next(),
        Some("BOARDSI_P2N_PARTIAL_V11|V9.4-MILS|4")
    );
    let all_copper: Vec<_> = wire.lines().filter(|row| row.starts_with("G|")).collect();
    assert_eq!(all_copper.len(), 1, "no extra or lost copper area");
    let fields: Vec<_> = all_copper[0].split('|').collect();
    assert_eq!(&fields[0..5], &["G", "P", "1", "COP_HOLE", "GND"]);
    assert!(
        fields[5].parse::<usize>().is_ok(),
        "source line must remain numeric provenance"
    );
    let outer = polygon(
        "G|P|1|COP_HOLE|GND",
        &[[40, 40], [140, 40], [140, 120], [40, 120], [40, 40]],
    );
    // Field 5 carries source-line provenance, so remove ONLY that field:
    // exact authored geometry (field 6 onward) remains fully compared.
    let normalized = format!("{}|{}", fields[..5].join("|"), fields[6..].join("|"));
    assert_eq!(normalized, outer);
    let expected: BTreeSet<_> = [
        polygon("H|0", &[[60, 60], [80, 60], [80, 80], [60, 80], [60, 60]]),
        polygon(
            "H|0",
            &[[100, 80], [120, 80], [120, 100], [100, 100], [100, 80]],
        ),
    ]
    .into_iter()
    .collect();
    let holes: Vec<_> = wire.lines().filter(|row| row.starts_with("H|")).collect();
    let actual: BTreeSet<_> = holes.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(holes.len(), 2);
    assert_eq!(
        actual, expected,
        "two explicitly owned voids must be preserved"
    );
    assert!(
        wire.lines().all(|row| !row.starts_with("J|")),
        "no invented circular void"
    );
    assert!(wire.ends_with("CANONICAL_PARTIAL|SI_NOT_RUN\n"));
}
