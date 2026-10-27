//! A_011 P2-C: deliberately narrow, fail-closed V9.4 MILS component-pad preview.
//! Accepts zero-outline I-unit decals with explicit T terminals and 3-layer
//! drill-free round, square, or limited zero-offset rectangular SMD padstacks.
//! No mirrored placement, arbitrary finger rotation, or inferred connectivity.
use boardsi_model::*;
use std::collections::{BTreeMap, BTreeSet};

const LIMIT: i64 = 5_600_000; // exact 0.01 MILS source ticks
pub const MAX_PADS: usize = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentPad {
    pub reference: String,
    pub pin: String,
    pub point: [i64; 2],
    pub component_origin: [i64; 2],
    pub rotation: u8,
    pub layer: u32,
    pub size_ticks: i64,
    pub square: bool,
    pub rectangular: bool,
    pub bottom: bool,
    pub width_ticks: i64,
    pub height_ticks: i64,
}

#[derive(Clone)]
struct Terminal {
    pin: String,
    point: [i64; 2],
}
#[derive(Clone, Copy)]
struct Stack {
    width: i64,
    height: i64,
    square: bool,
    rectangular: bool,
}
struct Decal {
    terminals: Vec<Terminal>,
    stacks: BTreeMap<String, Stack>,
}
pub struct PadImport {
    pub pads: Vec<ComponentPad>,
    pub skipped: usize,
}

fn words(line: &str) -> Vec<&str> {
    line.split_ascii_whitespace().collect()
}

