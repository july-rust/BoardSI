//! D_056_04: lossless, bounded, versioned Canonical via-padstack geometry IPC.
//! This carries geometry and source declarations, never fabricated barrel continuity or SI.
use boardsi_model::{
    BoardModel, LayerKind, ObjectId, PadRole, PadShape, Rational, SourceFormat, ViaKind,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

pub const MAX_WIRE_BYTES: usize = 4 * 1024 * 1024;
const MAX_VIAS: usize = 10_000;
const MAX_PADS: usize = 640_000;

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("infallible string");
    }
    out
}
fn id(id: &ObjectId) -> Result<String, &'static str> {
    if id.source_object.as_bytes().contains(&0)
        || id.part.as_bytes().contains(&0)
        || id.source_object.len() + id.part.len() > 512
    {
        return Err("D05604_INVALID_OBJECT_ID");
    }
    let mut bytes = id.source_object.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend_from_slice(id.part.as_bytes());
    Ok(hex(&bytes))
}
fn rat(r: Rational) -> String {
    format!("{}:{}", r.numerator(), r.denominator())
}
fn layer_number(layers: &BTreeMap<ObjectId, usize>, id: &ObjectId) -> Result<usize, &'static str> {
    layers.get(id).copied().ok_or("D05604_UNMAPPED_LAYER")
}
fn fmt_source(f: &SourceFormat) -> String {
    match f {
        SourceFormat::PadsAsc => "PADS_ASC".into(),
        SourceFormat::Odb => "ODB".into(),
        SourceFormat::Ipc2581 => "IPC2581".into(),
        SourceFormat::KiCad => "KICAD".into(),
        SourceFormat::Gerber => "GERBER".into(),
        SourceFormat::Synthetic => "SYNTHETIC".into(),
        SourceFormat::Other(s) => format!("OTHER:{s}"),
    }
}
fn shape(shape: &PadShape) -> Result<(&'static str, Rational, Rational), &'static str> {
    let zero = Rational::integer(0);
    match shape {
        PadShape::Circle { diameter_m } if diameter_m.positive() => {
            Ok(("CIRCLE", *diameter_m, zero))
        }
        PadShape::Rectangle { size_m } if size_m[0].positive() && size_m[1].positive() => {
            Ok(("RECT", size_m[0], size_m[1]))
        }
        PadShape::Annulus {
            outer_diameter_m,
            inner_diameter_m,
        } if outer_diameter_m.positive()
            && inner_diameter_m.positive()
            && inner_diameter_m.checked_cmp(*outer_diameter_m).ok()
                == Some(std::cmp::Ordering::Less) =>
        {
            Ok(("ANNULUS", *outer_diameter_m, *inner_diameter_m))
        }
        _ => Err("D05604_UNSUPPORTED_OR_INVALID_PAD_SHAPE"),
    }
}

/// Only one complete, source-identified file is supported by V1.
/// Multi-file source assemblies and Custom boundaries must fail closed, not lose information.
pub fn encode(board: &BoardModel) -> Result<Vec<u8>, &'static str> {
    if board.validate().has_errors() {
        return Err("D05604_INVALID_CANONICAL_MODEL");
    }
    if board.sources.len() != 1 || board.sources[0].id.files.len() != 1 {
        return Err("D05604_MULTIFILE_SOURCE_UNSUPPORTED");
    }
    let source = &board.sources[0].id;
    let file = &source.files[0];
    if file.logical_path.starts_with('/')
        || file.logical_path.contains('\\')
        || file.logical_path.contains("..")
    {
        return Err("D05604_UNSAFE_SOURCE_PATH");
    }
    if board.vias.len() > MAX_VIAS
        || board.padstacks.len() > MAX_VIAS
        || board.drills.len() > MAX_VIAS
        || board.frames.len() > 1024
        || source.conditions.len() > 1024
    {
        return Err("D05604_OBJECT_LIMIT");
    }
    let mut physical: Vec<_> = board
        .stackup
        .layers
        .iter()
        .filter(|l| l.kind == LayerKind::Copper)
        .collect();
    physical.sort_by_key(|l| l.order);
    if !(2..=64).contains(&physical.len()) {
        return Err("D05604_LAYER_LIMIT");
    }
    let mut layer_map = BTreeMap::new();
    for (index, l) in physical.iter().enumerate() {
        if layer_map.insert(l.meta.id.clone(), index + 1).is_some() {
            return Err("D05604_DUPLICATE_LAYER");
        }
    }
    let mut padstacks: Vec<_> = board.padstacks.iter().collect();
    padstacks.sort_by_key(|s| (&s.meta.id.source_object, &s.meta.id.part));
    let mut drills: Vec<_> = board.drills.iter().collect();
    drills.sort_by_key(|s| (&s.meta.id.source_object, &s.meta.id.part));
    let mut vias: Vec<_> = board.vias.iter().collect();
    vias.sort_by_key(|s| (&s.meta.id.source_object, &s.meta.id.part));
    let mut frames: Vec<_> = board.frames.iter().collect();
    frames.sort_by_key(|s| (&s.meta.id.source_object, &s.meta.id.part));
    let mut seen = BTreeSet::new();
    let mut pad_count: usize = 0;
    let mut body = String::new();
    for (k, v) in &source.conditions {
        writeln!(&mut body, "I|{}|{}", hex(k.as_bytes()), hex(v.as_bytes()))
            .map_err(|_| "D05604_WIRE_FORMAT")?;
    }
    for frame in &frames {
        let (turn, mirror) = match frame.orientation {
            boardsi_model::Orientation::Orthogonal { turn, mirror_x } => {
                let t = match turn {
                    boardsi_model::QuarterTurn::Zero => 0,
                    boardsi_model::QuarterTurn::One => 1,
                    boardsi_model::QuarterTurn::Two => 2,
                    boardsi_model::QuarterTurn::Three => 3,
                };
                (t, mirror_x)
            }
            _ => return Err("D05604_UNSUPPORTED_FRAME_ORIENTATION"),
        };
        writeln!(
            &mut body,
            "F|{}|{}|{}|{}|{}|{}|{}|{}",
            id(&frame.meta.id)?,
            hex(frame.unit_label.as_bytes()),
            rat(frame.metres_per_tick),
            rat(frame.origin_m[0]),
            rat(frame.origin_m[1]),
            turn,
            mirror,
            frame.source_layer_order_reversed
        )
        .map_err(|_| "D05604_WIRE_FORMAT")?;
    }
    for (index, layer) in physical.iter().enumerate() {
        writeln!(
            &mut body,
            "L|{}|{}|{}",
            index + 1,
            id(&layer.meta.id)?,
            layer.order
        )
        .map_err(|_| "D05604_WIRE_FORMAT")?;
    }
    for stack in padstacks {
        if !seen.insert(id(&stack.meta.id)?) {
            return Err("D05604_DUPLICATE_PADSTACK");
        }
        pad_count = pad_count
            .checked_add(stack.entries.len())
            .ok_or("D05604_PAD_LIMIT")?;
        if pad_count > MAX_PADS || stack.entries.len() > 64 {
            return Err("D05604_PAD_LIMIT");
        }
        let sid = id(&stack.meta.id)?;
        writeln!(&mut body, "S|{}|{}", sid, stack.entries.len())
            .map_err(|_| "D05604_WIRE_FORMAT")?;
        let mut used = BTreeSet::new();
        for pad in &stack.entries {
            let layer = layer_number(&layer_map, &pad.layer)?;
            if !used.insert(layer) {
                return Err("D05604_DUPLICATE_PAD_LAYER");
            }
            let role = match pad.role {
                PadRole::Conductive => "CONDUCTIVE",
                PadRole::AntiPad => "ANTIPAD",
                PadRole::Clearance => "CLEARANCE",
            };
            match &pad.shape {
                PadShape::Thermal {
                    round, centre_size_m, outer_size_m, spoke_width_m,
                    first_spoke_degrees, spoke_count,
                } => {
                    if pad.role != PadRole::Conductive {
                        return Err("D05606_THERMAL_MUST_BE_CONDUCTIVE");
                    }
                    writeln!(
                        &mut body, "P|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                        sid, layer, role,
                        if *round { "THERMAL_RT" } else { "THERMAL_ST" },
                        rat(*centre_size_m), rat(*outer_size_m), rat(*spoke_width_m),
                        first_spoke_degrees, spoke_count,
                    ).map_err(|_| "D05606_WIRE_FORMAT")?;
                }
                _ => {
                    let (code, a, b) = shape(&pad.shape)?;
                    writeln!(
                        &mut body, "P|{}|{}|{}|{}|{}|{}",
                        sid, layer, role, code, rat(a), rat(b)
                    ).map_err(|_| "D05604_WIRE_FORMAT")?;
                }
            }
        }
    }
    seen.clear();
    for drill in drills {
        let did = id(&drill.meta.id)?;
        if !seen.insert(did.clone()) {
            return Err("D05604_DUPLICATE_DRILL");
        }
        writeln!(
            &mut body,
            "D|{}|{}|{}|{}|{}|PLATING_UNVERIFIED",
            did,
            rat(drill.diameter_m),
            layer_number(&layer_map, &drill.span.first_layer)?,
            layer_number(&layer_map, &drill.span.last_layer)?,
            drill.plated
        )
        .map_err(|_| "D05604_WIRE_FORMAT")?;
    }
    seen.clear();
    for via in vias {
        let vid = id(&via.meta.id)?;
        if !seen.insert(vid.clone()) {
            return Err("D05604_DUPLICATE_VIA");
        }
        if !via.connected_layers.is_empty() {
            return Err("D05604_CONNECTIVITY_EVIDENCE_UNSUPPORTED");
        }
        let kind = match via.kind {
            ViaKind::Through => "THROUGH",
            ViaKind::Blind => "BLIND",
            ViaKind::Buried => "BURIED",
            ViaKind::Microvia => "MICROVIA_DECLARED",
        };
        let net = match &via.net {
            Some(x) => id(x)?,
            None => "-".into(),
        };
        writeln!(
            &mut body,
            "V|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|CONNECTIVITY_UNKNOWN",
            vid,
            id(&via.padstack)?,
            id(&via.drill)?,
            net,
            id(&via.position.frame)?,
            via.position.ticks[0],
            via.position.ticks[1],
            layer_number(&layer_map, &via.span.first_layer)?,
            layer_number(&layer_map, &via.span.last_layer)?,
            kind
        )
        .map_err(|_| "D05604_WIRE_FORMAT")?;
    }
    writeln!(
        &mut body,
        "END|{}|SOURCE_ONLY|GLOBAL_CONNECTIVITY_NOT_VERIFIED",
        board.vias.len()
    )
    .map_err(|_| "D05604_WIRE_FORMAT")?;
    let has_thermal = board.padstacks.iter().any(|s| s.entries.iter().any(
        |p| matches!(p.shape, PadShape::Thermal { .. })
    ));
    let version = if has_thermal { "BOARDSI_CANONICAL_PADSTACK_V2" }
        else { "BOARDSI_CANONICAL_PADSTACK_V1" };
    let body_sha = hex(&crate::digest::sha256(body.as_bytes()));
    let mut header = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|GEOMETRY_ONLY|SI_NOT_RUN\n",
        version, hex(&file.sha256), hex(fmt_source(&source.format).as_bytes()),
        hex(source.format_version.as_bytes()), hex(source.importer_version.as_bytes()),
        hex(file.logical_path.as_bytes()), frames.len(), physical.len(),
        board.padstacks.len(), board.drills.len(), board.vias.len(),
        source.conditions.len(), body.len(), body.lines().count(), body_sha
    );
    header.push_str(&body);
    if header.len() > MAX_WIRE_BYTES {
        return Err("D05604_OUTPUT_LIMIT");
    }
    Ok(header.into_bytes())
}

