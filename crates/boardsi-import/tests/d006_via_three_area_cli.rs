//! D_006 end-to-end bounded PADS source test of the independently executed CLI.
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SOURCE: &str = include_str!("fixtures/d006_via_three_area_v94.asc");

fn run(source: &str, layer: &str) -> (bool, String) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("boardsi-d006-{}-{unique}.asc", std::process::id()));
    std::fs::write(&path, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_boardsi-d006-via-three-area"))
        .args([
            path.to_str().unwrap(),
            "RA",
            "RB",
            "RC",
            "NET_A",
            layer,
            "0",
        ])
        .output()
        .unwrap();
    std::fs::remove_file(&path).unwrap();
    let bytes = if result.status.success() {
        &result.stdout
    } else {
        &result.stderr
    };
    (
        result.status.success(),
        String::from_utf8_lossy(bytes).into_owned(),
    )
}

#[test]
fn positive_source_candidate_never_means_electrical() {
    let (passed, output) = run(SOURCE, "1");
    assert!(passed, "{output}");
    assert!(output.starts_with("BOARDSI_D006_VIA_THREE_AREA_V1|"));
    assert!(
        output.contains("CONTACT|SOURCE_VIA_PAD_AREA_CANDIDATE|0|0"),
        "{output}"
    );
    assert!(output.contains("PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN"));
    assert!(output.ends_with("END|SOURCE_ONLY_NOT_ELECTRICAL\n"));
    assert!(!output.contains("PHYSICAL_PASS"));
}

#[test]
fn pad_perimeter_boundary_and_multiple_area_owner_are_not_accepted() {
    let edge = SOURCE.replace("150 250 1 10 0 SIMPLEVIA", "110 250 1 10 0 SIMPLEVIA");
    let (passed, output) = run(&edge, "1");
    assert!(passed, "{output}");
    assert!(
        output.contains("CONTACT|SOURCE_PAD_BOUNDARY_RISK|-|-"),
        "{output}"
    );

    let ambiguous = SOURCE.replace("150 250 1 10 0 SIMPLEVIA", "190 250 1 10 0 SIMPLEVIA");
    let (passed, output) = run(&ambiguous, "1");
    assert!(passed, "{output}");
    assert!(
        output.contains("CONTACT|SOURCE_AMBIGUOUS_AREA_OWNER|-|-"),
        "{output}"
    );
}

#[test]
fn copper_cutout_blocks_candidate() {
    // Windows checkout may use CRLF: normalize before matching a multiline span.
    let cutout = SOURCE
        .replace("\\r\\n", "\\n")
        .replace("RA COPPER 0 0 1 1 0 NET_A", "RA COPPER 0 0 2 1 0 NET_A")
        .replace(
            "100 300\n100 200\n",
            "100 300\n100 200\nCOPCUT 5 0.01 0 1\n140 240\n160 240\n160 260\n140 260\n140 240\n",
        );
    let (passed, output) = run(&cutout, "1");
    assert!(passed, "{output}");
    assert!(
        output.contains("CONTACT|SOURCE_COPCUT_RISK|-|-"),
        "{output}"
    );
}

#[test]
fn layer_net_and_incomplete_source_are_rejected() {
    assert!(!run(SOURCE, "2").0);
    let wrong_net = SOURCE.replace("RB COPPER 0 0 1 1 0 NET_A", "RB COPPER 0 0 1 1 0 NET_B");
    assert!(!run(&wrong_net, "1").0);
    let no_via = SOURCE.replace(" SIMPLEVIA", "");
    assert!(!run(&no_via, "1").0);
    let unsupported = SOURCE.replace("SIMPLEVIA 15 2", "SIMPLEVIA 15 3");
    assert!(!run(&unsupported, "1").0);
}

#[test]
fn different_source_bytes_cannot_reuse_sha() {
    let (ok_a, a) = run(SOURCE, "1");
    let changed = SOURCE.replace("150 250 1 10 0 SIMPLEVIA", "151 250 1 10 0 SIMPLEVIA");
    let (ok_b, b) = run(&changed, "1");
    assert!(ok_a && ok_b, "{a}\n{b}");
    assert_ne!(a.lines().next(), b.lines().next());
}
