//! D_013: opt-in legacy V1 barrel projection provenance; never physical electricity.
use boardsi_import::{contact_wire, route_preview};

pub fn encode_from_qualified_span(
    bytes: &[u8],
    span: &str,
    layers: [usize; 2],
) -> Result<String, &'static str> {
    const HEADER: &str = "BOARDSI_D012_VIA_SPAN_PROVENANCE_V1|";
    if span.len() > 64_000
        || !span.starts_with(HEADER)
        || !span.contains("\nD011_BEGIN\n")
        || !span.contains("\nD011_END\n")
        || !span.ends_with("END|NO_ELECTRICAL_BARREL_EDGE\n")
        || layers[0] == 0
        || layers[0] >= layers[1]
    {
        return Err("D013_D012_UNQUALIFIED");
    }
    let row = span.lines().next().ok_or("D013_D012_HEADER")?;
    let fields: Vec<_> = row.split('|').collect();
    if fields.len() != 6
        || fields[0] != "BOARDSI_D012_VIA_SPAN_PROVENANCE_V1"
        || fields[1].len() != 64
        || !fields[1].bytes().all(|b| b.is_ascii_hexdigit())
        || fields[2..]
            != [
                "SOURCE_SPAN_ONLY",
                "PLATING_IMPORTER_ASSUMED",
                "BARREL_NOT_VERIFIED",
                "NO_CROSS_LAYER_ELECTRICAL_EDGE",
            ]
    {
        return Err("D013_BAD_SOURCE_PROVENANCE");
    }
    // D_012, produced by the caller on this exact source, already requalified
    // D_011/D_010 Source geometry and SHA. Independently rebuild the legacy
    // Canonical endpoint projection rather than trusting a caller's count.
    let parsed = route_preview::parse(bytes)?;
    let board = route_preview::into_canonical(bytes, &parsed)?;
    if board.validate().has_errors()
        || parsed.vias.len() != 1
        || board.vias.len() != 1
        || parsed.segments.len() != 2
        || !parsed.arcs.is_empty()
    {
        return Err("D013_UNSUPPORTED_LEGACY_SOURCE");
    }
    let source_via = &parsed.vias[0];
    if source_via.first_layer as usize > layers[0]
        || (source_via.last_layer as usize) < layers[1]
    {
        return Err("D013_SOURCE_SPAN_MISMATCH");
    }
    let (local_contacts, legacy_barrels) = contact_wire::legacy_v1_counts(&board);
    // The two Source Trace endpoints must independently qualify exactly one
    // compatibility-only barrel in this narrowly selected one-Via fixture.
    if local_contacts < 2 || legacy_barrels != 1 {
        return Err("D013_LEGACY_PROJECTION_UNQUALIFIED");
    }
    let mut wire = format!(
        "BOARDSI_D013_LEGACY_BARREL_PROVENANCE_V1|{}|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\nD012_BEGIN\n",
        fields[1]
    );
    wire.push_str(span);
    wire.push_str("D012_END\n");
    wire.push_str(&format!(
        "LEGACY|0|VIA:0:L{}|VIA:0:L{}|V1_COMPATIBILITY_PROJECTION|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED\n",
        layers[0], layers[1]
    ));
    wire.push_str(
        "COUNT|LEGACY_COMPAT_BARRELS|1|SOURCE_QUALIFIED_LOCAL_CONTACTS_NOT_ELECTRICAL\nOVERALL|LEGACY_PROJECTION_NOT_ELECTRICAL\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n",
    );
    if wire.len() > 72_000 {
        return Err("D013_WIRE_LIMIT");
    }
    Ok(wire)
}
