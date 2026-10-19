//! Versioned, content-bound M0 SYNTHETIC subset transport, not physical SI.
//! v1 IPC remains unchanged; this is an isolated opt-in proof of a v2 boundary.
//! SourceFile sha256 fields are fixture/importer claims, not independently authenticated CAD bytes.
use boardsi_model::*;
use serde_json::{json, Value};
use std::io::{self, Read, Write};

pub const MAX_V2_FRAME: usize = 65_536;
const MODEL_DOMAIN: &[u8] = b"BoardSI M0 synthetic model v2\0";
const SNAPSHOT_DOMAIN: &[u8] = b"BoardSI M0 synthetic run snapshot v2\0";

fn bad(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
fn hex(bytes: &[u8]) -> String {
    const NIBBLES: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(NIBBLES[(byte >> 4) as usize] as char);
        text.push(NIBBLES[(byte & 15) as usize] as char);
    }
    text
}
// SHA-256 FIPS 180-4. No security/authentication claim: this binds the transported bytes.
fn sha256_hex(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut padded = bytes.to_vec();
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(block[i * 4..i * 4 + 4].try_into().expect("word"));
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut q] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let t1 = q
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(majority);
            q = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (v, delta) in h.iter_mut().zip([a, b, c, d, e, f, g, q]) {
            *v = v.wrapping_add(delta);
        }
    }
    let mut digest_bytes = [0u8; 32];
    for (chunk, word) in digest_bytes.chunks_exact_mut(4).zip(h.iter()) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    hex(&digest_bytes)
}
fn digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut v = Vec::with_capacity(domain.len() + bytes.len());
    v.extend_from_slice(domain);
    v.extend_from_slice(bytes);
    sha256_hex(&v)
}
fn number(v: f64) -> io::Result<f64> {
    if !v.is_finite() {
        return Err(input("NonFiniteProperty"));
    }
    Ok(if v == 0.0 { 0.0 } else { v })
}
fn optional_number(v: Option<f64>) -> io::Result<Option<f64>> {
    v.map(number).transpose()
}
fn rational(r: Rational) -> Value {
    json!([r.numerator().to_string(), r.denominator().to_string()])
}
fn property_source(s: &PropertySource) -> io::Result<Value> {
    Ok(
        json!({"document":s.document,"revision":s.revision,"method":s.method,
        "reference_frequency_hz":optional_number(s.reference_frequency_hz)?,
        "temperature_c":optional_number(s.temperature_c)?}),
    )
}
fn scalar(p: &Property<f64>) -> io::Result<Value> {
    match p {
        Property::Confirmed { value, source } => {
            Ok(json!({"value":number(*value)?,"source":property_source(source)?}))
        }
        _ => Err(input("UnconfirmedMaterial")),
    }
}
fn interval(p: &Property<FrequencyRange>) -> io::Result<Value> {
    match p {
        Property::Confirmed { value, source } => Ok(json!({"min_hz":number(value.min_hz)?,
            "max_hz":number(value.max_hz)?,"source":property_source(source)?})),
        _ => Err(input("UnconfirmedBand")),
    }
}
fn rational_property(p: &Property<Rational>) -> io::Result<Value> {
    match p {
        Property::Confirmed { value, source } => {
            Ok(json!({"value":rational(*value),"source":property_source(source)?}))
        }
        _ => Err(input("UnconfirmedLayer")),
    }
}
fn meta(m: &ObjectMeta, source: &SourceId, kind: ObjectKind) -> io::Result<Value> {
    if m.id.source.as_ref() != source.as_ref()
        || m.id.kind != kind
        || !m.provenance.history.is_empty()
    {
        return Err(input("UnsupportedMetaOrConversion"));
    }
    Ok(
        json!({"kind":format!("{kind:?}"),"object":m.id.source_object,"part":m.id.part,
        "provenance_type":m.provenance.source_object_type}),
    )
}
fn point(p: &Point, frame: &ObjectId) -> io::Result<Value> {
    if &p.frame != frame {
        return Err(input("PointFrameMismatch"));
    }
    Ok(json!([p.ticks[0], p.ticks[1]]))
}
fn synthetic_model_value(model: &BoardModel, points: u32) -> io::Result<Value> {
    if model.kind != BoardKind::Pcb
        || model.sources.len() != 1
        || model.frames.len() != 1
        || model.stackup.layers.len() != 1
        || model.materials.len() != 1
        || model.nets.len() != 1
        || model.outlines.len() != 1
        || !model.components.is_empty()
        || !model.pins.is_empty()
        || !model.traces.is_empty()
        || !model.arcs.is_empty()
        || !model.areas.is_empty()
        || !model.padstacks.is_empty()
        || !model.pads.is_empty()
        || !model.drills.is_empty()
        || !model.vias.is_empty()
        || !model.connectivity.nodes.is_empty()
        || !model.connectivity.edges.is_empty()
    {
        return Err(input("UnsupportedSyntheticSubset"));
    }
    let source = &model.sources[0];
    if source.id.format != SourceFormat::Synthetic {
        return Err(input("OnlySyntheticSourceSupported"));
    }
    let report = model.validate();
    if !report.issues.is_empty() {
        return Err(input("ModelNotFullyConfirmed"));
    }
    for frequency in [0.0, (f64::from(points) - 1.0) * 1e6] {
        if !model.validate_materials_at(frequency).issues.is_empty() {
            return Err(input("MaterialNotReady"));
        }
    }
    let s = &source.id;
    let frame = &model.frames[0];
    let (turn, mirror_x) = match frame.orientation {
        Orientation::Orthogonal { turn, mirror_x } => (turn, mirror_x),
        Orientation::PendingRotation { .. } => return Err(input("UnsupportedRotation")),
    };
    let layer = &model.stackup.layers[0];
    if layer.kind != LayerKind::Copper
        || layer.material.as_ref() != Some(&model.materials[0].meta.id)
    {
        return Err(input("UnsupportedLayer"));
    }
    let material = &model.materials[0];
    let net = &model.nets[0];
    let outline = &model.outlines[0];
    if !outline.cutouts.is_empty() {
        return Err(input("UnsupportedCutout"));
    }
    let mut segments = Vec::new();
    for seg in &outline.outer.segments {
        match seg {
            BoundarySegment::Line { start, end } => segments.push(json!({
                "start":point(start,&frame.meta.id)?,"end":point(end,&frame.meta.id)?})),
            _ => return Err(input("UnsupportedCurvedOutline")),
        }
    }
    let cap = &source.capability;
    Ok(json!({
        "scope":"M0_SYNTHETIC_SUBSET_NOT_PHYSICS",
        "kind":"Pcb",
        "source":{
            "format":"Synthetic","format_version":s.format_version,"importer_version":s.importer_version,
            "producer":source.producer,"producer_version":source.producer_version,
            "files":s.files.iter().map(|f|json!({"logical_path":f.logical_path,"claimed_sha256":hex(&f.sha256)})).collect::<Vec<_>>(),
            "conditions":s.conditions,"capability":{
                "geometry":format!("{:?}",cap.geometry),"net":format!("{:?}",cap.net),
                "pin_pad":format!("{:?}",cap.pin_pad),"stackup":format!("{:?}",cap.stackup),
                "reference":format!("{:?}",cap.reference),"model":format!("{:?}",cap.model),
                "object_coverage":cap.object_coverage
            }
        },
        "frame":{"meta":meta(&frame.meta,s,ObjectKind::Frame)?,"unit_label":frame.unit_label,
            "metres_per_tick":rational(frame.metres_per_tick),
            "origin_m":[rational(frame.origin_m[0]),rational(frame.origin_m[1])],
            "turn":format!("{turn:?}"),"mirror_x":mirror_x,
            "source_layer_order_reversed":frame.source_layer_order_reversed},
        "layer":{"meta":meta(&layer.meta,s,ObjectKind::Layer)?,"name":layer.name,
            "order":layer.order,"kind":"Copper",
            "z_m":rational_property(&layer.z_m)?,"thickness_m":rational_property(&layer.thickness_m)?,
            "material_object":material.meta.id.source_object},
        "material":{"meta":meta(&material.meta,s,ObjectKind::Material)?,"name":material.name,
            "dk":scalar(&material.dk)?,"df":scalar(&material.df)?,
            "conductivity_s_per_m":scalar(&material.conductivity_s_per_m)?,
            "valid_band":interval(&material.valid_band)?},
        "net":{"meta":meta(&net.meta,s,ObjectKind::Net)?,"name":net.name,
            "role":format!("{:?}",net.role)},
        "outline":{"meta":meta(&outline.meta,s,ObjectKind::Outline)?,"segments":segments}
    }))
}

