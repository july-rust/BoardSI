//! A_011 P2-C: deliberately narrow, fail-closed V9.4 MILS component-pad preview.
//! Accepts zero-outline I-unit decals with explicit T terminals and 3-layer
//! drill-free round or square SMD padstacks. Does NOT infer connectivity.
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
}

#[derive(Clone)]
struct Terminal {
    pin: String,
    point: [i64; 2],
}
#[derive(Clone, Copy)]
struct Stack {
    size: i64,
    square: bool,
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
                if row.len() != 4
                    || !matches!(row[2], "R" | "S")
                    || row[3] != "0"
                    || !matches!(row[0], "-2" | "-1" | "0")
                {
                    good = false;
                    continue;
                }
                let Some(size) = fixed(row[1]) else {
                    good = false;
                    continue;
                };
                if rows.insert(row[0].to_owned(), (size, row[2])).is_some() {
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
            let Some(&(size, shape)) = rows.get("-2") else {
                good = false;
                continue;
            };
            if size <= 0
                || stacks
                    .insert(
                        head[1].to_owned(),
                        Stack {
                            size,
                            square: shape == "S",
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
    match s {
        "0" | "0.0" => Some(0),
        "90" | "90.0" => Some(1),
        "180" | "180.0" => Some(2),
        "270" | "270.0" => Some(3),
        _ => None,
    }
}
pub fn parse(source: &[u8], max_layer: u32) -> PadImport {
    let src = String::from_utf8_lossy(source);
    let defs = decals(&sections(&src, "*PARTDECAL*"));
    let mut type_to_decal = BTreeMap::new();
    for line in sections(&src, "*PARTTYPE*") {
        let fields = words(&line);
        if fields.len() >= 7
            && safe_name(fields[0], 40)
            && safe_name(fields[1], 40)
            && !type_to_decal.contains_key(fields[0])
        {
            type_to_decal.insert(fields[0].to_owned(), fields[1].to_owned());
        }
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
        let Some(decal_name) = type_to_decal.get(p[1]) else {
            skipped += 1;
            continue;
        };
        let Some(decal) = defs.get(decal_name) else {
            skipped += 1;
            continue;
        };
        // Do not mistake an alternate-footprint selection or mirrored bottom
        // placement for the verified primary unmirrored top case.
        if !matches!(p[5], "G" | "U") || p[6] != "N" || p[7] != "0" {
            skipped += decal.terminals.len();
            continue;
        }
        let (Some(x), Some(y), Some(rot)) = (fixed(p[2]), fixed(p[3]), turn(p[4])) else {
            skipped += decal.terminals.len();
            continue;
        };
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
            let Some(xx) = x.checked_add(delta[0]) else {
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
                layer: 1.min(max_layer),
                size_ticks: stack.size,
                square: stack.square,
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
pub fn augment(board: &mut BoardModel, imported: &PadImport) -> Result<(), &'static str> {
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
                    "PART unmirrored / verified component origin and turn",
                ),
                reference: pad.reference.clone(),
                placement: Placement {
                    position: Point {
                        frame: frame.clone(),
                        ticks: pad.component_origin,
                    },
                    orientation: Orientation::Orthogonal {
                        turn: match pad.rotation {
                            0 => QuarterTurn::Zero,
                            1 => QuarterTurn::One,
                            2 => QuarterTurn::Two,
                            _ => QuarterTurn::Three,
                        },
                        mirror_x: false,
                    },
                    bottom_side: false,
                },
            });
            references.insert(pad.reference.clone(), component_id.clone());
        }
        let key = format!("p2c-{}-{}", pad.reference, pad.pin);
        let stack_key = format!("p2c-padstack-{}", key);
        let stack_id = ObjectId::new(&source, ObjectKind::Padstack, &stack_key, "");
        let length = Rational::new(i128::from(pad.size_ticks) * 127, 500_000_000)
            .map_err(|_| "PAD_SIZE_OVERFLOW")?;
        let shape = if pad.square {
            PadShape::Rectangle {
                size_m: [length, length],
            }
        } else {
            PadShape::Circle { diameter_m: length }
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
                "PADS PARTDECAL top-only SMD round/square",
            ),
            entries: vec![PadstackLayer {
                layer,
                role: PadRole::Conductive,
                shape,
            }],
        });
        let pad_key = format!("p2c-pad-{}", key);
        let pad_id = ObjectId::new(&source, ObjectKind::Pad, &pad_key, "");
        board.pads.push(Pad {
            meta: meta(
                &source,
                ObjectKind::Pad,
                &pad_key,
                "PADS PARTDECAL terminal + PART placement (top-only)",
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
            net: None,
        });
        board.pins.push(Pin {
            meta: meta(
                &source,
                ObjectKind::Pin,
                &format!("p2c-pin-{}", key),
                "PADS PARTDECAL pin ID, net not verified",
            ),
            component: component_id,
            number: pad.pin.clone(),
            pad: pad_id,
            net: None,
        });
    }
    board.sources[0].capability.pin_pad = CapabilityState::Partial;
    board.sources[0].capability.object_coverage.push(
        "PARTDECAL/PARTTYPE/PART: top only drill-free round/square SMD; net NOT_VERIFIED".into(),
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
        assert!(parse(SAMPLE.replace("90 U N 0", "90 U M 0").as_bytes(), 4)
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
            " 0 0 -1 0", " -1 1 -1 0", " -1 0 1 0",
            " -1 0 -1 1", " -1 0 -1 0 EXTRA",
            " -1 0 -1",
        ] {
            // A complete but truncated default suffix is valid and covered above;
            // every changed value / extra token is not.
            if suffix == " -1 0 -1" { continue; }
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
    fn square_only_not_unsupported_asymmetric_rectangle() {
        let x = parse(SAMPLE.replace("-2 40 R 0", "-2 40 S 0").as_bytes(), 4);
        assert_eq!(x.pads.len(), 2);
        assert!(x.pads[0].square);
    }
}
