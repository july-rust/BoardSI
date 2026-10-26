//! B_001_2: strict JSON adapter to the real Rust boardsi-model crate.
//! This validates *assumed* layer/material values; no synthetic PCB net/area is invented.
use boardsi_model::*;
use serde_json::{json, Value};
use std::io::{self, Read};
use std::sync::Arc;

fn text_field<'a>(row: &'a [Value], i: usize) -> Result<&'a str, &'static str> {
    row.get(i).and_then(Value::as_str).ok_or("ROW_FIELD_NOT_STRING")
}
fn float_field(row: &[Value], i: usize) -> Result<f64, &'static str> {
    let v: f64 = text_field(row, i)?.parse().map_err(|_| "INVALID_NUMERIC_FIELD")?;
    if !v.is_finite() { return Err("INVALID_NUMERIC_FIELD"); }
    Ok(v)
}
fn optional_float(row: &[Value], i: usize) -> Result<Option<f64>, &'static str> {
    if text_field(row, i)?.is_empty() { Ok(None) } else { float_field(row, i).map(Some) }
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
    if s.len() != 64 || !s.is_ascii() { return Err("INVALID_SOURCE_DIGEST"); }
    let mut bytes = [0u8;32];
    for (i, target) in bytes.iter_mut().enumerate() {
        *target = u8::from_str_radix(&s[i*2..i*2+2], 16)
            .map_err(|_| "INVALID_SOURCE_DIGEST")?;
    }
    Ok(bytes)
}
fn audit(input: &Value) -> Result<Value, &'static str> {
    let object = input.as_object().ok_or("INVALID_DOCUMENT")?;
    if object.get("schema").and_then(Value::as_str) != Some("boardsi-b001-stackup-overlay-v2")
        || object.get("scope").and_then(Value::as_str) != Some("USER_OVERLAY_NOT_PHYSICAL_SI")
    { return Err("INVALID_SCHEMA"); }
    let digest = sha_bytes(object.get("source_sha256").and_then(Value::as_str)
        .ok_or("MISSING_SOURCE_DIGEST")?)?;
    let frequency = object.get("audit_frequency_hz").and_then(Value::as_str)
        .ok_or("MISSING_AUDIT_FREQUENCY")?
        .parse::<f64>().map_err(|_| "INVALID_AUDIT_FREQUENCY")?;
    if !frequency.is_finite() || frequency < 0.0 {return Err("INVALID_AUDIT_FREQUENCY");}
    let layers = object.get("layers").and_then(Value::as_array).ok_or("MISSING_LAYERS")?;
    if layers.is_empty() || layers.len() > 64 { return Err("INVALID_LAYER_COUNT"); }
    let mut edits = Vec::with_capacity(layers.len());
    for (index, item) in layers.iter().enumerate() {
        let row = item.as_array().ok_or("INVALID_LAYER_ROW")?;
        if row.len() != 16 || row.iter().any(|v| {
            v.as_str().map_or(true, |x| x.len() > 512)
        }) {
            return Err("INVALID_LAYER_ROW");
        }
        let index_str = text_field(row, 0)?;
        if index_str.parse::<usize>().map_err(|_| "INVALID_LAYER_ID")? != index + 1 {
            return Err("MISMATCH_LAYER_ID");
        }
        let thickness_um = text_field(row, 2)?;
        // Strict exact integer micrometres; reject truncation rather than silently round.
        let thickness = thickness_um.parse::<i64>().map_err(|_| "INVALID_THICKNESS_UM")?;
        let reference_layer_id = if text_field(row, 8)?.is_empty() {
            None
        } else {
            Some(text_field(row, 8)?.parse::<u32>().map_err(|_| "INVALID_REFERENCE_ID")?)
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
    let source: SourceId = Arc::new(SourceIdentity {
        format: SourceFormat::PadsAsc,
        format_version: "V9.4-MILS-partial".into(),
        files: vec![SourceFile { logical_path: "imported-source.asc".into(), sha256: digest }],
        importer_version: "b001-ui-overlay (no copper metadata)".into(),
        conditions: vec![("overlay".into(), "user-assumed".into())],
    });
    let result = build_stackup_configuration(source, &edits, frequency)
        .map_err(|_| "CANONICAL_MATERIAL_OR_STACKUP_REJECTED")?;
    let total = result.total_thickness_m.to_f64()
        .map_err(|_| "INVALID_TOTAL_THICKNESS")?;
    let report = result.model.validate_materials_at(frequency);
    if report.has_errors() || report.has_missing() { return Err("CANONICAL_VALIDATION_REJECTED"); }
    Ok(json!({
        "schema": "boardsi-b001-canonical-audit-v1",
        "canonical_stackup": "ASSUMED",
        "canonical_materials": "ASSUMED_VALID_IN_BAND",
        "reference_continuity": "UNSUPPORTED_NO_COPPER_NET_EVIDENCE",
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
    if std::env::args().collect::<Vec<_>>().as_slice() != ["boardsi-stackup-config", "--audit-stdin-v1"] {
        // Only the documented non-interactive CLI contract is supported.
        if std::env::args().len() != 2 || std::env::args().nth(1).as_deref() != Some("--audit-stdin-v1") {
            eprintln!("Usage: boardsi-stackup-config --audit-stdin-v1");
            std::process::exit(2);
        }
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
            println!("{}", json!({
                "schema": "boardsi-b001-canonical-audit-v1",
                "canonical_stackup": "REJECTED",
                "reason": error,
                "solver_ready": false,
                "physical_si": "NOT_RUN"
            }));
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_source_is_not_canonical() {
        assert!(sha_bytes("not-sha").is_err());
        let obj = json!({"schema":"boardsi-b001-stackup-overlay-v2","scope":"USER_OVERLAY_NOT_PHYSICAL_SI",
            "source_sha256":"0000000000000000000000000000000000000000000000000000000000000000",
            "audit_frequency_hz":"1000000000","layers":[["1","Unknown","35","","","","","Unknown","","","","","","","",""]]});
        assert_eq!(audit(&obj).unwrap_err(), "UNSUPPORTED_LAYER_KIND");
    }
}