#[derive(Debug)]
pub struct BoundSyntheticSnapshot {
    encoded: Vec<u8>,
    run_id: String,
    model_sha256: String,
    snapshot_id: String,
}
impl BoundSyntheticSnapshot {
    pub fn capture(
        model: &BoardModel,
        run_id: &str,
        points: u32,
        delay_ms: u32,
    ) -> io::Result<Self> {
        if run_id.is_empty()
            || run_id.len() > 128
            || !(1..=1000).contains(&points)
            || delay_ms > 1000
        {
            return Err(input("InvalidRunParameters"));
        }
        let wire_model = synthetic_model_value(model, points)?;
        // Capture and receiver MUST enforce the same strict synthetic schema.
        validate_wire_model(&wire_model).map_err(|_| input("UnsupportedWireModel"))?;
        let model_json = serde_json::to_string(&wire_model)
            .map_err(|_| input("CannotSerializeModel"))?;
        if model_json.len() > 32_768 {
            return Err(input("ModelTooLarge"));
        }
        let model_sha256 = digest(MODEL_DOMAIN, model_json.as_bytes());
        let snapshot_data =
            json!({"model_sha256":model_sha256,"points":points,"delay_ms":delay_ms});
        let snapshot_bytes =
            serde_json::to_vec(&snapshot_data).map_err(|_| input("SnapshotEncoding"))?;
        let snapshot_id = digest(SNAPSHOT_DOMAIN, &snapshot_bytes);
        let payload = json!({"schema":2,"kind":"SyntheticSnapshotV2","run_id":run_id,
            "model_json":model_json,"model_sha256":model_sha256,
            "snapshot_id":snapshot_id,"points":points,"delay_ms":delay_ms});
        let encoded = serde_json::to_vec(&payload).map_err(|_| input("CannotEncodeEnvelope"))?;
        if encoded.len() > MAX_V2_FRAME {
            return Err(input("FrameTooLarge"));
        }
        Ok(Self {
            encoded,
            run_id: run_id.to_owned(),
            model_sha256,
            snapshot_id,
        })
    }
    pub fn frame(&self) -> Vec<u8> {
        let mut b = (self.encoded.len() as u32).to_be_bytes().to_vec();
        b.extend_from_slice(&self.encoded);
        b
    }
    pub fn model_sha256(&self) -> &str {
        &self.model_sha256
    }
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }
    pub fn verify_receipt(&self, bytes: &[u8]) -> io::Result<()> {
        let v: Value = serde_json::from_slice(bytes).map_err(|_| bad("InvalidReceipt"))?;
        let m = v.as_object().ok_or_else(|| bad("InvalidReceiptObject"))?;
        if m.len() != 6
            || v["schema"] != 2
            || v["type"] != "SyntheticModelReceipt"
            || v["run_id"] != self.run_id
            || v["snapshot_id"] != self.snapshot_id
            || v["model_sha256"] != self.model_sha256
            || v["synthetic"] != true
        {
            return Err(bad("ReceiptIdentityMismatch"));
        }
        Ok(())
    }
}
pub fn read_frame(input: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut header = [0u8; 4];
    input.read_exact(&mut header)?;
    let len = u32::from_be_bytes(header) as usize;
    if len == 0 || len > MAX_V2_FRAME {
        return Err(bad("InvalidV2FrameSize"));
    }
    let mut b = vec![0u8; len];
    input.read_exact(&mut b)?;
    Ok(b)
}
pub fn write_frame(output: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    if payload.is_empty() || payload.len() > MAX_V2_FRAME {
        return Err(bad("InvalidV2FrameSize"));
    }
    output.write_all(&(payload.len() as u32).to_be_bytes())?;
    output.write_all(payload)?;
    output.flush()
}

