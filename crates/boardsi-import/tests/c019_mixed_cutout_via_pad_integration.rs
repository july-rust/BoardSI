//! C_019: V9–V12 cross-version integration on public synthetic PADS V9.4 MILS.
//! Receipt provenance and source-only geometry risk, never physical SI or DRC.
use boardsi_import::{
    reference_route_v10 as v10, reference_route_v11 as v11, reference_route_v12 as v12,
    reference_route_v8 as v8, reference_route_v9 as v9,
};

const HEAD: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const ROUTES: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n170 250 4 10 0\n*SIGNAL* NET_REF\nR1.1 R2.2\n140 280 2 10 0\n180 280 3 10 0 SIMPLEVIA\n190 280 3 10 0\n";

#[derive(Clone, Copy)]
enum Holes {
    Single,
    Multi,
    Mixed,
}

fn copper(holes: Holes, first_rect: &str) -> String {
    let mut output = String::from("*LINES*\n");
    for (name, layer) in [("RA", 2), ("RB", 3)] {
        let count = if matches!(holes, Holes::Single) { 2 } else { 3 };
        output.push_str(&format!(
            "{name} COPPER 100 200 {count} 1 0 NET_REF\nCOPCLS 5 0.01 0 {layer}\n0 0\n100 0\n100 100\n0 100\n0 0\n"
        ));
        match holes {
            Holes::Mixed => output.push_str(&format!(
                "COPCCO 2 0.01 0 {layer}\n30 50\n70 50\n"
            )),
            Holes::Single | Holes::Multi => {
                output.push_str(&format!("COPCUT 5 0.01 0 {layer}\n{first_rect}\n"));
            }
        }
        if !matches!(holes, Holes::Single) {
            output.push_str(&format!(
                "COPCUT 5 0.01 0 {layer}\n5 5\n12 5\n12 12\n5 12\n5 5\n"
            ));
        }
    }
    output
}

const NORMAL_RECT: &str = "30 30\n70 30\n70 70\n30 70\n30 30";
const TANGENT_RECT: &str = "32.5 32.5\n67.5 32.5\n67.5 67.5\n32.5 67.5\n32.5 32.5";
const OVERLAP_RECT: &str = "32.51 32.5\n67.51 32.5\n67.51 67.5\n32.51 67.5\n32.51 32.5";

fn board(holes: Holes, rect: &str) -> Vec<u8> {
    format!("{HEAD}{}{ROUTES}*END*\n", copper(holes, rect)).into_bytes()
}

fn encode(version: u8, bytes: &[u8]) -> Result<String, &'static str> {
    encode_selection(version, bytes, 1, ["NET_REF"; 2], [2, 3])
}

fn encode_selection(
    version: u8,
    bytes: &[u8],
    stitch: usize,
    refs: [&str; 2],
    layers: [usize; 2],
) -> Result<String, &'static str> {
    match version {
        9 => v9::encode(bytes, &[0, 1], 1, refs, layers, stitch),
        10 => v10::encode(bytes, &[0, 1], 1, refs, layers, stitch),
        11 => v11::encode(bytes, &[0, 1], 1, refs, layers, stitch),
        12 => v12::encode(bytes, &[0, 1], 1, refs, layers, stitch),
        _ => Err("C019_UNSUPPORTED_TEST_VERSION"),
    }
}

fn field(wire: &str, index: usize) -> &str {
    wire.lines().next().unwrap().split('|').nth(index).unwrap()
}

fn embedded(wire: &str, marker: &str) -> String {
    let lines: Vec<_> = wire.lines().collect();
    let position = lines.iter().position(|line| line.starts_with(marker)).unwrap();
    let count: usize = lines[position]
        .split('|')
        .last()
        .unwrap()
        .parse()
        .unwrap();
    format!("{}\n", lines[position + 1..position + 1 + count].join("\n"))
}

