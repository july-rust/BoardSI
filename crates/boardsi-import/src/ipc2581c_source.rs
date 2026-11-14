//! E_025. Bounded IPC-2581 Revision C Layer/LogicalNet SOURCE inventory.
//! This is not XSD validation, physical stackup, copper connectivity or SI.
use boardsi_model::{
    CapabilityState, ConversionQuality, ConversionStep, DielectricKind, GeometryProvenance,
    ImportCapability, LayerKind, Net, NetRole, ObjectId, ObjectKind, ObjectMeta, Rational,
    SourceFile, SourceFormat, SourceId, SourceIdentity,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
const LIMIT: usize = 8 * 1024 * 1024;
const NAMESPACE: &str = "http://webstds.ipc.org/2581";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportError {
    pub code: &'static str,
}
fn err(code: &'static str) -> ImportError {
    ImportError { code }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SourceLayer {
    pub meta: ObjectMeta,
    pub name: String,
    pub layer_function: String,
    pub side: String,
    pub polarity: String,
    pub kind: LayerKind,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SourceNet {
    pub net: Net,
    /// Declared PinRefs, not verified electrical connections.
    pub pin_refs: Vec<(Option<String>, String)>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Inventory {
    pub identity: SourceId,
    pub selected_step: String,
    pub layers: Vec<SourceLayer>,
    pub nets: Vec<SourceNet>,
    pub capability: ImportCapability,
    pub unknown_layer_functions: usize,
    pub solver_ready: bool,
}
impl Inventory {
    pub fn diagnostic(&self) -> String {
        format!("format=IPC2581;revision=C;scope=SOURCE_PARTIAL;layers={};nets={};solver_ready=false;physical_si=NOT_RUN", self.layers.len(), self.nets.len())
    }
}
#[derive(Clone, Debug)]
struct Tag {
    path: String,
    attrs: BTreeMap<String, String>,
}
fn valid(c: char) -> bool {
    matches!(c as u32, 9 | 10 | 13 | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..=0x10FFFF)
}
fn decode(s: &str) -> Result<String, ImportError> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        if rest[..i].chars().any(|c| !valid(c)) {
            return Err(err("INVALID_XML_CHAR"));
        }
        out.push_str(&rest[..i]);
        rest = &rest[i + 1..];
        let n = rest.find(';').ok_or_else(|| err("BAD_ENTITY"))?;
        let name = &rest[..n];
        let c = match name {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let number = if let Some(h) = name.strip_prefix("#x") {
                    u32::from_str_radix(h, 16).ok()
                } else if let Some(d) = name.strip_prefix('#') {
                    d.parse::<u32>().ok()
                } else {
                    None
                };
                number
                    .and_then(char::from_u32)
                    .ok_or_else(|| err("BAD_ENTITY"))?
            }
        };
        if !valid(c) {
            return Err(err("INVALID_XML_CHAR"));
        }
        out.push(c);
        rest = &rest[n + 1..];
    }
    if rest.chars().any(|c| !valid(c)) {
        return Err(err("INVALID_XML_CHAR"));
    }
    out.push_str(rest);
    Ok(out)
}
fn name_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'.' | b':')
}
fn parse_tag(body: &str) -> Result<(String, BTreeMap<String, String>), ImportError> {
    let b = body.as_bytes();
    let mut i = 0;
    while i < b.len() && name_byte(b[i]) {
        i += 1;
    }
    if i == 0 || b[0].is_ascii_digit() {
        return Err(err("BAD_XML_TAG"));
    }
    let name = body[..i].to_owned();
    if name.contains(':') {
        return Err(err("PREFIXED_ELEMENT_UNSUPPORTED"));
    }
    let mut attrs = BTreeMap::new();
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == b.len() {
            break;
        }
        let start = i;
        while i < b.len() && name_byte(b[i]) {
            i += 1;
        }
        if i == start {
            return Err(err("BAD_XML_ATTRIBUTE"));
        }
        let key = &body[start..i];
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if b.get(i) != Some(&b'=') {
            return Err(err("BAD_XML_ATTRIBUTE"));
        }
        i += 1;
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        let quote = *b.get(i).ok_or_else(|| err("BAD_XML_ATTRIBUTE"))?;
        if quote != b'"' && quote != b'\'' {
            return Err(err("BAD_XML_ATTRIBUTE"));
        }
        i += 1;
        let start = i;
        while i < b.len() && b[i] != quote {
            if b[i] == b'<' {
                return Err(err("BAD_XML_ATTRIBUTE"));
            }
            i += 1;
        }
        if i == b.len() || i - start > 4096 {
            return Err(err("BAD_XML_ATTRIBUTE"));
        }
        let value = decode(&body[start..i])?;
        i += 1;
        if attrs.insert(key.to_owned(), value).is_some() {
            return Err(err("DUPLICATE_XML_ATTRIBUTE"));
        }
    }
    Ok((name, attrs))
}
/// Small, bounded XML lexical reader. Reject DTD/CDATA/PI except initial XML declaration.
fn scan(source: &str) -> Result<Vec<Tag>, ImportError> {
    let xml = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut pos = 0usize;
    let mut stack: Vec<String> = Vec::new();
    let mut tags = Vec::new();
    let mut roots = 0;
    while pos < xml.len() {
        let tail = &xml[pos..];
        let i = match tail.find('<') {
            Some(i) => i,
            None => {
                if tail.contains("]]>") {
                    return Err(err("INVALID_XML_TEXT"));
                }
                decode(tail)?;
                if !tail.trim().is_empty() {
                    return Err(err("TEXT_OUTSIDE_ROOT"));
                }
                break;
            }
        };
        if stack.is_empty() && !tail[..i].trim().is_empty() {
            return Err(err("TEXT_OUTSIDE_ROOT"));
        }
        if tail[..i].contains("]]>") {
            return Err(err("INVALID_XML_TEXT"));
        }
        decode(&tail[..i])?;
        pos += i;
        let tail = &xml[pos..];
        if let Some(after_comment_open) = tail.strip_prefix("<!--") {
            let n = after_comment_open
                .find("-->")
                .ok_or_else(|| err("UNCLOSED_XML_COMMENT"))?;
            let body = &after_comment_open[..n];
            if body.contains("--") || body.ends_with('-') {
                return Err(err("INVALID_XML_COMMENT"));
            }
            pos += 4 + n + 3;
            continue;
        }
        if tail.starts_with("<?xml ") && pos == 0 {
            let n = tail.find("?>").ok_or_else(|| err("BAD_XML_DECLARATION"))?;
            pos += n + 2;
            continue;
        }
        if tail.starts_with("<!") || tail.starts_with("<?") {
            return Err(err("UNSAFE_XML_MARKUP"));
        }
        let mut quote = 0;
        let mut gt = None;
        for (n, c) in tail.as_bytes().iter().copied().enumerate().skip(1) {
            if quote != 0 {
                if c == quote {
                    quote = 0;
                }
            } else if c == b'"' || c == b'\'' {
                quote = c;
            } else if c == b'<' {
                return Err(err("NESTED_XML_TAG"));
            } else if c == b'>' {
                gt = Some(n);
                break;
            }
        }
        let gt = gt.ok_or_else(|| err("UNCLOSED_XML_TAG"))?;
        let raw = tail[1..gt].trim();
        if let Some(close) = raw.strip_prefix('/') {
            let open = stack.pop().ok_or_else(|| err("EXTRA_XML_CLOSE"))?;
            if close.trim() != open {
                return Err(err("MISMATCHED_XML_CLOSE"));
            }
        } else {
            let empty = raw.ends_with('/');
            let body = if empty {
                raw[..raw.len() - 1].trim_end()
            } else {
                raw
            };
            let (name, attrs) = parse_tag(body)?;
            if stack.is_empty() {
                roots += 1;
            }
            stack.push(name);
            if stack.len() > 64 {
                return Err(err("XML_DEPTH_LIMIT"));
            }
            if stack.len() > 1
                && attrs
                    .keys()
                    .any(|k| k == "xmlns" || k.starts_with("xmlns:"))
            {
                return Err(err("NESTED_NAMESPACE_UNSUPPORTED"));
            }
            tags.push(Tag {
                path: stack.join("/"),
                attrs,
            });
            if tags.len() > 100_000 {
                return Err(err("XML_NODE_LIMIT"));
            }
            if empty {
                stack.pop();
            }
        }
        pos += gt + 1;
    }
    if roots != 1 || !stack.is_empty() {
        return Err(err("INCOMPLETE_XML"));
    }
    Ok(tags)
}
fn field<'a>(attrs: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, ImportError> {
    let v = attrs.get(key).ok_or_else(|| err("MISSING_ATTRIBUTE"))?;
    if v.is_empty() || v.len() > 256 || v.chars().any(char::is_control) {
        return Err(err("BAD_IDENTIFIER"));
    }
    Ok(v)
}
fn classify(f: &str) -> (LayerKind, bool) {
    match f {
        "CONDUCTOR" | "CONDFILM" | "CONDFOIL" | "PLANE" | "SIGNAL" | "MIXED" => {
            (LayerKind::Copper, true)
        }
        "DIELCORE" => (LayerKind::Dielectric(DielectricKind::Core), true),
        "DIELPREG" => (LayerKind::Dielectric(DielectricKind::Prepreg), true),
        "DIELBASE" | "DIELBONDPLY" | "DIELCOVERLAY" | "DIELADHV" => {
            (LayerKind::Dielectric(DielectricKind::Other), true)
        }
        _ => (LayerKind::Other, false),
    }
}
fn object(source: &SourceId, kind: ObjectKind, name: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, name, ""),
        provenance: GeometryProvenance {
            source_object_type: "IPC-2581C declaration (not physical geometry)".into(),
            history: vec![ConversionStep {
                operation: "preserve-source-declaration".into(),
                quality: ConversionQuality::Exact,
                max_error_m: Some(Rational::integer(0)),
                note: "No z, plating, electrical connectivity or SI inference".into(),
            }],
        },
    }
}
/// Opt-in source inventory. No Canonical BoardModel is emitted from Layer/Net alone.
pub fn inspect(bytes: &[u8]) -> Result<Inventory, ImportError> {
    if bytes.len() > LIMIT {
        return Err(err("INPUT_TOO_LARGE"));
    }
    let xml = std::str::from_utf8(bytes).map_err(|_| err("INVALID_UTF8"))?;
    let tags = scan(xml)?;
    let root = tags.first().ok_or_else(|| err("EMPTY_XML"))?;
    if root.path != "IPC-2581" {
        return Err(err("BAD_ROOT"));
    }
    if field(&root.attrs, "revision")? != "C" {
        return Err(err("UNSUPPORTED_REVISION"));
    }
    if field(&root.attrs, "xmlns")? != NAMESPACE {
        return Err(err("UNSUPPORTED_NAMESPACE"));
    }
    let mut raw_layers: Vec<(String, String, String, String)> = Vec::new();
    type RawNet = (String, Vec<(Option<String>, String)>);
    let mut raw_nets: Vec<RawNet> = Vec::new();
    let mut layer_names: BTreeSet<String> = BTreeSet::new();
    let mut net_names: BTreeSet<String> = BTreeSet::new();
    let mut layer_refs: BTreeSet<String> = BTreeSet::new();
    let mut step: Option<String> = None;
    let (mut content, mut cad_data) = (false, false);
    for tag in &tags {
        match tag.path.as_str() {
            "IPC-2581/Content" => {
                if content {
                    return Err(err("DUPLICATE_CONTENT"));
                }
                content = true;
            }
            "IPC-2581/Ecad/CadData" => {
                if cad_data {
                    return Err(err("DUPLICATE_CADDATA"));
                }
                cad_data = true;
            }
            "IPC-2581/Content/LayerRef" => {
                let name = field(&tag.attrs, "name")?;
                if !layer_refs.insert(name.to_owned()) {
                    return Err(err("DUPLICATE_LAYER_REF"));
                }
            }
            "IPC-2581/Ecad/CadData/Layer" => {
                let name = field(&tag.attrs, "name")?;
                if !layer_names.insert(name.into()) {
                    return Err(err("DUPLICATE_LAYER"));
                }
                if raw_layers.len() >= 4096 {
                    return Err(err("LAYER_LIMIT"));
                }
                let side = field(&tag.attrs, "side")?;
                if !matches!(
                    side,
                    "TOP" | "BOTTOM" | "BOTH" | "INTERNAL" | "ALL" | "NONE"
                ) {
                    return Err(err("INVALID_LAYER_SIDE"));
                }
                let polarity = field(&tag.attrs, "polarity")?;
                if !matches!(polarity, "POSITIVE" | "NEGATIVE") {
                    return Err(err("INVALID_LAYER_POLARITY"));
                }
                raw_layers.push((
                    name.into(),
                    field(&tag.attrs, "layerFunction")?.into(),
                    side.into(),
                    polarity.into(),
                ));
            }
            "IPC-2581/Ecad/CadData/Step" => {
                let name = field(&tag.attrs, "name")?;
                if step.replace(name.into()).is_some() {
                    return Err(err("MULTI_STEP_UNSUPPORTED"));
                }
            }
            "IPC-2581/Ecad/CadData/Step/LogicalNet" => {
                let name = field(&tag.attrs, "name")?;
                if !net_names.insert(name.into()) {
                    return Err(err("DUPLICATE_NET"));
                }
                if raw_nets.len() >= 65536 {
                    return Err(err("NET_LIMIT"));
                }
                raw_nets.push((name.into(), Vec::new()));
            }
            "IPC-2581/Ecad/CadData/Step/LogicalNet/PinRef" => {
                let pin = field(&tag.attrs, "pin")?;
                let comp = tag.attrs.get("componentRef").cloned();
                if comp
                    .as_deref()
                    .is_some_and(|v| v.is_empty() || v.len() > 256)
                {
                    return Err(err("BAD_IDENTIFIER"));
                }
                raw_nets
                    .last_mut()
                    .ok_or_else(|| err("ORPHAN_PIN_REF"))?
                    .1
                    .push((comp, pin.into()));
            }
            _ => {}
        }
    }
    if !content || !cad_data || step.is_none() || raw_layers.is_empty() || raw_nets.is_empty() {
        return Err(err("MISSING_SOURCE_DECLARATIONS"));
    }
    if layer_refs.iter().any(|r| !layer_names.contains(r)) {
        return Err(err("UNKNOWN_LAYER_REF"));
    }
    if raw_nets.iter().any(|(_, pins)| pins.is_empty()) {
        return Err(err("NET_MISSING_PIN_REF"));
    }
    let selected_step = step.expect("validated");
    let source = Arc::new(SourceIdentity {
        format: SourceFormat::Ipc2581,
        format_version: "C".into(),
        files: vec![SourceFile {
            logical_path: "input.xml".into(),
            sha256: crate::digest::sha256(bytes),
        }],
        importer_version: env!("CARGO_PKG_VERSION").into(),
        conditions: vec![
            ("selected_step".into(), selected_step.clone()),
            ("scope".into(), "source-layer-logical-net".into()),
        ],
    });
    let mut unknown_layer_functions = 0;
    let layers = raw_layers
        .into_iter()
        .map(|(name, f, side, polarity)| {
            let (kind, known) = classify(&f);
            if !known {
                unknown_layer_functions += 1;
            }
            SourceLayer {
                meta: object(&source, ObjectKind::Layer, &name),
                name,
                layer_function: f,
                side,
                polarity,
                kind,
            }
        })
        .collect();
    let nets = raw_nets
        .into_iter()
        .map(|(name, pin_refs)| SourceNet {
            net: Net {
                meta: object(
                    &source,
                    ObjectKind::Net,
                    &format!("{selected_step}/LogicalNet/{name}"),
                ),
                name,
                role: NetRole::Unknown,
            },
            pin_refs,
        })
        .collect();
    Ok(Inventory {
        identity: source,
        selected_step,
        layers,
        nets,
        unknown_layer_functions,
        capability: ImportCapability {
            geometry: CapabilityState::Missing,
            net: CapabilityState::Partial,
            pin_pad: CapabilityState::Missing,
            stackup: CapabilityState::Partial,
            reference: CapabilityState::Missing,
            model: CapabilityState::Missing,
            object_coverage: vec!["Layer declaration".into(), "Step LogicalNet/PinRef".into()],
        },
        solver_ready: false,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample(count: usize) -> String {
        use std::fmt::Write as _;
        let mut layers = String::new();
        let mut refs = String::new();
        for n in 0..count {
            write!(
                &mut layers,
                "<Layer name=\"L{n}\" layerFunction=\"SIGNAL\" side=\"INTERNAL\" polarity=\"POSITIVE\"/>"
            )
            .expect("String is writable");
            write!(&mut refs, "<LayerRef name=\"L{n}\"/>").expect("String is writable");
        }
        format!("<?xml version=\"1.0\"?><IPC-2581 revision=\"C\" xmlns=\"http://webstds.ipc.org/2581\"><Content>{refs}</Content><LogisticHeader/><HistoryRecord/><Ecad><CadHeader units=\"MILLIMETER\"/><CadData>{layers}<Step name=\"BOARD\"><LogicalNet name=\"SIG\"><PinRef componentRef=\"U1\" pin=\"1\"/></LogicalNet></Step></CadData></Ecad></IPC-2581>")
    }
    fn reject(s: String, reason: &str) {
        assert_eq!(inspect(s.as_bytes()).unwrap_err().code, reason);
    }
    #[test]
    fn inventory_is_deterministic_partial_and_bound_to_sha() {
        let s = sample(4);
        let p = inspect(s.as_bytes()).unwrap();
        assert_eq!(p, inspect(s.as_bytes()).unwrap());
        assert_eq!(p.layers.len(), 4);
        assert_eq!(p.nets[0].net.name, "SIG");
        assert_eq!(p.nets[0].pin_refs[0], (Some("U1".into()), "1".into()));
        assert_eq!(p.layers[0].meta.id.source_object, "L0");
        assert_eq!(
            p.identity.files[0].sha256,
            crate::digest::sha256(s.as_bytes())
        );
        assert_eq!(p.capability.net, CapabilityState::Partial);
        assert!(!p.solver_ready);
        assert_ne!(
            p.identity,
            inspect(s.replace("SIG", "SIG2").as_bytes())
                .unwrap()
                .identity
        );
    }
    #[test]
    fn layer_counts_and_unknown_functions() {
        for n in [2, 4, 6, 8, 64] {
            assert_eq!(inspect(sample(n).as_bytes()).unwrap().layers.len(), n);
        }
        let p = inspect(sample(2).replace("SIGNAL", "NOT_DEFINED").as_bytes()).unwrap();
        assert_eq!(p.unknown_layer_functions, 2);
        assert_eq!(p.layers[0].kind, LayerKind::Other);
    }
    #[test]
    fn revision_namespace_and_source_mutations_fail_closed() {
        let s = sample(2);
        reject(
            s.replace("revision=\"C\"", "revision=\"B\""),
            "UNSUPPORTED_REVISION",
        );
        reject(
            s.replace("http://webstds.ipc.org/2581", "http://bad"),
            "UNSUPPORTED_NAMESPACE",
        );
        reject(s.replace("<IPC-2581 ", "<Fake "), "MISMATCHED_XML_CLOSE");
        reject(s.replace("</IPC-2581>", "</Fake>"), "MISMATCHED_XML_CLOSE");
        reject(
            s.replace("name=\"L1\"", "name=\"L0\""),
            "DUPLICATE_LAYER_REF",
        );
        reject(
            s.replace("<LayerRef name=\"L1\"/>", "<LayerRef name=\"MISSING\"/>"),
            "UNKNOWN_LAYER_REF",
        );
        reject(s.replace("name=\"SIG\"", ""), "MISSING_ATTRIBUTE");
        reject(s.replace(" pin=\"1\"", ""), "MISSING_ATTRIBUTE");
        reject(
            s.replace("<PinRef componentRef=\"U1\" pin=\"1\"/>", ""),
            "NET_MISSING_PIN_REF",
        );
        reject(format!("<!DOCTYPE D>{s}"), "UNSAFE_XML_MARKUP");
        reject(s.replace("SIG", "&evil;"), "BAD_ENTITY");
        reject(
            s.replace("<Content>", "<Content xmlns=\"http://other\">"),
            "NESTED_NAMESPACE_UNSUPPORTED",
        );
    }
    #[test]
    fn lexical_boundaries_and_layer_enums() {
        let s = sample(2);
        assert_eq!(
            inspect(format!("{s}\n").as_bytes()).unwrap().layers.len(),
            2
        );
        reject(format!("garbage{s}"), "TEXT_OUTSIDE_ROOT");
        reject(
            s.replace("side=\"INTERNAL\"", "side=\"UNKNOWN\""),
            "INVALID_LAYER_SIDE",
        );
        reject(
            s.replace("polarity=\"POSITIVE\"", "polarity=\"UNKNOWN\""),
            "INVALID_LAYER_POLARITY",
        );
    }
    #[test]
    fn illegal_xml_comment_and_cdata_terminator_fail_closed() {
        let s = sample(2);
        reject(
            s.replace("<Content>", "<!--bad -- double-hyphen--><Content>"),
            "INVALID_XML_COMMENT",
        );
        reject(
            s.replace("<Content>", "<!--trailing-dash---><Content>"),
            "INVALID_XML_COMMENT",
        );
        reject(s.replace("<Content>", "<Content>]]>"), "INVALID_XML_TEXT");
        reject(format!("{s}]]>"), "INVALID_XML_TEXT");
    }
    #[test]
    fn duplicate_nets_and_size_are_rejected() {
        let s = sample(2);
        reject(
            s.replace(
                "</LogicalNet>",
                "</LogicalNet><LogicalNet name=\"SIG\"><PinRef pin=\"1\"/></LogicalNet>",
            ),
            "DUPLICATE_NET",
        );
        assert_eq!(
            inspect(&vec![0; LIMIT + 1]).unwrap_err().code,
            "INPUT_TOO_LARGE"
        );
    }
}
