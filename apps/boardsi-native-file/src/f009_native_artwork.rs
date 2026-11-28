//! F_009: source-free, explicitly bounded PADS/ODB Native Artwork replay.
//! This is a derived display-only receipt, never a second persistence format,
//! electrical connectivity result, or evidence that original CAD is available.
use boardsi_model::{
    BoardModel, LayerKind, ObjectId, Orientation, PadRole, PadShape, Point, QuarterTurn, Rational,
    SourceFormat,
};
use serde_json::{json, Value};

const MAX_LAYERS: usize = 32;
const MAX_COORD_NM: i64 = 1_000_000_000_000;
const MAX_SIZE_NM: i64 = 1_000_000_000;

fn exact_nm(value: Rational) -> Result<i64, &'static str> {
    let scaled = value
        .numerator()
        .checked_mul(1_000_000_000)
        .ok_or("BSI_F009_DIMENSION_OVERFLOW")?;
    if scaled % value.denominator() != 0 {
        return Err("BSI_F009_SUB_NM_UNREPRESENTABLE");
    }
    i64::try_from(scaled / value.denominator()).map_err(|_| "BSI_F009_DIMENSION_OVERFLOW")
}

fn size_nm(value: Rational) -> Result<i64, &'static str> {
    let n = exact_nm(value)?;
    if n <= 0 || n > MAX_SIZE_NM {
        return Err("BSI_F009_SIZE_LIMIT");
    }
    Ok(n)
}

fn point_nm(board: &BoardModel, point: &Point) -> Result<[i64; 2], &'static str> {
    let frame = board
        .frames
        .iter()
        .find(|frame| frame.meta.id == point.frame)
        .ok_or("BSI_F009_FRAME_MISSING")?;
    let coords = frame
        .regrid(
            point,
            Rational::new(1, 1_000_000_000).map_err(|_| "BSI_F009_FRAME")?,
        )
        .map_err(|_| "BSI_F009_FRAME_UNREPRESENTABLE")?;
    if coords
        .iter()
        .any(|n| *n < -MAX_COORD_NM || *n > MAX_COORD_NM)
    {
        return Err("BSI_F009_COORD_LIMIT");
    }
    Ok(coords)
}

fn name<'a>(board: &'a BoardModel, id: &ObjectId) -> Result<&'a str, &'static str> {
    board
        .stackup
        .layers
        .iter()
        .find(|l| l.meta.id == *id && l.kind == LayerKind::Copper)
        .map(|l| l.name.as_str())
        .ok_or("BSI_F009_COPPER_LAYER_MISSING")
}

fn net(id: Option<&ObjectId>) -> &str {
    id.map(|n| n.source_object.as_str()).unwrap_or("UNKNOWN")
}