fn source_only(version: u8, wire: &str) {
    assert_eq!(field(wire, 0), format!("BOARDSI_REFERENCE_ROUTE_V{version}"));
    assert_eq!(&[field(wire, 3), field(wire, 4), field(wire, 5)], &["UNSUPPORTED", "NOT_RUN", "0"]);
    assert!(wire.ends_with("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
    assert!(!wire.contains("PHYSICAL_PASS"));
}

#[test]
fn one_rectangle_preserves_all_version_specific_legacy_receipts() {
    let bytes = board(Holes::Single, NORMAL_RECT);
    let original_v8 = v8::encode(&bytes, &[0, 1], 1, ["NET_REF"; 2], [2, 3], 1).unwrap();
    let wires: Vec<_> = (9..=12).map(|v| encode(v, &bytes).unwrap()).collect();
    for (offset, wire) in wires.iter().enumerate() {
        source_only(offset as u8 + 9, wire);
        assert_eq!(field(wire, 1), field(&wires[0], 1));
    }
    assert_eq!(embedded(&wires[0], "V8|"), original_v8);
    assert_eq!(embedded(&wires[1], "LEGACY|V9|"), wires[0]);
    assert_eq!(embedded(&wires[2], "LEGACY|V8|"), original_v8);
    assert_eq!(embedded(&wires[3], "LEGACY|V11|"), wires[2]);
    assert!(wires[1].lines().any(|line| line.starts_with("LEGACY|V9|")));
}

#[test]
fn multi_rectangle_v10_falls_back_to_original_v8_not_unsupported_v9() {
    let bytes = board(Holes::Multi, NORMAL_RECT);
    assert!(encode(9, &bytes).is_err());
    let wire10 = encode(10, &bytes).unwrap();
    let wire11 = encode(11, &bytes).unwrap();
    let wire12 = encode(12, &bytes).unwrap();
    let original_v8 = v8::encode(&bytes, &[0, 1], 1, ["NET_REF"; 2], [2, 3], 1).unwrap();
    for (v, wire) in [(10, &wire10), (11, &wire11), (12, &wire12)] {
        source_only(v, wire);
        assert_eq!(field(wire, 1), field(&wire10, 1));
    }
    assert_eq!(wire10.lines().filter(|line| line.starts_with("HOLE|")).count(), 4);
    assert_eq!(embedded(&wire10, "LEGACY|V8|"), original_v8);
    assert_eq!(embedded(&wire11, "LEGACY|V8|"), original_v8);
    assert_eq!(embedded(&wire12, "LEGACY|V11|"), wire11);
}

#[test]
fn mixed_circle_requires_genuine_v8_and_v11_v12_preserve_it() {
    let bytes = board(Holes::Mixed, NORMAL_RECT);
    assert!(encode(9, &bytes).is_err());
    assert!(encode(10, &bytes).is_err());
    let wire11 = encode(11, &bytes).unwrap();
    let wire12 = encode(12, &bytes).unwrap();
    let original_v8 = v8::encode(&bytes, &[0, 1], 1, ["NET_REF"; 2], [2, 3], 1).unwrap();
    source_only(11, &wire11);
    source_only(12, &wire12);
    assert_eq!(field(&wire11, 1), field(&wire12, 1));
    assert_eq!(embedded(&wire11, "LEGACY|V8|"), original_v8);
    assert_eq!(embedded(&wire12, "LEGACY|V11|"), wire11);
    assert_eq!(wire11.lines().filter(|line| line.contains("|C|")).count(), 2);
    assert_eq!(wire12.lines().filter(|line| line.starts_with("STITCH_PAD|")).count(), 2);
    assert!(wire11.contains("|V8|V8_SOURCE_CONTACT_CANDIDATE\n"));
}

#[test]
fn exact_tangent_and_overlap_remain_risks_across_v9_v10_and_v12() {
    for (rect, state) in [
        (TANGENT_RECT, "SOURCE_CUTOUT_TANGENCY_RISK"),
        (OVERLAP_RECT, "SOURCE_REFERENCE_COPPER_OVERLAP_RISK"),
    ] {
        let bytes = board(Holes::Single, rect);
        let wire9 = encode(9, &bytes).unwrap();
        let wire10 = encode(10, &bytes).unwrap();
        let wire11 = encode(11, &bytes).unwrap();
        let wire12 = encode(12, &bytes).unwrap();
        assert_eq!(field(&wire9, 2), state);
        assert_eq!(field(&wire10, 2), state);
        assert_eq!(embedded(&wire10, "LEGACY|V9|"), wire9);
        assert_eq!(embedded(&wire12, "LEGACY|V11|"), wire11);
        source_only(12, &wire12);
    }
}

#[test]
fn source_mutation_wrong_selection_and_illegal_copper_fail_closed() {
    let original = board(Holes::Single, NORMAL_RECT);
    let mutated = String::from_utf8(original.clone())
        .unwrap()
        .replace("NET_SIG", "NET_SIG_CHANGED");
    for version in 9..=12 {
        let a = encode(version, &original).unwrap();
        let b = encode(version, mutated.as_bytes()).unwrap();
        assert_ne!(field(&a, 1), field(&b, 1));
        source_only(version, &b);
        assert!(encode_selection(version, &original, 0, ["NET_REF"; 2], [2, 3]).is_err());
        assert!(encode_selection(version, &original, 99, ["NET_REF"; 2], [2, 3]).is_err());
        assert!(encode_selection(version, &original, 1, ["NET_REF"; 2], [2, 2]).is_err());
        assert!(encode_selection(version, &original, 1, ["NET_REF", "OTHER"], [2, 3]).is_err());
    }
    let broken_net = String::from_utf8(original).unwrap().replacen(
        "RA COPPER 100 200 2 1 0 NET_REF",
        "RA COPPER 100 200 2 1 0 OTHER",
        1,
    );
    for version in 9..=12 {
        assert!(encode(version, broken_net.as_bytes()).is_err());
    }
    let mixed = String::from_utf8(board(Holes::Mixed, NORMAL_RECT)).unwrap();
    let overlapping_circle = mixed.replacen("30 50\n70 50", "5 5\n45 5", 1);
    assert!(encode(11, overlapping_circle.as_bytes()).is_err());
    assert!(encode(12, overlapping_circle.as_bytes()).is_err());
}