fn fixed(value: &str) -> Option<i64> {
    let (neg, raw) = if let Some(s) = value.strip_prefix('-') {
        (true, s)
    } else if let Some(s) = value.strip_prefix('+') {
        (false, s)
    } else {
        (false, value)
    };
    let (whole, frac) = raw.split_once('.').unwrap_or((raw, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || frac.len() > 2
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let w = whole.parse::<i64>().ok()?;
    let f = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().ok()?.checked_mul(10)?,
        _ => frac.parse::<i64>().ok()?,
    };
    let n = w.checked_mul(100)?.checked_add(f)?;
    let v = if neg { n.checked_neg()? } else { n };
    if v.abs() > LIMIT {
        None
    } else {
        Some(v)
    }
}
fn safe_name(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
fn sections(source: &str, wanted: &str) -> Vec<String> {
    let mut active = false;
    let mut lines = Vec::new();
    for line in source.lines().skip(1) {
        let s = line.trim();
        let tag = s.split_ascii_whitespace().next().unwrap_or("");
        if tag == "*END*" {
            break;
        }
        if tag == "*REMARK*" || s.starts_with("REMARK ") || s.is_empty() {
            continue;
        }
        if tag.starts_with('*') {
            // *STANDARD* is an optional secondary directive within *PART*;
            // it is not the beginning of another PCB section.
            if active && wanted == "*PART*" && tag == "*STANDARD*" {
                continue;
            }
            active = tag == wanted;
            continue;
        }
        if active {
            lines.push(s.to_owned());
        }
    }
    lines
}
fn decals(lines: &[String]) -> BTreeMap<String, Decal> {
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i < lines.len() {
        let h = words(&lines[i]);
        // For an unsupported piece/text/label count, the following variable
        // number of lines cannot safely be re-synchronized: stop this section.
        if h.len() < 7
            || h.len() > 9
            || h[1] != "I"
            || h[4] != "0"
            || h.get(7).is_some_and(|s| *s != "0")
            || h.get(8).is_some_and(|s| *s != "0")
            || !safe_name(h[0], 40)
        {
            break;
        }
        let (Ok(term_count), Ok(stack_count)) = (h[5].parse::<usize>(), h[6].parse::<usize>())
        else {
            break;
        };
        if !(1..=128).contains(&term_count)
            || !(1..=128).contains(&stack_count)
            || i + term_count >= lines.len()
        {
            break;
        }
        let name = h[0].to_owned();
        i += 1;
        let mut terms = Vec::new();
        let mut good = true;
        let mut seen = BTreeSet::new();
        for _ in 0..term_count {
            let row = words(&lines[i]);
            i += 1;
            if row.len() != 5
                || !row[0].starts_with('T')
                || !safe_name(row[4], 7)
                || !seen.insert(row[4].to_owned())
            {
                good = false;
                continue;
            }
            let Some(x) = fixed(&row[0][1..]) else {
                good = false;
                continue;
            };
            let Some(y) = fixed(row[1]) else {
                good = false;
                continue;
            };
            if fixed(row[2]).is_none() || fixed(row[3]).is_none() {
                good = false;
                continue;
            }
            terms.push(Terminal {
                pin: row[4].to_owned(),
                point: [x, y],
            });
        }
        let mut stacks = BTreeMap::new();
        for _ in 0..stack_count {
            if i >= lines.len() {
                good = false;
                break;
            }
            let head = words(&lines[i]);
            i += 1;
            if head.len() != 3
                || head[0] != "PAD"
                || head[2] != "3"
                || !safe_name(head[1], 7)
                || i + 3 > lines.len()
            {
                good = false;
                break;
            }
            let mut rows = BTreeMap::new();
            for _ in 0..3 {
                let row = words(&lines[i]);
                i += 1;
                let finger = row.get(2) == Some(&"RF");
                if !matches!(row.get(2), Some(&"R" | &"S" | &"RF"))
                    || (finger && row.len() != 7)
                    || (!finger && row.len() != 4)
                    || (!finger && row[3] != "0")
                    || !matches!(row[0], "-2" | "-1" | "0")
                    || (finger && row[0] != "-2")
                {
                    good = false;
                    continue;
                }
                let Some(size) = fixed(row[1]) else {
                    good = false;
                    continue;
                };
                // PADS V9.4: level size RF finori finlength finoffset corner.
                // Inner diameter is omitted for the non-annular RF shape.
                // Only orthogonal, zero-offset, sharp-corner rectangular SMD
                // fingerprints are currently independently renderable.
                let (length, finger_turn) = if finger {
                    if fixed(row[5]) != Some(0) || fixed(row[6]) != Some(0) {
                        good = false;
                        continue;
                    }
                    let (Some(length), Some(angle)) = (fixed(row[4]), turn(row[3])) else {
                        good = false;
                        continue;
                    };
                    if !(100..=100_000).contains(&length) || angle > 1 {
                        good = false;
                        continue;
                    }
                    (length, angle)
                } else {
                    (size, 0)
                };
                if rows
                    .insert(row[0].to_owned(), (size, row[2], length, finger_turn))
                    .is_some()
                {
                    good = false;
                }
            }
            // SMD top only: any hole, inner or bottom conductive pad is excluded.
            if rows.len() != 3
                || rows.get("-1").map(|r| r.0) != Some(0)
                || rows.get("0").map(|r| r.0) != Some(0)
            {
                good = false;
                continue;
            }
            let Some(&(size, shape, length, finger_turn)) = rows.get("-2") else {
                good = false;
                continue;
            };
            if size <= 0
                || stacks
                    .insert(
                        head[1].to_owned(),
                        Stack {
                            width: if finger_turn == 1 { size } else { length },
                            height: if finger_turn == 1 { length } else { size },
                            square: shape == "S",
                            rectangular: shape == "RF",
                        },
                    )
                    .is_some()
            {
                good = false;
            }
        }
        if good && !out.contains_key(&name) {
            out.insert(
                name,
                Decal {
                    terminals: terms,
                    stacks,
                },
            );
        }
    }
    out
}
fn turn(s: &str) -> Option<u8> {
    // PADS allows up to three fractional decimal places in orientation.
    // Accept only exact orthogonal values with zero fractional digits.
    let whole = if let Some((whole, fraction)) = s.split_once('.') {
        if !(1..=3).contains(&fraction.len()) || !fraction.bytes().all(|b| b == b'0') {
            return None;
        }
        whole
    } else {
        s
    };
    match whole {
        "0" => Some(0),
        "90" => Some(1),
        "180" => Some(2),
        "270" => Some(3),
        _ => None,
    }
}
pub fn parse(source: &[u8], max_layer: u32) -> PadImport {
    let src = String::from_utf8_lossy(source);
    let defs = decals(&sections(&src, "*PARTDECAL*"));
    let mut type_to_decal = BTreeMap::new();
    for line in sections(&src, "*PARTTYPE*") {
        let fields = words(&line);
        // Restricted preview: do not generate fabricated pin identity when
        // gate/signal subrecords or alphanumeric mapping lines are present.
        // The documented dcalnm[:altdcal...] field chooses index 0 for
        // unmirrored PART records with alt=0, so alternate names are allowed
        // here without claiming their geometry is supported.
        if !(7..=8).contains(&fields.len())
            || !safe_name(fields[0], 40)
            || !safe_name(fields[2], 3)
            || fields[3] != "0"
            || fields[4] != "0"
            || fields[5] != "0"
            || fields[6] != "0"
            || fields.get(7).is_some_and(|eco| !matches!(*eco, "Y" | "N"))
            || type_to_decal.contains_key(fields[0])
        {
            continue;
        }
        let decals: Vec<_> = fields[1].split(':').collect();
        if decals.is_empty() || decals.len() > 16 || !decals.iter().all(|name| safe_name(name, 40))
        {
            continue;
        }
        type_to_decal.insert(fields[0].to_owned(), decals[0].to_owned());
    }
    let mut pads = Vec::new();
    let mut skipped = 0;
    let mut seen_refs = BTreeSet::new();
    for line in sections(&src, "*PART*") {
        let p = words(&line);
        // Documented PART suffix: clstid, clsattr, brotherid, labels.
        // Preserve ordinary unclustered/unlabeled exports (including partially
        // omitted defaults), but never reinterpret non-default references as
        // an independent simple pad with potentially different ownership.
        const DEFAULT_SUFFIX: [&str; 4] = ["-1", "0", "-1", "0"];
        if !(8..=12).contains(&p.len())
            || !safe_name(p[0], 15)
            || p[8..]
                .iter()
                .zip(DEFAULT_SUFFIX.iter())
                .any(|(actual, expected)| actual != expected)
            || !seen_refs.insert(p[0].to_owned())
        {
            skipped += 1;
            continue;
        }
        // A per-instance `TYPE@DECAL` override identifies a decal record,
        // never a new part type. A modified decal absent from the supported
        // definition set must fail closed rather than guessing geometry.
        let (type_name, decal_override) = if let Some((name, decal)) = p[1].split_once('@') {
            if !safe_name(name, 40) || !safe_name(decal, 40) {
                skipped += 1;
                continue;
            }
            (name, Some(decal))
        } else {
            (p[1], None)
        };
        let Some(primary_decal) = type_to_decal.get(type_name) else {
            skipped += 1;
            continue;
        };
        let decal_name = decal_override.unwrap_or(primary_decal.as_str());
        let Some(decal) = defs.get(decal_name) else {
            skipped += 1;
            continue;
        };
        // Only primary-decal, ordinary top/bottom placement is modeled;
        // unsupported ALT/cluster and nonorthogonal placement fail closed.
        if !matches!(p[5], "G" | "U")
            || !matches!(p[6], "N" | "M")
            || p[7] != "0"
            || (p[6] == "M" && max_layer < 2)
        {
            skipped += decal.terminals.len();
            continue;
        }
        let (Some(x), Some(y), Some(rot)) = (fixed(p[2]), fixed(p[3]), turn(p[4])) else {
            skipped += decal.terminals.len();
            continue;
        };
        let bottom = p[6] == "M";
        for term in &decal.terminals {
            if pads.len() >= MAX_PADS {
                skipped += 1;
                continue;
            }
            let Some(stack) = decal
                .stacks
                .get(&term.pin)
                .or_else(|| decal.stacks.get("0"))
            else {
                skipped += 1;
                continue;
            };
            let [a, b] = term.point;
            let delta = match rot {
                0 => [a, b],
                1 => [-b, a],
                2 => [-a, -b],
                _ => [b, -a],
            };
            // PADS PART bottom placement: rotate local coordinates first, then
            // flip horizontally about the component origin in the top-view.
            // This is the restricted LEFT_RIGHT footprint flip used by KiCad PADS.
            let dx = if bottom { -delta[0] } else { delta[0] };
            let Some(xx) = x.checked_add(dx) else {
                skipped += 1;
                continue;
            };
            let Some(yy) = y.checked_add(delta[1]) else {
                skipped += 1;
                continue;
            };
            if xx.abs() > LIMIT || yy.abs() > LIMIT {
                skipped += 1;
                continue;
            }
            pads.push(ComponentPad {
                reference: p[0].to_owned(),
                pin: term.pin.clone(),
                point: [xx, yy],
                component_origin: [x, y],
                rotation: rot,
                layer: if bottom { max_layer } else { 1 },
                size_ticks: stack.width,
                square: stack.square,
                rectangular: stack.rectangular,
                bottom,
                width_ticks: if rot % 2 == 0 {
                    stack.width
                } else {
                    stack.height
                },
                height_ticks: if rot % 2 == 0 {
                    stack.height
                } else {
                    stack.width
                },
            });
        }
    }
    PadImport { pads, skipped }
}
fn meta(source: &SourceId, kind: ObjectKind, key: &str, description: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, key, ""),
        provenance: GeometryProvenance {
            source_object_type: description.into(),
            history: vec![],
        },
    }
}
/// Legacy artwork-only import: a decal pin label alone never proves its Net.
pub fn augment(board: &mut BoardModel, imported: &PadImport) -> Result<(), &'static str> {
    augment_inner(board, imported, &BTreeMap::new())
}