/// Strictly reconstruct display geometry from the validated native Reader model.
/// Err means the native file remains structurally openable but Artwork is NOT_RENDERED.
pub(crate) fn render(board: &BoardModel) -> Result<(&'static str, Value), &'static str> {
    if board.sources.len() != 1 || board.validate().has_errors() {
        return Err("BSI_F009_UNQUALIFIED_CANONICAL");
    }
    let (format, schema) = match board.sources[0].id.format {
        SourceFormat::PadsAsc if board.sources[0].id.format_version == "V9.4-MILS" => {
            ("PADS_V94_SOURCE_SUBSET", "BSI_F009_NATIVE_CAD_PREVIEW_V1")
        }
        SourceFormat::Odb => ("ODB_SOURCE_SUBSET", "BSI_F009_NATIVE_CAD_PREVIEW_V1"),
        _ => return Err("BSI_F009_UNSUPPORTED_SOURCE"),
    };
    if !board.arcs.is_empty()
        || !board.areas.is_empty()
        || !board.outlines.is_empty()
        || !board.components.is_empty()
        || !board.pins.is_empty()
    {
        return Err("BSI_F009_UNRENDERED_GEOMETRY_KIND");
    }
    if board.traces.len() + board.pads.len() + board.vias.len() == 0
        || board.traces.len() + board.pads.len() + board.vias.len()
            > crate::f011_artwork_stream::item_limit()
    {
        return Err("BSI_F009_ITEM_LIMIT");
    }
    let mut layers = board
        .stackup
        .layers
        .iter()
        .filter(|l| l.kind == LayerKind::Copper)
        .collect::<Vec<_>>();
    layers.sort_by_key(|l| l.order);
    if layers.len() < 2 || layers.len() > MAX_LAYERS {
        return Err("BSI_F009_LAYER_LIMIT");
    }
    if layers
        .iter()
        .any(|l| l.name.is_empty() || l.name == "*" || l.name == "All copper layers")
        || layers.windows(2).any(|pair| pair[0].name == pair[1].name)
    {
        return Err("BSI_F009_LAYER_NAME");
    }
    let mut items = Vec::new();
    for (index, trace) in board.traces.iter().enumerate() {
        if trace.centreline.len() != 2 {
            return Err("BSI_F009_TRACE_SHAPE");
        }
        let width = size_nm(trace.width_m)?;
        let a = point_nm(board, &trace.centreline[0])?;
        let b = point_nm(board, &trace.centreline[1])?;
        if a == b {
            return Err("BSI_F009_DEGENERATE_TRACE");
        }
        items.push(json!({
            "kind": "trace", "id": trace.meta.id.source_object,
            "inspection_ref": crate::f012_inspection::artwork_ref(board,"traces",index,&trace.meta.id),
            "layer": name(board, &trace.layer)?,
            "net": net(trace.net.as_ref()), "a": a, "b": b, "width_nm": width,
            "contact": "NOT_VERIFIED"
        }));
    }
    for (index, pad) in board.pads.iter().enumerate() {
        if !matches!(
            pad.orientation,
            Orientation::Orthogonal {
                turn: QuarterTurn::Zero,
                mirror_x: false
            }
        ) {
            return Err("BSI_F009_PAD_ORIENTATION");
        }
        let stack = board
            .padstacks
            .iter()
            .find(|s| s.meta.id == pad.padstack)
            .ok_or("BSI_F009_PADSTACK_MISSING")?;
        if stack.entries.len() != 1 || stack.entries[0].role != PadRole::Conductive {
            return Err("BSI_F009_PADSTACK_UNSUPPORTED");
        }
        let entry = &stack.entries[0];
        let at = point_nm(board, &pad.position)?;
        let layer = name(board, &entry.layer)?;
        let common = (pad.meta.id.source_object.as_str(), net(pad.net.as_ref()));
        let mut row = match &entry.shape {
            PadShape::Circle { diameter_m } => json!({
                "kind": "flash", "id": common.0, "layer": layer, "net": common.1,
                "at": at, "diameter_nm": size_nm(*diameter_m)?,
                "contact": "NOT_VERIFIED"
            }),
            PadShape::Rectangle { size_m } => json!({
                "kind": "rectangle", "id": common.0, "layer": layer, "net": common.1,
                "at": at, "width_nm": size_nm(size_m[0])?,
                "height_nm": size_nm(size_m[1])?, "contact": "NOT_VERIFIED"
            }),
            _ => return Err("BSI_F009_PAD_SHAPE_UNSUPPORTED"),
        };
        row["inspection_ref"] = crate::f012_inspection::artwork_ref(board, "pads", index, &pad.meta.id);
        items.push(row);
    }
    for (index, via) in board.vias.iter().enumerate() {
        // Native models may already contain source-local connectivity evidence.
        // Rendering the source-defined copper NEVER promotes that evidence to
        // manufactured/verified physical contact or solver readiness.
        let stack = board
            .padstacks
            .iter()
            .find(|s| s.meta.id == via.padstack)
            .ok_or("BSI_F009_VIA_PADSTACK_MISSING")?;
        if stack.entries.is_empty() {
            return Err("BSI_F009_VIA_PADSTACK_EMPTY");
        }
        let diameter = match &stack.entries[0].shape {
            PadShape::Circle { diameter_m } => size_nm(*diameter_m)?,
            _ => return Err("BSI_F009_VIA_SHAPE_UNSUPPORTED"),
        };
        if stack.entries.iter().any(|entry| {
            entry.role != PadRole::Conductive
                || !matches!(&entry.shape, PadShape::Circle { diameter_m }
                    if size_nm(*diameter_m).ok() == Some(diameter))
                || name(board, &entry.layer).is_err()
        }) {
            return Err("BSI_F009_VIA_LAYER_SHAPE_UNSUPPORTED");
        }
        let first_order = board
            .stackup
            .layers
            .iter()
            .find(|l| l.meta.id == via.span.first_layer)
            .ok_or("BSI_F009_VIA_SPAN_LAYER")?
            .order;
        let last_order = board
            .stackup
            .layers
            .iter()
            .find(|l| l.meta.id == via.span.last_layer)
            .ok_or("BSI_F009_VIA_SPAN_LAYER")?
            .order;
        if first_order > last_order {
            return Err("BSI_F009_VIA_SPAN_ORDER");
        }
        let mut pad_layers = Vec::new();
        for entry in &stack.entries {
            let layer = board
                .stackup
                .layers
                .iter()
                .find(|l| l.meta.id == entry.layer && l.kind == LayerKind::Copper)
                .ok_or("BSI_F009_VIA_COPPER_LAYER")?;
            if layer.order < first_order
                || layer.order > last_order
                || pad_layers.contains(&layer.name.as_str())
            {
                return Err("BSI_F009_VIA_LAYER_INVENTORY");
            }
            pad_layers.push(layer.name.as_str());
        }
        let drill = board
            .drills
            .iter()
            .find(|d| d.meta.id == via.drill)
            .ok_or("BSI_F009_VIA_DRILL_MISSING")?;
        let drill_nm = size_nm(drill.diameter_m)?;
        if drill_nm >= diameter {
            return Err("BSI_F009_VIA_ANNULUS_INVALID");
        }
        items.push(json!({
            "kind": "via", "id": via.meta.id.source_object,
            "inspection_ref": crate::f012_inspection::artwork_ref(board,"vias",index,&via.meta.id),
            "layer": "*", "net": net(via.net.as_ref()),
            "at": point_nm(board, &via.position)?,
            "diameter_nm": diameter, "drill_nm": drill_nm,
            "span_first": name(board, &via.span.first_layer)?,
            "span_last": name(board, &via.span.last_layer)?,
            "pad_layers": pad_layers, "contact": "NOT_VERIFIED"
        }));
    }
    let layers = layers
        .iter()
        .map(|l| json!({"name": l.name, "order": l.order}))
        .collect::<Vec<_>>();
    Ok((
        format,
        json!({
            "schema": schema, "units": "integer-nm", "solver_ready": false,
            "layers": layers, "items": items,
            "physical_contact": "NOT_VERIFIED", "source_reopened": false
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nm_exactness_is_checked_not_rounded() {
        assert_eq!(exact_nm(Rational::new(127, 500_000_000).unwrap()), Ok(254));
        assert_eq!(
            exact_nm(Rational::new(1, 2_000_000_000).unwrap()),
            Err("BSI_F009_SUB_NM_UNREPRESENTABLE")
        );
        assert_eq!(size_nm(Rational::integer(0)), Err("BSI_F009_SIZE_LIMIT"));
    }
}