fn object_keys<'a>(
    value: &'a Value,
    expected: &[&str],
) -> io::Result<&'a serde_json::Map<String, Value>> {
    let map = value.as_object().ok_or_else(|| bad("ObjectRequired"))?;
    if map.len() != expected.len() || expected.iter().any(|key| !map.contains_key(*key)) {
        return Err(bad("UnexpectedOrMissingNestedField"));
    }
    Ok(map)
}
fn string(v: &Value) -> io::Result<&str> {
    v.as_str().ok_or_else(|| bad("StringRequired"))
}
fn nonempty(v: &Value) -> io::Result<&str> {
    let s = string(v)?;
    if s.trim().is_empty() {
        return Err(bad("EmptyRequiredString"));
    }
    Ok(s)
}
fn array(v: &Value, min: usize, max: usize) -> io::Result<&[Value]> {
    let a = v.as_array().ok_or_else(|| bad("ArrayRequired"))?;
    if a.len() < min || a.len() > max {
        return Err(bad("InvalidArrayLength"));
    }
    Ok(a)
}
fn finite_number(v: &Value) -> io::Result<f64> {
    let f = v.as_f64().ok_or_else(|| bad("NumberRequired"))?;
    if !f.is_finite() {
        return Err(bad("NonFiniteNumber"));
    }
    Ok(f)
}
fn rational_wire(v: &Value, require_positive: bool) -> io::Result<()> {
    let pair = array(v, 2, 2)?;
    let nstr = nonempty(&pair[0])?;
    let dstr = nonempty(&pair[1])?;
    let n = nstr.parse::<i128>().map_err(|_| bad("RationalNumerator"))?;
    let d = dstr
        .parse::<i128>()
        .map_err(|_| bad("RationalDenominator"))?;
    if n.to_string() != nstr
        || d.to_string() != dstr
        || d <= 0
        || n == i128::MIN
        || (require_positive && n <= 0)
    {
        return Err(bad("NonCanonicalRational"));
    }
    Ok(())
}
fn point_wire(v: &Value) -> io::Result<[i64; 2]> {
    let pair = array(v, 2, 2)?;
    Ok([
        pair[0].as_i64().ok_or_else(|| bad("InvalidCoordinate"))?,
        pair[1].as_i64().ok_or_else(|| bad("InvalidCoordinate"))?,
    ])
}
fn meta_wire<'a>(v: &'a Value, kind: &str) -> io::Result<&'a str> {
    object_keys(v, &["kind", "object", "part", "provenance_type"])?;
    if string(&v["kind"])? != kind {
        return Err(bad("WrongObjectKind"));
    }
    let name = nonempty(&v["object"])?;
    string(&v["part"])?;
    nonempty(&v["provenance_type"])?;
    Ok(name)
}
fn provenance_wire(v: &Value) -> io::Result<()> {
    object_keys(
        v,
        &[
            "document",
            "revision",
            "method",
            "reference_frequency_hz",
            "temperature_c",
        ],
    )?;
    nonempty(&v["document"])?;
    nonempty(&v["revision"])?;
    nonempty(&v["method"])?;
    for key in ["reference_frequency_hz", "temperature_c"] {
        if !v[key].is_null() {
            let f = finite_number(&v[key])?;
            if key == "reference_frequency_hz" && f < 0.0 {
                return Err(bad("NegativeReferenceFrequency"));
            }
        }
    }
    Ok(())
}
fn property_wire(v: &Value, minimum: f64) -> io::Result<()> {
    object_keys(v, &["value", "source"])?;
    if finite_number(&v["value"])? < minimum {
        return Err(bad("InvalidMaterialValue"));
    }
    provenance_wire(&v["source"])
}
fn rational_property_wire(v: &Value, positive: bool) -> io::Result<()> {
    object_keys(v, &["value", "source"])?;
    rational_wire(&v["value"], positive)?;
    provenance_wire(&v["source"])
}
fn source_path_valid(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.split('/').any(|p| p == "." || p == ".." || p.is_empty())
}
fn digest_hex_wire(v: &Value) -> io::Result<()> {
    let s = string(v)?;
    if s.len() != 64
        || !s
            .bytes()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(bad("InvalidHexDigest"));
    }
    Ok(())
}
/// Independently check the *exact* supported nested wire grammar.
/// This is a structural/schema check, NOT an independent physical BoardModel validator.
fn validate_wire_model(model: &Value) -> io::Result<()> {
    object_keys(
        model,
        &[
            "scope", "kind", "source", "frame", "layer", "material", "net", "outline",
        ],
    )?;
    if model["scope"] != "M0_SYNTHETIC_SUBSET_NOT_PHYSICS" || model["kind"] != "Pcb" {
        return Err(bad("UnsupportedModelScope"));
    }
    let source = &model["source"];
    object_keys(
        source,
        &[
            "format",
            "format_version",
            "importer_version",
            "producer",
            "producer_version",
            "files",
            "conditions",
            "capability",
        ],
    )?;
    if source["format"] != "Synthetic" {
        return Err(bad("NonSyntheticSource"));
    }
    for k in [
        "format_version",
        "importer_version",
        "producer",
        "producer_version",
    ] {
        nonempty(&source[k])?;
    }
    let files = array(&source["files"], 1, 64)?;
    let mut previous: Option<&str> = None;
    for f in files {
        object_keys(f, &["logical_path", "claimed_sha256"])?;
        let name = nonempty(&f["logical_path"])?;
        if !source_path_valid(name) || previous.is_some_and(|last| last >= name)
        {
            return Err(bad("InvalidSourceFilePathOrOrder"));
        }
        digest_hex_wire(&f["claimed_sha256"])?;
        previous = Some(name);
    }
    let mut last_condition: Option<(String, String)> = None;
    for p in array(&source["conditions"], 0, 128)? {
        let pair = array(p, 2, 2)?;
        let entry = (nonempty(&pair[0])?.to_owned(), string(&pair[1])?.to_owned());
        if last_condition.as_ref().is_some_and(|last| last.0 >= entry.0) {
            return Err(bad("UnsortedImportCondition"));
        }
        last_condition = Some(entry);
    }
    let cap = &source["capability"];
    object_keys(
        cap,
        &[
            "geometry",
            "net",
            "pin_pad",
            "stackup",
            "reference",
            "model",
            "object_coverage",
        ],
    )?;
    for k in [
        "geometry",
        "net",
        "pin_pad",
        "stackup",
        "reference",
        "model",
    ] {
        if !["Complete", "Partial", "Missing", "Conflicting"].contains(&string(&cap[k])?) {
            return Err(bad("InvalidCapability"));
        }
    }
    for name in array(&cap["object_coverage"], 0, 1024)? {
        nonempty(name)?;
    }

    let frame = &model["frame"];
    object_keys(
        frame,
        &[
            "meta",
            "unit_label",
            "metres_per_tick",
            "origin_m",
            "turn",
            "mirror_x",
            "source_layer_order_reversed",
        ],
    )?;
    meta_wire(&frame["meta"], "Frame")?;
    nonempty(&frame["unit_label"])?;
    rational_wire(&frame["metres_per_tick"], true)?;
    for r in array(&frame["origin_m"], 2, 2)? {
        rational_wire(r, false)?;
    }
    if !["Zero", "One", "Two", "Three"].contains(&string(&frame["turn"])?) {
        return Err(bad("InvalidQuarterTurn"));
    }
    if !frame["mirror_x"].is_boolean() || !frame["source_layer_order_reversed"].is_boolean() {
        return Err(bad("InvalidFrameFlags"));
    }

    let material = &model["material"];
    object_keys(
        material,
        &[
            "meta",
            "name",
            "dk",
            "df",
            "conductivity_s_per_m",
            "valid_band",
        ],
    )?;
    let material_id = meta_wire(&material["meta"], "Material")?;
    nonempty(&material["name"])?;
    property_wire(&material["dk"], f64::MIN_POSITIVE)?;
    property_wire(&material["df"], 0.0)?;
    property_wire(&material["conductivity_s_per_m"], f64::MIN_POSITIVE)?;
    let band = &material["valid_band"];
    object_keys(band, &["min_hz", "max_hz", "source"])?;
    let min_f = finite_number(&band["min_hz"])?;
    let max_f = finite_number(&band["max_hz"])?;
    if min_f < 0.0 || max_f < min_f {
        return Err(bad("InvalidFrequencyBand"));
    }
    provenance_wire(&band["source"])?;

    let layer = &model["layer"];
    object_keys(
        layer,
        &[
            "meta",
            "name",
            "order",
            "kind",
            "z_m",
            "thickness_m",
            "material_object",
        ],
    )?;
    meta_wire(&layer["meta"], "Layer")?;
    nonempty(&layer["name"])?;
    if layer["order"].as_u64() != Some(0)
        || layer["kind"] != "Copper"
        || string(&layer["material_object"])? != material_id
    {
        return Err(bad("LayerMaterialReferenceMismatch"));
    }
    rational_property_wire(&layer["z_m"], false)?;
    rational_property_wire(&layer["thickness_m"], true)?;

    let net = &model["net"];
    object_keys(net, &["meta", "name", "role"])?;
    meta_wire(&net["meta"], "Net")?;
    nonempty(&net["name"])?;
    if !["Signal", "Ground", "Power", "Floating", "Unknown"].contains(&string(&net["role"])?) {
        return Err(bad("InvalidNetRole"));
    }

    let outline = &model["outline"];
    object_keys(outline, &["meta", "segments"])?;
    meta_wire(&outline["meta"], "Outline")?;
    let segments = array(&outline["segments"], 3, 4096)?;
    let mut first: Option<[i64; 2]> = None;
    let mut previous_end: Option<[i64; 2]> = None;
    for segment in segments {
        object_keys(segment, &["start", "end"])?;
        let start = point_wire(&segment["start"])?;
        let end = point_wire(&segment["end"])?;
        if start == end || previous_end.is_some_and(|last| last != start) {
            return Err(bad("OpenOrDegenerateOutline"));
        }
        if first.is_none() {
            first = Some(start);
        }
        previous_end = Some(end);
    }
    if first != previous_end {
        return Err(bad("OpenOutline"));
    }
    Ok(())
}