/// Source ROUTE pin-pair declarations establish *declared* Pad/Pin Net ownership.
/// A graph edge still requires independently matching copper geometry and layer.
pub fn augment_source_bound(
    board: &mut BoardModel,
    imported: &PadImport,
    source_bytes: &[u8],
) -> Result<(), &'static str> {
    let assignments = declared_route_pin_nets(source_bytes)?;
    augment_inner(board, imported, &assignments)?;
    crate::connectivity_preview::augment_pad_centres(board)?;
    if board.validate().has_errors() {
        return Err("PAD_SOURCE_CONTACT_INVALID");
    }
    Ok(())
}

fn declared_route_pin_nets(source_bytes: &[u8]) -> Result<BTreeMap<String, String>, &'static str> {
    // This function is called only after route_preview::parse accepted the source.
    // Restrict declarations to the first END and the explicit ROUTE/SIGNAL section.
    let source = String::from_utf8_lossy(source_bytes);
    let mut in_route = false;
    let mut current_net: Option<&str> = None;
    let mut pins = BTreeMap::<String, String>::new();
    for line in source.lines().skip(1) {
        let line = line.trim();
        let fields: Vec<&str> = line.split_ascii_whitespace().collect();
        if fields.is_empty() || fields[0] == "*REMARK*" || fields[0] == "REMARK" {
            continue;
        }
        if fields[0] == "*END*" {
            break;
        }
        if fields[0].starts_with('*') {
            match fields[0] {
                "*ROUTE*" => {
                    in_route = true;
                    current_net = None;
                }
                "*SIGNAL*" if in_route => current_net = fields.get(1).copied(),
                _ => {
                    in_route = false;
                    current_net = None;
                }
            }
            continue;
        }
        if !in_route || fields.len() != 2 {
            continue;
        }
        let (Some(a), Some(b), Some(net)) = (
            fields[0].rsplit_once('.'),
            fields[1].rsplit_once('.'),
            current_net,
        ) else {
            continue;
        };
        for (reference, pin) in [a, b] {
            if reference.is_empty() || pin.is_empty() {
                continue;
            }
            let key = format!("{}.{}", reference, pin);
            if let Some(previous) = pins.insert(key, net.to_owned()) {
                if previous != net {
                    return Err("CONFLICTING_SOURCE_PIN_NET");
                }
            }
        }
    }
    Ok(pins)
}

