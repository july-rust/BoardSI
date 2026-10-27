//! B_001_2: strict JSON adapter to the real Rust boardsi-model crate.
//! This validates *assumed* layer/material values; no synthetic PCB net/area is invented.
use boardsi_model::*;
use serde_json::{json, Value};
use std::io::{self, Read};
use std::sync::Arc;

fn text_field(row: &[Value], i: usize) -> Result<&str, &'static str> {
    row.get(i)
        .and_then(Value::as_str)
        .ok_or("ROW_FIELD_NOT_STRING")
}
fn float_field(row: &[Value], i: usize) -> Result<f64, &'static str> {
    let v: f64 = text_field(row, i)?
        .parse()
        .map_err(|_| "INVALID_NUMERIC_FIELD")?;
    if !v.is_finite() {
        return Err("INVALID_NUMERIC_FIELD");
    }
    Ok(v)
}
fn optional_float(row: &[Value], i: usize) -> Result<Option<f64>, &'static str> {
    if text_field(row, i)?.is_empty() {
        Ok(None)
    } else {
        float_field(row, i).map(Some)
    }
}
fn layer_kind(value: &str) -> Result<LayerKind, &'static str> {
    match value {
        "Copper" => Ok(LayerKind::Copper),
        "Core" => Ok(LayerKind::Dielectric(DielectricKind::Core)),
        "Prepreg" => Ok(LayerKind::Dielectric(DielectricKind::Prepreg)),
        "Mask" => Ok(LayerKind::Mask),
        _ => Err("UNSUPPORTED_LAYER_KIND"),
    }
}
fn role_hint(value: &str) -> Result<NetRole, &'static str> {
    match value {
        "Signal" => Ok(NetRole::Signal),
        "Ground" => Ok(NetRole::Ground),
        "Power" => Ok(NetRole::Power),
        "Floating" => Ok(NetRole::Floating),
        "Unknown" => Ok(NetRole::Unknown),
        _ => Err("INVALID_ROLE_HINT"),
    }
}
fn sha_bytes(s: &str) -> Result<[u8; 32], &'static str> {
    if s.len() != 64 || !s.is_ascii() {
        return Err("INVALID_SOURCE_DIGEST");
    }
    let mut bytes = [0u8; 32];
    for (i, target) in bytes.iter_mut().enumerate() {
        *target =
            u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| "INVALID_SOURCE_DIGEST")?;
    }
    Ok(bytes)
}
/// This is source-owned *object inventory*, not conductor connectivity.
 /// It can invalidate an unsafe Reference candidate but can never certify P009.
