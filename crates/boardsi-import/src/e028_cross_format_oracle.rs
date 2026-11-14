//! E_028: one bounded, opt-in cross-format SOURCE geometry oracle.
//! Only a single four-layer equal-circular THRU via from PADS 9.4 and KiCad 9.
//! Not a full-board equality proof. KiCad via *envelope* is not an electrical pad set.
use boardsi_model::{LayerKind, Orientation, PadRole, PadShape, QuarterTurn, Rational, ViaKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViaOracle {
    /// Independently authored fixture values in exact integer nanometres.
    pub copper_layers: u32,
    pub centre_nm: [i64; 2],
    pub diameter_nm: i64,
    pub drill_nm: i64,
    pub first_layer: u32,
    pub last_layer: u32,
    pub kind: ViaKind,
    /// Immutable raw source bytes, not Git blob SHA-1 or canonical IDs.
    pub pads_sha256: [u8; 32],
    pub kicad_sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViaQualification {
    pub pads_sha256: [u8; 32],
    pub kicad_sha256: [u8; 32],
    pub verified_via_geometry: bool,
    pub verified_end_pad_geometry_pads_only: bool,
    /// KiCad source declares a nominal via size, not per-layer pad contacts.
    pub kicad_per_layer_pad_contact_verified: bool,
    pub electrical_connectivity_verified: bool,
    pub solver_ready: bool,
}

/// Three-way comparison: PADS canonical model, KiCad independent source
/// inventory, and caller-pinned independent raw-byte/physical-geometry oracle.
/// Any malformed, missing, contradicting or out-of-scope evidence is rejected.
pub fn qualify_pads_kicad_single_through_via(
    pads: &[u8],
    kicad: &[u8],
    expected: &ViaOracle,
) -> Result<ViaQualification, &'static str> {
    if expected.copper_layers != 4
        || expected.first_layer != 1
        || expected.last_layer != 4
        || expected.kind != ViaKind::Through
        || expected.drill_nm <= 0
        || expected.diameter_nm <= expected.drill_nm
    {
        return Err("E028_UNSUPPORTED_ORACLE_SCOPE");
    }
    let pads_sha = crate::digest::sha256(pads);
    let kicad_sha = crate::digest::sha256(kicad);
    if pads_sha != expected.pads_sha256 || kicad_sha != expected.kicad_sha256 {
        return Err("E028_RAW_SOURCE_SHA_MISMATCH");
    }

    let artwork = crate::route_preview::parse_generalized(pads)
        .map_err(|_| "E028_PADS_SOURCE_REJECTED")?;
    if artwork.max_layer != expected.copper_layers
        || artwork.vias.len() != 1
        || artwork.skipped != 0
        || !artwork.arcs.is_empty()
    {
        return Err("E028_PADS_SCOPE_MISMATCH");
    }
    let model = crate::route_preview::into_canonical(pads, &artwork)
        .map_err(|_| "E028_PADS_CANONICAL_REJECTED")?;
    if model.vias.len() != 1 || model.stackup.layers.len() != 4
        || model.sources.len() != 1 || model.frames.len() != 1
        || model.sources[0].id.files.len() != 1
        || model.sources[0].id.files[0].sha256 != pads_sha
    {
        return Err("E028_PADS_CANONICAL_EVIDENCE_MISSING");
    }
    let frame = &model.frames[0];
    if frame.metres_per_tick != Rational::new(127, 500_000_000).map_err(|_| "E028_FRAME_INVALID")?
        || frame.origin_m != [Rational::integer(0); 2]
        || frame.orientation != Orientation::Orthogonal {
            turn: QuarterTurn::Zero, mirror_x: false
        }
    {
        return Err("E028_UNSUPPORTED_PADS_FRAME");
    }
    for (index, layer) in model.stackup.layers.iter().enumerate() {
        if layer.order != index as u32 || layer.kind != LayerKind::Copper {
            return Err("E028_PADS_LAYER_MISMATCH");
        }
    }
    let p = &artwork.vias[0];
    if p.shaped.is_some() || p.glued
        || p.first_layer != expected.first_layer
        || p.last_layer != expected.last_layer
        || p.point[0].checked_mul(254) != Some(expected.centre_nm[0])
        || p.point[1].checked_mul(254) != Some(expected.centre_nm[1])
        || p.drill_ticks.checked_mul(254) != Some(expected.drill_nm)
    {
        return Err("E028_PADS_VIA_GEOMETRY_MISMATCH");
    }
    let topology = p.topology.as_ref().ok_or("E028_PADS_TOPOLOGY_UNKNOWN")?;
    if topology.kind != expected.kind || topology.pads.len() != 2
        || topology.first_layer != expected.first_layer
        || topology.last_layer != expected.last_layer
        || topology.diameter_on(1).and_then(|v| v.checked_mul(254)) != Some(expected.diameter_nm)
        || topology.diameter_on(4).and_then(|v| v.checked_mul(254)) != Some(expected.diameter_nm)
        || topology.diameter_on(2).is_some() || topology.diameter_on(3).is_some()
    {
        return Err("E028_PADS_LAYER_PAD_MISMATCH");
    }
    let canonical_via = &model.vias[0];
    if canonical_via.kind != expected.kind
        || canonical_via.position.frame != frame.meta.id
        || canonical_via.position.ticks != p.point
        || canonical_via.span.first_layer != model.stackup.layers[0].meta.id
        || canonical_via.span.last_layer != model.stackup.layers[3].meta.id
    {
        return Err("E028_PADS_CANONICAL_VIA_MISMATCH");
    }
    let padstack = model.padstacks.iter().find(|v| v.meta.id == canonical_via.padstack)
        .ok_or("E028_PADS_PADSTACK_MISSING")?;
    let drill = model.drills.iter().find(|v| v.meta.id == canonical_via.drill)
        .ok_or("E028_PADS_DRILL_MISSING")?;
    let expected_diameter_m = Rational::new(i128::from(expected.diameter_nm), 1_000_000_000)
        .map_err(|_| "E028_ORACLE_DIMENSION_INVALID")?;
    let expected_drill_m = Rational::new(i128::from(expected.drill_nm), 1_000_000_000)
        .map_err(|_| "E028_ORACLE_DIMENSION_INVALID")?;
    if padstack.entries.len() != 2
        || drill.diameter_m != expected_drill_m
        || drill.span != canonical_via.span
    {
        return Err("E028_CANONICAL_PAD_DRILL_MISMATCH");
    }
    for (entry, index) in padstack.entries.iter().zip([0usize, 3usize]) {
        if entry.layer != model.stackup.layers[index].meta.id
            || entry.role != PadRole::Conductive
            || !matches!(&entry.shape, PadShape::Circle { diameter_m } if *diameter_m == expected_diameter_m)
        {
            return Err("E028_CANONICAL_PAD_DRILL_MISMATCH");
        }
    }

    let inventory = crate::kicad9_source::inspect(kicad)
        .map_err(|_| "E028_KICAD_SOURCE_REJECTED")?;
    let copper: Vec<_> = inventory.layers.iter().filter(|l| l.kind == LayerKind::Copper).collect();
    if inventory.identity.files.len() != 1
        || inventory.identity.files[0].sha256 != kicad_sha
        || inventory.vias.len() != 1
        || copper.len() != expected.copper_layers as usize
        || inventory.solver_ready
    {
        return Err("E028_KICAD_SCOPE_MISMATCH");
    }
    let mut orders: Vec<_> = copper.iter().filter_map(|l| l.copper_order).collect();
    orders.sort_unstable();
    if orders != vec![0, 1, 2, 3] {
        return Err("E028_KICAD_LAYER_MAPPING_MISMATCH");
    }
    let kv = &inventory.vias[0];
    if kv.kind != expected.kind || kv.centre_nm != expected.centre_nm
        || kv.diameter_nm != expected.diameter_nm || kv.drill_nm != expected.drill_nm
        || kv.source_first_layer != "F.Cu" || kv.source_last_layer != "B.Cu"
        || kv.span.first_layer != copper.iter().find(|v| v.copper_order == Some(0))
            .ok_or("E028_KICAD_LAYER_MAPPING_MISMATCH")?.meta.id
        || kv.span.last_layer != copper.iter().find(|v| v.copper_order == Some(3))
            .ok_or("E028_KICAD_LAYER_MAPPING_MISMATCH")?.meta.id
        || kv.envelope_layers.len() != 4
        || kv.remove_unused_layers || kv.keep_end_layers
    {
        return Err("E028_KICAD_VIA_GEOMETRY_MISMATCH");
    }
    for (i, id) in kv.envelope_layers.iter().enumerate() {
        if *id != copper.iter().find(|l| l.copper_order == Some(i))
            .ok_or("E028_KICAD_LAYER_MAPPING_MISMATCH")?.meta.id {
            return Err("E028_KICAD_LAYER_MAPPING_MISMATCH");
        }
    }

    Ok(ViaQualification {
        pads_sha256: pads_sha,
        kicad_sha256: kicad_sha,
        verified_via_geometry: true,
        verified_end_pad_geometry_pads_only: true,
        kicad_per_layer_pad_contact_verified: false,
        electrical_connectivity_verified: false,
        solver_ready: false,
    })
}