/// Accept exact PADS source bytes only, reimport and verify the resulting canonical SHA.
pub fn encode_pads_source(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    if bytes.len() > crate::route_preview::MAX_FILE_BYTES {
        return Err("D05604_SOURCE_LIMIT");
    }
    let parsed = crate::route_preview::parse_generalized(bytes)?;
    if parsed.skipped != 0 {
        return Err("D05604_SOURCE_COVERAGE_PARTIAL");
    }
    let board = crate::route_preview::into_canonical(bytes, &parsed)?;
    if board.sources.len() != 1
        || board.sources[0].id.files.len() != 1
        || board.sources[0].id.files[0].sha256 != crate::digest::sha256(bytes)
    {
        return Err("D05604_SOURCE_SHA_MISMATCH");
    }
    encode(&board)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &[u8] = include_bytes!("../tests/fixtures/pads_via_round_through_v94.asc");
    #[test]
    fn source_canonical_wire_reimport_and_hash() {
        let wire = encode_pads_source(SAMPLE).unwrap();
        let text = std::str::from_utf8(&wire).unwrap();
        assert!(text.starts_with("BOARDSI_CANONICAL_PADSTACK_V1|"));
        assert!(text.contains("|CONNECTIVITY_UNKNOWN\n"));
        assert!(text.contains("|PLATING_UNVERIFIED\n"));
        assert_eq!(wire, encode_pads_source(SAMPLE).unwrap());
        let changed = String::from_utf8(SAMPLE.to_vec())
            .unwrap()
            .replace("NET_A", "NET_B");
        let other = encode_pads_source(changed.as_bytes()).unwrap();
        assert_ne!(wire, other);
    }
    #[test]
    fn generic_layer_number_and_shapes_preserve_missing_landing() {
        for layers in [2, 4, 6, 8, 64] {
            let src = String::from_utf8(SAMPLE.to_vec())
                .unwrap()
                .replace("MAXLAYER 4", &format!("MAXLAYER {layers}"));
            let mut board = crate::route_preview::into_canonical(
                src.as_bytes(),
                &crate::route_preview::parse_generalized(src.as_bytes()).unwrap(),
            )
            .unwrap();
            let stack = &mut board.padstacks[0];
            // Preserve only declared end landings; intermediate layers remain absent.
            stack.entries.retain(|p| {
                p.layer.source_object == "layer-1" || p.layer.source_object == "layer-2"
            });
            let encoded = encode(&board).unwrap();
            let s = std::str::from_utf8(&encoded).unwrap();
            assert!(s.contains("|CIRCLE|"));
            assert!(s.contains("GLOBAL_CONNECTIVITY_NOT_VERIFIED"));
        }
    }
}