fn receipt_for_payload(payload: &[u8]) -> io::Result<Vec<u8>> {
    let v: Value = serde_json::from_slice(payload).map_err(|_| bad("InvalidV2Json"))?;
    let fields = v.as_object().ok_or_else(|| bad("NotObject"))?;
    if fields.len() != 8 || v["schema"] != 2 || v["kind"] != "SyntheticSnapshotV2" {
        return Err(bad("UnsupportedV2Schema"));
    }
    let run = v["run_id"].as_str().ok_or_else(|| bad("RunId"))?;
    if run.is_empty() || run.len() > 128 {
        return Err(bad("RunId"));
    }
    let points = v["points"].as_u64().ok_or_else(|| bad("Points"))?;
    let delay = v["delay_ms"].as_u64().ok_or_else(|| bad("Delay"))?;
    if !(1..=1000).contains(&points) || delay > 1000 {
        return Err(bad("SyntheticParameters"));
    }
    let source_json = v["model_json"]
        .as_str()
        .ok_or_else(|| bad("ModelMissing"))?;
    if source_json.len() > 32_768 {
        return Err(bad("ModelTooLarge"));
    }
    let model: Value = serde_json::from_str(source_json).map_err(|_| bad("ModelJsonInvalid"))?;
    // Reject non-canonical bytes before hashing; serde_json pinned, BTreeMap key order.
    if serde_json::to_string(&model).map_err(|_| bad("ModelEncode"))? != source_json
        || model.as_object().map_or(true, |o| o.len() != 8)
        || model["scope"] != "M0_SYNTHETIC_SUBSET_NOT_PHYSICS"
        || model["kind"] != "Pcb"
        || model["source"]["format"] != "Synthetic"
        || validate_wire_model(&model).is_err()
    {
        return Err(bad("NonCanonicalOrUnsupportedModel"));
    }
    let model_sha = digest(MODEL_DOMAIN, source_json.as_bytes());
    if v["model_sha256"] != model_sha {
        return Err(bad("ModelDigestMismatch"));
    }
    let snapshot_data = json!({"model_sha256":model_sha,"points":points,"delay_ms":delay});
    let snapshot_bytes = serde_json::to_vec(&snapshot_data).map_err(|_| bad("SnapshotEncode"))?;
    let snapshot_id = digest(SNAPSHOT_DOMAIN, &snapshot_bytes);
    if v["snapshot_id"] != snapshot_id {
        return Err(bad("SnapshotDigestMismatch"));
    }
    // Receipt is the only v2 response in this bounded phase. No SI solve performed.
    let receipt = json!({"schema":2,"type":"SyntheticModelReceipt","run_id":run,
        "model_sha256":model_sha,"snapshot_id":snapshot_id,"synthetic":true});
    serde_json::to_vec(&receipt).map_err(|_| bad("ReceiptEncode"))
}
pub fn serve_stdio() -> io::Result<()> {
    let payload = read_frame(&mut io::stdin().lock())?;
    let receipt = receipt_for_payload(&payload)?;
    write_frame(&mut io::stdout().lock(), &receipt)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_paths_fail_closed_on_canonical_forbidden_components() {
        for invalid in ["./board", "C:board", "a/./b", "../b", "a//b", "/root", "a\\b"] {
            assert!(!source_path_valid(invalid), "accepted {invalid}");
        }
        assert!(source_path_valid("synthetic/model-handoff"));
    }

    #[test]
    fn forged_canonical_nested_model_is_rejected_even_with_recomputed_digests() {
        // Attack model: all eight top-level keys, valid canonical JSON and fresh
        // *correct* SHA-256 IDs, but no nested material, net or frame schema.
        let forged_model = json!({
            "scope": "M0_SYNTHETIC_SUBSET_NOT_PHYSICS",
            "kind": "Pcb", "source": {"format": "Synthetic"},
            "frame": {}, "layer": {}, "material": {}, "net": {}, "outline": {}
        });
        let model_json = serde_json::to_string(&forged_model).unwrap();
        let model_sha = digest(MODEL_DOMAIN, model_json.as_bytes());
        let snapshot_bytes = serde_json::to_vec(&json!({
            "model_sha256":model_sha,"points":5,"delay_ms":0
        }))
        .unwrap();
        let snapshot_id = digest(SNAPSHOT_DOMAIN, &snapshot_bytes);
        let request = json!({
            "schema":2,"kind":"SyntheticSnapshotV2","run_id":"forged",
            "model_json":model_json,"model_sha256":model_sha,
            "snapshot_id":snapshot_id,"points":5,"delay_ms":0
        });
        assert!(validate_wire_model(&forged_model).is_err());
        assert!(receipt_for_payload(&serde_json::to_vec(&request).unwrap()).is_err());
    }
    #[test]
    fn sha256_nist_known_answers() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
    #[test]
    fn framed_bytes_reject_truncation_and_size() {
        assert!(read_frame(&mut &b"\x00\x00\x00\x06ok"[..]).is_err());
        assert!(read_frame(&mut &b"\x00\x01\x00\x01"[..]).is_err());
    }
}