fn augment_inner(
    board: &mut BoardModel,
    imported: &PadImport,
    assignments: &BTreeMap<String, String>,
) -> Result<(), &'static str> {
    if imported.pads.is_empty() {
        return Ok(());
    }
    let source = board.sources.first().ok_or("MISSING_SOURCE")?.id.clone();
    let frame = board.frames.first().ok_or("MISSING_FRAME")?.meta.id.clone();
    let mut references = BTreeMap::new();
    for pad in &imported.pads {
        let component_key = format!("p2c-component-{}", pad.reference);
        let component_id = ObjectId::new(&source, ObjectKind::Component, &component_key, "");
        if !references.contains_key(&pad.reference) {
            board.components.push(Component {
                meta: meta(
                    &source,
                    ObjectKind::Component,
                    &component_key,
                    "PART orthogonal origin/turn and explicit N(top)/M(bottom) flag; geometry preview only",
                ),
                reference: pad.reference.clone(),
                placement: Placement {
                    position: Point {
                        frame: frame.clone(),
                        ticks: pad.component_origin,
                    },
                    orientation: Orientation::Orthogonal {
                        // CoordinateFrame applies mirror_x before quarter-turn.
                        // PART bottom geometry is rotate-then-mirror, therefore
                        // R(-theta) * mirror_x == mirror_x * R(theta).
                        turn: match (pad.bottom, pad.rotation) {
                            (_, 0) => QuarterTurn::Zero,
                            (false, 1) | (true, 3) => QuarterTurn::One,
                            (_, 2) => QuarterTurn::Two,
                            (false, 3) | (true, 1) => QuarterTurn::Three,
                            _ => unreachable!("rotation was validated by parse"),
                        },
                        mirror_x: pad.bottom,
                    },
                    bottom_side: pad.bottom,
                },
            });
            references.insert(pad.reference.clone(), component_id.clone());
        }
        let key = format!("p2c-{}-{}", pad.reference, pad.pin);
        let stack_key = format!("p2c-padstack-{}", key);
        let stack_id = ObjectId::new(&source, ObjectKind::Padstack, &stack_key, "");
        let width = Rational::new(i128::from(pad.width_ticks) * 127, 500_000_000)
            .map_err(|_| "PAD_SIZE_OVERFLOW")?;
        let height = Rational::new(i128::from(pad.height_ticks) * 127, 500_000_000)
            .map_err(|_| "PAD_SIZE_OVERFLOW")?;
        let shape = if pad.square || pad.rectangular {
            PadShape::Rectangle {
                size_m: [width, height],
            }
        } else {
            PadShape::Circle { diameter_m: width }
        };
        let layer = ObjectId::new(
            &source,
            ObjectKind::Layer,
            &format!("layer-{}", pad.layer),
            "",
        );
        board.padstacks.push(Padstack {
            meta: meta(
                &source,
                ObjectKind::Padstack,
                &stack_key,
                "PADS PARTDECAL restricted SMD round/square/RF; PART top/bottom side mapped",
            ),
            entries: vec![PadstackLayer {
                layer,
                role: PadRole::Conductive,
                shape,
            }],
        });
        let source_pin = format!("{}.{}", pad.reference, pad.pin);
        let proven_net = assignments.get(&source_pin).and_then(|name| {
            board
                .nets
                .iter()
                .find(|net| net.name == *name)
                .map(|net| net.meta.id.clone())
        });
        let pad_key = format!("p2c-pad-{}", key);
        let pad_id = ObjectId::new(&source, ObjectKind::Pad, &pad_key, "");
        board.pads.push(Pad {
            meta: meta(
                &source,
                ObjectKind::Pad,
                &pad_key,
                "PADS PARTDECAL terminal + PART placement (top/bottom restricted)",
            ),
            position: Point {
                frame: frame.clone(),
                ticks: pad.point,
            },
            orientation: Orientation::Orthogonal {
                turn: QuarterTurn::Zero,
                mirror_x: false,
            },
            padstack: stack_id,
            net: proven_net.clone(),
        });
        board.pins.push(Pin {
            meta: meta(
                &source,
                ObjectKind::Pin,
                &format!("p2c-pin-{}", key),
                "PADS PARTDECAL pin; optional Net from explicit parsed ROUTE source declaration",
            ),
            component: component_id,
            number: pad.pin.clone(),
            pad: pad_id,
            net: proven_net,
        });
    }
    board.sources[0].capability.pin_pad = CapabilityState::Partial;
    board.sources[0].capability.object_coverage.push(
        "PARTDECAL/PARTTYPE/PART: restricted top/bottom drill-free R/S/RF SMD; only explicit ROUTE pair Net optionally declared; contact NOT_VERIFIED without source geometry"
            .into(),
    );
    if board.validate().has_errors() {
        return Err("PAD_CANONICAL_INVALID");
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = "!PADS-POWERPCB-V9.4-MILS!\n*PARTDECAL*\nD2 I 1000 1000 0 2 1 0 0\nT0 0 0 0 1\nT100 0 0 0 2\nPAD 0 3\n-2 40 R 0\n-1 0 R 0\n0 0 R 0\n*PARTTYPE*\nTYPEA D2 UND 0 0 0 0\n*PART*\nU1 TYPEA 200 300 90 U N 0\n*END*\n";
    #[test]
    fn import_exact_top_smd_positions_and_nonconnectivity() {
        let out = parse(SAMPLE.as_bytes(), 4);
        assert_eq!(out.skipped, 0);
        assert_eq!(out.pads.len(), 2);
        assert_eq!(out.pads[0].point, [20_000, 30_000]);
        assert_eq!(out.pads[1].point, [20_000, 40_000]);
        assert_eq!(out.pads[1].size_ticks, 4000);
        assert!(!out.pads[1].square);
    }
    #[test]
    fn unsupported_decals_and_mirrors_fail_closed() {
        assert!(
            parse(SAMPLE.replace("-2 40 R 0", "-2 40 RF 0").as_bytes(), 4)
                .pads
                .is_empty()
        );
        let bottom = parse(SAMPLE.replace("90 U N 0", "90 U M 0").as_bytes(), 4);
        assert_eq!(bottom.pads.len(), 2);
        assert_eq!(bottom.pads[0].layer, 4);
        assert!(bottom.pads[0].bottom);
        assert_eq!(bottom.pads[1].point, [20_000, 40_000]);
        assert!(parse(SAMPLE.replace("90 U N 0", "90 U M 0").as_bytes(), 1)
            .pads
            .is_empty());
        assert!(
            parse(SAMPLE.replace("0 2 1 0 0", "1 2 1 0 0").as_bytes(), 4)
                .pads
                .is_empty()
        );
        assert!(parse(SAMPLE.replace("90 U N 0", "90 0 0 0").as_bytes(), 4)
            .pads
            .is_empty());
        assert!(parse(SAMPLE.replace("90 U N 0", "90 U N 1").as_bytes(), 4)
            .pads
            .is_empty());
        assert_eq!(
            parse(SAMPLE.replace("90 U N 0", "90 G N 0").as_bytes(), 4)
                .pads
                .len(),
            2
        );
    }
    #[test]
    fn accepts_documented_part_suffix_defaults_without_inventing_contacts() {
        // Documented optional four fields: clstid clsattr brotherid labels.
        // Valid exports often include all four even for ordinary standalone parts.
        for suffix in ["", " -1", " -1 0", " -1 0 -1", " -1 0 -1 0"] {
            let source = SAMPLE.replace(
                "U1 TYPEA 200 300 90 U N 0",
                &format!("U1 TYPEA 200 300 90 U N 0{suffix}"),
            );
            let parsed = parse(source.as_bytes(), 4);
            assert_eq!(parsed.skipped, 0, "valid default suffix: {suffix}");
            assert_eq!(parsed.pads.len(), 2, "valid default suffix: {suffix}");
            assert_eq!(parsed.pads[0].point, [20_000, 30_000]);
        }
    }
    #[test]
    fn refuses_nondefault_cluster_brother_labels_and_excess_part_tokens() {
        for suffix in [
            " 0 0 -1 0",
            " -1 1 -1 0",
            " -1 0 1 0",
            " -1 0 -1 1",
            " -1 0 -1 0 EXTRA",
        ] {
            let source = SAMPLE.replace(
                "U1 TYPEA 200 300 90 U N 0",
                &format!("U1 TYPEA 200 300 90 U N 0{suffix}"),
            );
            let parsed = parse(source.as_bytes(), 4);
            assert!(parsed.pads.is_empty(), "unsupported suffix: {suffix}");
            assert!(parsed.skipped > 0);
        }
    }
    #[test]
    fn standard_directive_is_part_section_secondary_control() {
        let source = SAMPLE.replace("*PART*\n", "*PART*\n*STANDARD*\n");
        let parsed = parse(source.as_bytes(), 4);
        assert_eq!(parsed.skipped, 0);
        assert_eq!(parsed.pads.len(), 2);
        assert_eq!(parsed.pads[1].point, [20_000, 40_000]);
        let separated = SAMPLE.replace("*PART*\n", "*PART*\n*STANDARD*\n*ROUTE*\n");
        assert!(parse(separated.as_bytes(), 4).pads.is_empty());
    }
    #[test]
    fn accepts_primary_decal_from_alternate_list_only() {
        for list in ["D2:D3", "D2:D3:D4"] {
            let source = SAMPLE.replace("TYPEA D2 UND", &format!("TYPEA {list} UND"));
            let parsed = parse(source.as_bytes(), 4);
            assert_eq!(parsed.skipped, 0);
            assert_eq!(parsed.pads.len(), 2);
        }
        // Selecting an unmodeled alternate is deliberately not supported.
        let source = SAMPLE
            .replace("TYPEA D2 UND", "TYPEA D2:D3 UND")
            .replace("90 U N 0", "90 U N 1");
        assert!(parse(source.as_bytes(), 4).pads.is_empty());
        for list in ["D2:", ":D2", "D2::D3"] {
            let source = SAMPLE.replace("TYPEA D2 UND", &format!("TYPEA {list} UND"));
            assert!(parse(source.as_bytes(), 4).pads.is_empty());
        }
    }
    #[test]
    fn refuses_uninterpreted_alphanumeric_pins_and_gate_signal_subrecords() {
        for header in [
            "TYPEA D2 UND 0 0 2 0",
            "TYPEA D2 UND 1 0 0 0",
            "TYPEA D2 UND 0 1 0 0",
            "TYPEA D2 UND 0 0 0 32",
        ] {
            let source = SAMPLE.replace("TYPEA D2 UND 0 0 0 0", header);
            let parsed = parse(source.as_bytes(), 4);
            assert!(parsed.pads.is_empty(), "unsupported part type: {header}");
            assert!(parsed.skipped > 0);
        }
    }
    #[test]
    fn orthogonal_rotation_accepts_all_documented_zero_decimal_spellings() {
        for (name, quarter) in [("0", 0), ("90", 1), ("180", 2), ("270", 3)] {
            for suffix in ["", ".0", ".00", ".000"] {
                let orient = format!("{name}{suffix}");
                assert_eq!(turn(&orient), Some(quarter));
                let source = SAMPLE.replace("300 90 U N 0", &format!("300 {orient} U N 0"));
                let parsed = parse(source.as_bytes(), 4);
                assert_eq!(parsed.pads.len(), 2, "valid orientation {orient}");
            }
        }
        for orient in ["90.001", "90.0000", "90.", "45.000", "90.0.0", "91"] {
            assert!(turn(orient).is_none(), "unsupported orientation {orient}");
        }
    }
    #[test]
    fn per_part_decal_override_is_validated_against_supported_definitions() {
        let source = SAMPLE.replace("U1 TYPEA", "U1 TYPEA@D2");
        let parsed = parse(source.as_bytes(), 4);
        assert_eq!(parsed.skipped, 0);
        assert_eq!(parsed.pads.len(), 2);
        assert_eq!(parsed.pads[1].point, [20_000, 40_000]);
        for part in ["TYPEA@NOT_PRESENT", "TYPEA@D2@D3", "UNKNOWN@D2", "TYPEA@"] {
            let source = SAMPLE.replace("U1 TYPEA", &format!("U1 {part}"));
            let parsed = parse(source.as_bytes(), 4);
            assert!(parsed.pads.is_empty(), "unsupported part token: {part}");
            assert!(parsed.skipped > 0);
        }
    }
    #[test]
    fn pad_only_board_creates_canonical_objects_without_fake_net() {
        let source = SAMPLE.replace("*PARTDECAL*", "*PCB*\nMAXLAYER 4\n*PARTDECAL*");
        let route = crate::route_preview::parse(source.as_bytes()).unwrap();
        assert!(route.segments.is_empty());
        let mut model = crate::route_preview::into_canonical(source.as_bytes(), &route).unwrap();
        let parsed = parse(source.as_bytes(), route.max_layer);
        augment(&mut model, &parsed).unwrap();
        assert_eq!(model.pads.len(), 2);
        assert_eq!(model.pins.len(), 2);
        assert_eq!(model.components.len(), 1);
        assert!(model.pins.iter().all(|p| p.net.is_none()));
        assert!(model.pads.iter().all(|p| p.net.is_none()));
        assert!(!model.validate().has_errors());
    }
    #[test]
    fn restricted_rf_all_component_turns_and_finger_orientations() {
        let fixture = SAMPLE.replace("-2 40 R 0", "-2 30 RF 0 80 0 0");
        for (rotation, expected) in [
            ("0", (8000, 3000)),
            ("90", (3000, 8000)),
            ("180", (8000, 3000)),
            ("270", (3000, 8000)),
        ] {
            let source = fixture.replace("300 90 U N 0", &format!("300 {rotation} U N 0"));
            let parsed = parse(source.as_bytes(), 4);
            assert_eq!(parsed.pads.len(), 2);
            let pad = &parsed.pads[0];
            assert!(pad.rectangular);
            assert_eq!((pad.width_ticks, pad.height_ticks), expected);
            assert_eq!(pad.size_ticks, 8000);
        }
        let source = fixture.replace("-2 30 RF 0 80 0 0", "-2 30 RF 90 80 0 0");
        let parsed = parse(source.as_bytes(), 4);
        assert_eq!(parsed.pads.len(), 2);
        assert_eq!(
            (parsed.pads[0].width_ticks, parsed.pads[0].height_ticks),
            (8000, 3000)
        );
    }
    #[test]
    fn rejects_unmodeled_rf_extensions_and_preserves_other_pads() {
        let valid = SAMPLE.replace("-2 40 R 0", "-2 30 RF 0 80 0 0");
        for replacement in [
            "-2 30 RF 45 80 0 0",
            "-2 30 RF 180 80 0 0",
            "-2 30 RF 0 80 1 0",
            "-2 30 RF 0 80 0 1",
            "-2 30 RF 0 80 0 0 0",
            "-2 30 RF 0 0 0 0",
            "-2 30 RF 0 1001 0 0",
            "-2 30 RF 0 80 0",
        ] {
            let source = valid.replace("-2 30 RF 0 80 0 0", replacement);
            assert!(
                parse(source.as_bytes(), 4).pads.is_empty(),
                "invalid: {replacement}"
            );
        }
        // Canonical construction requires a real declared copper-layer count.
        let source = valid.replacen("*PARTDECAL*", "*PCB*\nMAXLAYER 4\n*PARTDECAL*", 1);
        let route = crate::route_preview::parse(source.as_bytes()).unwrap();
        let mut board = crate::route_preview::into_canonical(source.as_bytes(), &route).unwrap();
        augment(&mut board, &parse(source.as_bytes(), route.max_layer)).unwrap();
        assert!(!board.validate().has_errors());
        assert!(board.pins.iter().all(|pin| pin.net.is_none()));
    }
    #[test]
    fn bottom_mirror_post_rotation_and_layer_are_independent_of_top() {
        // Asymmetric two-terminal decal: a left/right flip must not be
        // mistaken for a Y flip or for rotation-first in the wrong direction.
        let src = SAMPLE.replace("300 90 U N 0", "300 0 U M 0");
        let zero = parse(src.as_bytes(), 4);
        assert_eq!(zero.pads[0].point, [20_000, 30_000]);
        assert_eq!(zero.pads[1].point, [10_000, 30_000]);
        for (angle, expected) in [
            ("0", [10_000, 30_000]),
            ("90", [20_000, 40_000]),
            ("180", [30_000, 30_000]),
            ("270", [20_000, 20_000]),
        ] {
            let sample = SAMPLE
                .replace("300 90 U N 0", &format!("300 {angle} U M 0"))
                .replace("*PARTDECAL*", "*PCB*\nMAXLAYER 4\n*PARTDECAL*");
            let imported = parse(sample.as_bytes(), 4);
            assert_eq!(imported.pads.len(), 2);
            assert_eq!(imported.pads[1].point, expected, "bottom {angle}");
            assert_eq!(imported.pads[1].layer, 4);
            assert!(imported.pads[1].bottom);
            let route = crate::route_preview::parse(sample.as_bytes()).unwrap();
            let mut model =
                crate::route_preview::into_canonical(sample.as_bytes(), &route).unwrap();
            augment(&mut model, &imported).unwrap();
            assert_eq!(model.components.len(), 1);
            assert!(model.components[0].placement.bottom_side);
            // Independent canonical reconstruction: CoordinateFrame follows
            // mirror-first conventions, unlike the source PART bottom geometry.
            let mut local_frame = model.frames[0].clone();
            local_frame.orientation = model.components[0].placement.orientation.clone();
            let origin = model.components[0].placement.position.ticks;
            local_frame.origin_m = [
                Rational::new(i128::from(origin[0]) * 127, 500_000_000).unwrap(),
                Rational::new(i128::from(origin[1]) * 127, 500_000_000).unwrap(),
            ];
            let canonical = local_frame
                .exact_m(&Point {
                    frame: local_frame.meta.id.clone(),
                    ticks: [10_000, 0],
                })
                .unwrap();
            let expected_point = imported.pads[1].point;
            assert_eq!(
                canonical,
                [
                    Rational::new(i128::from(expected_point[0]) * 127, 500_000_000).unwrap(),
                    Rational::new(i128::from(expected_point[1]) * 127, 500_000_000).unwrap(),
                ],
                "canonical transform disagrees with imported bottom pad at {angle}"
            );
            assert!(model.pins.iter().all(|p| p.net.is_none()));
            assert!(model.pads.iter().all(|p| p.net.is_none()));
            assert!(!model.validate().has_errors());
        }
    }
    #[test]
    fn square_only_not_unsupported_asymmetric_rectangle() {
        let x = parse(SAMPLE.replace("-2 40 R 0", "-2 40 S 0").as_bytes(), 4);
        assert_eq!(x.pads.len(), 2);
        assert!(x.pads[0].square);
    }
}