fn reference_inventory_audit(
    obj: &serde_json::Map<String, Value>,
    edits: &[StackupLayerEdit],
) -> Result<(&'static str, usize, usize), &'static str> {
    use std::collections::BTreeSet;
    let nets = obj
        .get("source_net_labels")
        .and_then(Value::as_array)
        .ok_or("MISSING_SOURCE_NET_INVENTORY")?;
    let copper = obj
        .get("reference_copper_inventory")
        .and_then(Value::as_array)
        .ok_or("MISSING_SOURCE_COPPER_INVENTORY")?;
    if nets.len() > 128 || copper.len() > 64 {
        return Err("OVERSIZED_SOURCE_REFERENCE_INVENTORY");
    }
    let complete = obj
        .get("net_inventory_complete")
        .and_then(Value::as_bool)
        .ok_or("MISSING_NET_INVENTORY_COMPLETENESS")?;
    let mut observed = BTreeSet::new();
    for name in nets {
        let n = name.as_str().ok_or("INVALID_SOURCE_NET_LABEL")?;
        if n.is_empty() || n.len() > 64 || !observed.insert(n) {
            return Err("INVALID_SOURCE_NET_LABEL");
        }
    }
    let mut areas = Vec::with_capacity(copper.len());
    let mut seen_names = BTreeSet::new();
    for entry in copper {
        let row = entry.as_array().ok_or("INVALID_SOURCE_COPPER_RECORD")?;
        if row.len() != 7 {
            return Err("INVALID_SOURCE_COPPER_RECORD");
        }
        let layer = row[0].as_u64().ok_or("INVALID_COPPER_LAYER")?;
        let name = row[1].as_str().ok_or("INVALID_COPPER_NAME")?;
        let net = row[2].as_str().ok_or("INVALID_COPPER_NET")?;
        let line = row[3].as_u64().ok_or("INVALID_COPPER_SOURCE_LINE")?;
        let holes = row[4].as_u64().ok_or("INVALID_COPPER_CUTOUTS")?;
        let curved = row[5].as_bool().ok_or("INVALID_COPPER_CURVE")?;
        let circle = row[6].as_bool().ok_or("INVALID_COPPER_CIRCLE")?;
        if layer == 0 || layer as usize > edits.len()
            || edits[(layer - 1) as usize].kind != LayerKind::Copper
            || name.is_empty() || name.len() > 16
            || net.is_empty() || net.len() > 47
            || line == 0 || holes > 8 || (curved && circle)
            || !observed.contains(net) || !seen_names.insert(name)
        {
            return Err("SOURCE_COPPER_INVENTORY_CONFLICT");
        }
        areas.push((layer as u32, net, holes));
    }
    let sig = obj
        .get("selected_signal_net")
        .and_then(Value::as_str)
        .ok_or("MISSING_SELECTED_SIGNAL_NET")?;
    let reference = obj
        .get("selected_reference_net")
        .and_then(Value::as_str)
        .ok_or("MISSING_SELECTED_REFERENCE_NET")?;
    if sig.len() > 64 || reference.len() > 64 {
        return Err("OVERSIZED_SELECTED_NET");
    }
    if sig.is_empty() || reference.is_empty() {
        return Ok(("NOT_SELECTED", nets.len(), copper.len()));
    }
    if sig == reference {
        return Err("SIGNAL_REFERENCE_SAME_NET");
    }
    if !complete || areas.is_empty() {
        return Ok(("NO_SOURCE_REFERENCE_EVIDENCE", nets.len(), copper.len()));
    }
    if !observed.contains(sig) || !observed.contains(reference) {
        return Ok(("UNKNOWN_OR_MISSING_SOURCE_NET", nets.len(), copper.len()));
    }
    let possible_layers: BTreeSet<u32> = edits
        .iter()
        .filter_map(|e| e.reference_layer_id)
        .collect();
    if possible_layers.is_empty() {
        return Ok(("UNKNOWN_OR_MISSING_SOURCE_NET", nets.len(), copper.len()));
    }
    let matching: Vec<_> = areas
        .iter()
        .filter(|(layer, net, _)| *net == reference && possible_layers.contains(layer))
        .collect();
    if matching.is_empty() {
        return Ok(("UNKNOWN_OR_MISSING_SOURCE_NET", nets.len(), copper.len()));
    }
    // Multiple disjoint source Area IDs, curves and holes do NOT prove a
    // continuous return conductor. Never bridge them or short other Nets.
    if matching.len() > 1 || matching.iter().any(|(_, _, holes)| *holes > 0) {
        return Ok(("CUTOUT_OR_SPLIT_UNSUPPORTED", nets.len(), copper.len()));
    }
    Ok(("OBSERVED_LABELS_NO_TOPOLOGY_PROOF", nets.len(), copper.len()))
}

fn audit(input: &Value) -> Result<Value, &'static str> {
    let object = input.as_object().ok_or("INVALID_DOCUMENT")?;
    if object.get("schema").and_then(Value::as_str) != Some("boardsi-b001-stackup-overlay-v2")
        || object.get("scope").and_then(Value::as_str) != Some("USER_OVERLAY_NOT_PHYSICAL_SI")
    {
        return Err("INVALID_SCHEMA");
    }
    let digest = sha_bytes(
        object
            .get("source_sha256")
            .and_then(Value::as_str)
            .ok_or("MISSING_SOURCE_DIGEST")?,
    )?;
    let frequency = object
        .get("audit_frequency_hz")
        .and_then(Value::as_str)
        .ok_or("MISSING_AUDIT_FREQUENCY")?
        .parse::<f64>()
        .map_err(|_| "INVALID_AUDIT_FREQUENCY")?;
    if !frequency.is_finite() || frequency < 0.0 {
        return Err("INVALID_AUDIT_FREQUENCY");
    }
    let layers = object
        .get("layers")
        .and_then(Value::as_array)
        .ok_or("MISSING_LAYERS")?;
    if layers.is_empty() || layers.len() > 64 {
        return Err("INVALID_LAYER_COUNT");
    }
    let mut edits = Vec::with_capacity(layers.len());
    for (index, item) in layers.iter().enumerate() {
        let row = item.as_array().ok_or("INVALID_LAYER_ROW")?;
        if row.len() != 16
            || row
                .iter()
                .any(|v| v.as_str().map_or(true, |x| x.len() > 512))
        {
            return Err("INVALID_LAYER_ROW");
        }
        let index_str = text_field(row, 0)?;
        if index_str.parse::<usize>().map_err(|_| "INVALID_LAYER_ID")? != index + 1 {
            return Err("MISMATCH_LAYER_ID");
        }
        let thickness_um = text_field(row, 2)?;
        // Strict exact integer micrometres; reject truncation rather than silently round.
        let thickness = thickness_um
            .parse::<i64>()
            .map_err(|_| "INVALID_THICKNESS_UM")?;
        let reference_layer_id = if text_field(row, 8)?.is_empty() {
            None
        } else {
            Some(
                text_field(row, 8)?
                    .parse::<u32>()
                    .map_err(|_| "INVALID_REFERENCE_ID")?,
            )
        };
        let band = match (optional_float(row, 14)?, optional_float(row, 15)?) {
            (Some(min_hz), Some(max_hz)) => Some(FrequencyRange { min_hz, max_hz }),
            (None, None) => None,
            _ => return Err("INCOMPLETE_MATERIAL_BAND"),
        };
        edits.push(StackupLayerEdit {
            source_layer_id: (index + 1) as u32,
            kind: layer_kind(text_field(row, 1)?)?,
            thickness_um: thickness,
            material_name: text_field(row, 3)?.into(),
            dk: optional_float(row, 4)?,
            df: optional_float(row, 5)?,
            conductivity_s_per_m: optional_float(row, 6)?,
            band_hz: band,
            evidence: PropertySource {
                document: text_field(row, 9)?.into(),
                revision: text_field(row, 10)?.into(),
                method: text_field(row, 11)?.into(),
                reference_frequency_hz: optional_float(row, 12)?,
                temperature_c: optional_float(row, 13)?,
            },
            role_hint: role_hint(text_field(row, 7)?)?,
            reference_layer_id,
        });
    }
    let (reference_candidate, observed_net_count, copper_area_count) =
        reference_inventory_audit(object, &edits)?;
    let source: SourceId = Arc::new(SourceIdentity {
        format: SourceFormat::PadsAsc,
        format_version: "V9.4-MILS-partial".into(),
        files: vec![SourceFile {
            logical_path: "imported-source.asc".into(),
            sha256: digest,
        }],
        importer_version: "b001-ui-overlay (no copper metadata)".into(),
        conditions: vec![("overlay".into(), "user-assumed".into())],
    });
    let result = build_stackup_configuration(source, &edits, frequency)
        .map_err(|_| "CANONICAL_MATERIAL_OR_STACKUP_REJECTED")?;
    let total = result
        .total_thickness_m
        .to_f64()
        .map_err(|_| "INVALID_TOTAL_THICKNESS")?;
    let report = result.model.validate_materials_at(frequency);
    if material_overlay_report_has_blocker(&report) {
        return Err("CANONICAL_VALIDATION_REJECTED");
    }
    Ok(json!({
        "schema": "boardsi-b001-canonical-audit-v1",
        "canonical_stackup": "ASSUMED",
        "canonical_materials": "ASSUMED_VALID_IN_BAND",
        "reference_continuity": "UNSUPPORTED_NO_COPPER_NET_EVIDENCE",
        "reference_candidate": reference_candidate,
        "observed_net_count": observed_net_count,
        "observed_copper_area_count": copper_area_count,
        "electrical_net_separation": "NOT_VERIFIED",
        "return_path_p009": "UNSUPPORTED",
        "physical_si": "NOT_RUN",
        "solver_ready": false,
        "source_sha256": object.get("source_sha256"),
        "layer_count": result.model.stackup.layers.len(),
        "material_count": result.model.materials.len(),
        "board_total_thickness_m": total,
        "analysis_frequency_hz": frequency
    }))
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 || args[1] != "--audit-stdin-v1" {
        eprintln!("Usage: boardsi-stackup-config --audit-stdin-v1");
        std::process::exit(2);
    }
    let mut input = String::new();
    let output = match io::stdin().take(65_537).read_to_string(&mut input) {
        Ok(n) if n <= 65_536 => match serde_json::from_str::<Value>(&input) {
            Ok(doc) => audit(&doc),
            Err(_) => Err("INVALID_JSON"),
        },
        _ => Err("INPUT_TOO_LARGE_OR_UNREADABLE"),
    };
    match output {
        Ok(receipt) => println!("{}", receipt),
        Err(error) => {
            println!(
                "{}",
                json!({
                    "schema": "boardsi-b001-canonical-audit-v1",
                    "canonical_stackup": "REJECTED",
                    "reason": error,
                    "solver_ready": false,
                    "physical_si": "NOT_RUN"
                })
            );
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_source_net_labels_never_certify_a_return_path() {
        let source = PropertySource {
            document: "PUBLIC_TEST_ONLY".into(), revision: "1".into(),
            method: "test".into(), reference_frequency_hz: Some(1e9),
            temperature_c: Some(25.0),
        };
        let edit = StackupLayerEdit {
            source_layer_id: 1, kind: LayerKind::Copper, thickness_um: 35,
            material_name: "public".into(), dk: None, df: None,
            conductivity_s_per_m: Some(5.8e7), band_hz: Some(FrequencyRange {
                min_hz: 1e6, max_hz: 2e9,
            }), evidence: source.clone(), role_hint: NetRole::Signal,
            reference_layer_id: Some(3),
        };
        let core = StackupLayerEdit {
            source_layer_id: 2, kind: LayerKind::Dielectric(DielectricKind::Core),
            thickness_um: 100, material_name: "public core".into(),
            dk: Some(4.2), df: Some(0.02), conductivity_s_per_m: None,
            band_hz: edit.band_hz, evidence: source.clone(),
            role_hint: NetRole::Unknown, reference_layer_id: None,
        };
        let gnd = StackupLayerEdit {
            source_layer_id: 3, reference_layer_id: None,
            role_hint: NetRole::Ground, ..edit.clone()
        };
        let layers = [edit, core, gnd];
        let mut evidence = json!({
            "selected_signal_net":"SIG", "selected_reference_net":"GND",
            "source_net_labels":["SIG","GND"],
            "reference_copper_inventory":[[3,"PLANE","GND",120,0,false,false]],
            "net_inventory_complete":true
        });
        let obj = evidence.as_object().unwrap();
        assert_eq!(reference_inventory_audit(obj, &layers).unwrap().0,
                   "OBSERVED_LABELS_NO_TOPOLOGY_PROOF");
        evidence["reference_copper_inventory"][0][4] = json!(1);
        assert_eq!(reference_inventory_audit(evidence.as_object().unwrap(), &layers).unwrap().0,
                   "CUTOUT_OR_SPLIT_UNSUPPORTED");
        evidence["reference_copper_inventory"][0][4] = json!(0);
        evidence["reference_copper_inventory"]
            .as_array_mut().unwrap().push(json!([3,"ISLAND","GND",130,0,false,false]));
        assert_eq!(reference_inventory_audit(evidence.as_object().unwrap(), &layers).unwrap().0,
                   "CUTOUT_OR_SPLIT_UNSUPPORTED");
        evidence["reference_copper_inventory"].as_array_mut().unwrap().pop();
        evidence["selected_reference_net"] = json!("SIG");
        assert_eq!(reference_inventory_audit(evidence.as_object().unwrap(), &layers),
                   Err("SIGNAL_REFERENCE_SAME_NET"));
        evidence["selected_reference_net"] = json!("UNKNOWN");
        assert_eq!(reference_inventory_audit(evidence.as_object().unwrap(), &layers).unwrap().0,
                   "UNKNOWN_OR_MISSING_SOURCE_NET");
        evidence["selected_reference_net"] = json!("GND");
        evidence["net_inventory_complete"] = json!(false);
        assert_eq!(reference_inventory_audit(evidence.as_object().unwrap(), &layers).unwrap().0,
                   "NO_SOURCE_REFERENCE_EVIDENCE");
        evidence["net_inventory_complete"] = json!(true);
        evidence["reference_copper_inventory"][0][0] = json!(2);
        assert_eq!(reference_inventory_audit(evidence.as_object().unwrap(), &layers),
                   Err("SOURCE_COPPER_INVENTORY_CONFLICT"));
        evidence["reference_copper_inventory"][0][0] = json!(3);
        evidence["reference_copper_inventory"][0][2] = json!("OTHER_NET");
        assert_eq!(reference_inventory_audit(evidence.as_object().unwrap(), &layers),
                   Err("SOURCE_COPPER_INVENTORY_CONFLICT"));
    }

    #[test]
    fn invalid_source_is_not_canonical() {
        assert!(sha_bytes("not-sha").is_err());
        let obj = json!({"schema":"boardsi-b001-stackup-overlay-v2","scope":"USER_OVERLAY_NOT_PHYSICAL_SI",
            "source_sha256":"0000000000000000000000000000000000000000000000000000000000000000",
            "audit_frequency_hz":"1000000000","selected_signal_net":"",
            "selected_reference_net":"","net_inventory_complete":false,
            "source_net_labels":[],"reference_copper_inventory":[],
            "layers":[["1","Unknown","35","","","","","Unknown","","","","","","","",""]]});
        assert_eq!(audit(&obj).unwrap_err(), "UNSUPPORTED_LAYER_KIND");
    }
}
