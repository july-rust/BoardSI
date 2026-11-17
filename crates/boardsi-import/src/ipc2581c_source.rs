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
// Limit repeated ancestor-name copies across all stored paths, independently of input bytes.
const PATH_STORAGE_LIMIT: usize = 8 * 1024 * 1024;
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
                    if !h.is_empty() && h.bytes().all(|b| b.is_ascii_hexdigit()) {
                        u32::from_str_radix(h, 16).ok()
                    } else {
                        None
                    }
                } else if let Some(d) = name.strip_prefix('#') {
                    if !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()) {
                        d.parse::<u32>().ok()
                    } else {
                        None
                    }
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
fn xml_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\r' | b'\n')
}
fn only_xml_space(s: &str) -> bool {
    s.bytes().all(xml_space)
}
fn normalize_literal_attr(s: &str) -> String {
    // XML 1.0 line-end normalization runs before attribute normalization.
    // Literal TAB/LF/CR become spaces; character references are decoded later.
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push(' ');
            }
            '\t' | '\n' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}
fn name_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
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
    if i == 0 || !name_start(b[0]) {
        return Err(err("BAD_XML_TAG"));
    }
    let name = body[..i].to_owned();
    if name.contains(':') {
        return Err(err("PREFIXED_ELEMENT_UNSUPPORTED"));
    }
    let mut attrs = BTreeMap::new();
    while i < b.len() {
        let separator_start = i;
        while i < b.len() && xml_space(b[i]) {
            i += 1;
        }
        if i == b.len() {
            break;
        }
        // XML requires at least one S character between attributes.
        if i == separator_start {
            return Err(err("BAD_XML_ATTRIBUTE"));
        }
        let start = i;
        while i < b.len() && name_byte(b[i]) {
            i += 1;
        }
        if i == start || !name_start(b[start]) {
            return Err(err("BAD_XML_ATTRIBUTE"));
        }
        let key = &body[start..i];
        while i < b.len() && xml_space(b[i]) {
            i += 1;
        }
        if b.get(i) != Some(&b'=') {
            return Err(err("BAD_XML_ATTRIBUTE"));
        }
        i += 1;
        while i < b.len() && xml_space(b[i]) {
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
        let value = decode(&normalize_literal_attr(&body[start..i]))?;
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
    let mut stored_path_bytes = 0usize;
    while pos < xml.len() {
        let tail = &xml[pos..];
        let i = match tail.find('<') {
            Some(i) => i,
            None => {
                if tail.contains("]]>") {
                    return Err(err("INVALID_XML_TEXT"));
                }
                decode(tail)?;
                if !only_xml_space(tail) {
                    return Err(err("TEXT_OUTSIDE_ROOT"));
                }
                break;
            }
        };
        if stack.is_empty() && !only_xml_space(&tail[..i]) {
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
            if body.chars().any(|c| !valid(c)) {
                return Err(err("INVALID_XML_CHAR"));
            }
            pos += 4 + n + 3;
            continue;
        }
        if tail.starts_with("<?xml") && pos == 0 {
            let n = tail.find("?>").ok_or_else(|| err("BAD_XML_DECLARATION"))?;
            if tail[..n + 2].chars().any(|c| !valid(c)) {
                return Err(err("INVALID_XML_CHAR"));
            }
            let declaration = &tail[2..n];
            if declaration.contains('&') {
                return Err(err("BAD_XML_DECLARATION"));
            }
            let (name, attrs) = parse_tag(declaration).map_err(|_| err("BAD_XML_DECLARATION"))?;
            if name != "xml"
                || attrs.get("version").map(String::as_str) != Some("1.0")
                || attrs
                    .keys()
                    .any(|k| !matches!(k.as_str(), "version" | "encoding" | "standalone"))
                || attrs
                    .get("encoding")
                    .is_some_and(|v| !v.eq_ignore_ascii_case("utf-8"))
                || attrs
                    .get("standalone")
                    .is_some_and(|v| v != "yes" && v != "no")
            {
                return Err(err("BAD_XML_DECLARATION"));
            }
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
        let raw = &tail[1..gt];
        if let Some(close) = raw.strip_prefix('/') {
            let close = close.trim_end_matches(|c| matches!(c, ' ' | '\t' | '\r' | '\n'));
            let open = stack.pop().ok_or_else(|| err("EXTRA_XML_CLOSE"))?;
            if close != open {
                return Err(err("MISMATCHED_XML_CLOSE"));
            }
        } else {
            // In XML, empty elements end in "/>" with no S between '/' and '>'.
            let empty = raw.ends_with('/');
            let body = if empty {
                raw[..raw.len() - 1].trim_end_matches(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
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
            // input-length/node/depth caps alone cannot bound repeated stack.join output.
            // Compute the exact path allocation and aggregate before allocating.
            let path_len = stack
                .iter()
                .try_fold(0usize, |n, part| n.checked_add(part.len()))
                .and_then(|n| n.checked_add(stack.len().saturating_sub(1)))
                .ok_or_else(|| err("XML_PATH_STORAGE_LIMIT"))?;
            stored_path_bytes = stored_path_bytes
                .checked_add(path_len)
                .ok_or_else(|| err("XML_PATH_STORAGE_LIMIT"))?;
            if stored_path_bytes > PATH_STORAGE_LIMIT {
                return Err(err("XML_PATH_STORAGE_LIMIT"));
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
                if comp.as_deref().is_some_and(|v| {
                    v.is_empty() || v.len() > 256 || v.chars().any(char::is_control)
                }) {
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
    fn malformed_xml_attribute_boundaries_and_control_chars_fail_closed() {
        let s = sample(2);
        reject(
            s.replace("name=\"L0\" layerFunction", "name=\"L0\"layerFunction"),
            "BAD_XML_ATTRIBUTE",
        );
        reject(
            s.replace(
                "name=\"L0\" layerFunction",
                "name=\"L0\"\u{000b}layerFunction",
            ),
            "BAD_XML_ATTRIBUTE",
        );
        reject(
            s.replace("<IPC-2581 revision", "< IPC-2581 revision"),
            "BAD_XML_TAG",
        );
        reject(
            s.replace("</IPC-2581>", "</ IPC-2581>"),
            "MISMATCHED_XML_CLOSE",
        );
        reject(
            s.replace("</IPC-2581>", "</IPC-2581\u{000b}>"),
            "MISMATCHED_XML_CLOSE",
        );
        reject(
            s.replace("polarity=\"POSITIVE\"/>", "polarity=\"POSITIVE\"/ >"),
            "BAD_XML_ATTRIBUTE",
        );
        reject(
            s.replace("componentRef=\"U1\"", "componentRef=\"U&#10;1\""),
            "BAD_IDENTIFIER",
        );
        reject(
            s.replace("<Content>", "<!--\u{0001}--><Content>"),
            "INVALID_XML_CHAR",
        );
        reject(
            s.replace("<?xml version", "<?xml \u{0001}version"),
            "INVALID_XML_CHAR",
        );
    }
    #[test]
    fn numeric_entities_and_xml_boundary_whitespace_fail_closed() {
        let s = sample(2);
        reject(s.replace("name=\"SIG\"", "name=\"&#+65;\""), "BAD_ENTITY");
        reject(s.replace("name=\"SIG\"", "name=\"&#x+41;\""), "BAD_ENTITY");
        assert_eq!(
            inspect(s.replace("name=\"SIG\"", "name=\"&#65;\"").as_bytes())
                .unwrap()
                .nets[0]
                .net
                .name,
            "A"
        );
        reject(format!("{s}\u{a0}"), "TEXT_OUTSIDE_ROOT");
        reject(format!("\u{a0}{s}"), "TEXT_OUTSIDE_ROOT");
    }
    #[test]
    fn xml_declaration_and_attribute_normalization() {
        let s = sample(2);
        assert!(inspect(s.replace("<?xml version", "<?xml\tversion").as_bytes()).is_ok());
        reject(
            s.replace("<?xml version=\"1.0\"?>", "<?xml ?>"),
            "BAD_XML_DECLARATION",
        );
        reject(
            s.replace("version=\"1.0\"", "version=\"1.1\""),
            "BAD_XML_DECLARATION",
        );
        reject(
            s.replace(
                "<?xml version=\"1.0\"?>",
                "<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>",
            ),
            "BAD_XML_DECLARATION",
        );
        let tabs = inspect(s.replace("name=\"SIG\"", "name=\"S\tIG\"").as_bytes()).unwrap();
        assert_eq!(tabs.nets[0].net.name, "S IG");
        let crlf = inspect(s.replace("name=\"SIG\"", "name=\"S\r\nIG\"").as_bytes()).unwrap();
        assert_eq!(crlf.nets[0].net.name, "S IG");
        reject(
            s.replace("name=\"SIG\"", "name=\"S&#10;IG\""),
            "BAD_IDENTIFIER",
        );
    }
    #[test]
    fn repeated_ancestor_path_storage_is_bounded() {
        let s = sample(2);
        let long_name = "A".repeat(16_384);
        let unknown = format!("<{long_name}>{}</{long_name}>", "<X/>".repeat(1_000));
        let oversized_paths =
            s.replace("<LogisticHeader/>", &format!("<LogisticHeader/>{unknown}"));
        assert!(oversized_paths.len() < LIMIT);
        reject(oversized_paths, "XML_PATH_STORAGE_LIMIT");
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

// F_007: opt-in strictly bounded native IPC-2581C copper Line/Polyline adapter.
// XSD: CadHeader.units; CadData.Step.LayerFeature.Set.Features.Line or Polyline,
// with an inline LineDesc. Any other Step fabrication geometry fails closed.
// This is SOURCE artwork, never manufactured connectivity, stackup, or SI.
fn f007_nm(raw: &str, factor: i128) -> Result<i64, ImportError> {
    if raw.is_empty() || raw.len() > 40 {
        return Err(err("F007_BAD_COORDINATE"));
    }
    let (negative, unsigned) = if let Some(s) = raw.strip_prefix('-') {
        (true, s)
    } else if let Some(s) = raw.strip_prefix('+') {
        (false, s)
    } else {
        (false, raw)
    };
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some()
        || (whole.is_empty() && fraction.is_empty())
        || (!whole.is_empty() && !whole.bytes().all(|b| b.is_ascii_digit()))
        || (!fraction.is_empty() && !fraction.bytes().all(|b| b.is_ascii_digit()))
        || fraction.len() > 18
    {
        return Err(err("F007_BAD_COORDINATE"));
    }
    let denom = 10_i128
        .checked_pow(fraction.len() as u32)
        .ok_or_else(|| err("F007_COORDINATE_OVERFLOW"))?;
    let int = if whole.is_empty() {
        0
    } else {
        whole
            .parse::<i128>()
            .map_err(|_| err("F007_COORDINATE_OVERFLOW"))?
    };
    let frac = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<i128>()
            .map_err(|_| err("F007_COORDINATE_OVERFLOW"))?
    };
    let decimal = int
        .checked_mul(denom)
        .and_then(|n| n.checked_add(frac))
        .and_then(|n| n.checked_mul(factor))
        .ok_or_else(|| err("F007_COORDINATE_OVERFLOW"))?;
    if decimal % denom != 0 {
        return Err(err("F007_SUB_NM_UNREPRESENTABLE"));
    }
    let nm = decimal / denom;
    let nm = if negative { -nm } else { nm };
    let value = i64::try_from(nm).map_err(|_| err("F007_COORDINATE_OVERFLOW"))?;
    if value.unsigned_abs() > 1_000_000_000_000 {
        return Err(err("F007_QT_COORDINATE_LIMIT"));
    }
    Ok(value)
}

pub fn into_native_canonical(bytes: &[u8]) -> Result<boardsi_model::BoardModel, ImportError> {
    use boardsi_model::{
        BoardKind, BoardModel, CoordinateFrame, ImportSource, Layer, Orientation, Point, Property,
        QuarterTurn, Trace,
    };
    let inventory = inspect(bytes)?;
    if inventory.layers.len() > 64 || inventory.nets.len() > 128 {
        return Err(err("F007_NATIVE_INVENTORY_LIMIT"));
    }
    let xml = std::str::from_utf8(bytes).map_err(|_| err("INVALID_UTF8"))?;
    let tags = scan(xml)?;
    let mut headers = tags.iter().filter(|t| t.path == "IPC-2581/Ecad/CadHeader");
    let header = headers
        .next()
        .ok_or_else(|| err("F007_MISSING_CAD_UNITS"))?;
    if headers.next().is_some() {
        return Err(err("F007_DUPLICATE_CAD_HEADER"));
    }
    let units = field(&header.attrs, "units")?;
    let factor: i128 = match units {
        "MILLIMETER" => 1_000_000,
        "MICRON" => 1_000,
        "INCH" => 25_400_000,
        _ => return Err(err("F007_UNSUPPORTED_CAD_UNIT")),
    };
    if header.attrs.len() != 1 {
        return Err(err("F007_UNSUPPORTED_CAD_HEADER_ATTRIBUTE"));
    }

    let mut source_data = (*inventory.identity).clone();
    let mut conditions = vec![
        ("selected_step".into(), inventory.selected_step.clone()),
        (
            "scope".into(),
            "f007-ipc2581c-inline-round-line-polyline-nm-source-only".into(),
        ),
        ("cad_units".into(), units.into()),
    ];
    for (i, l) in inventory.layers.iter().enumerate() {
        conditions.push((format!("layer/{i:03}/name"), l.name.clone()));
        conditions.push((format!("layer/{i:03}/function"), l.layer_function.clone()));
        conditions.push((format!("layer/{i:03}/side"), l.side.clone()));
        conditions.push((format!("layer/{i:03}/polarity"), l.polarity.clone()));
    }
    let mut pin_count = 0usize;
    for (i, n) in inventory.nets.iter().enumerate() {
        for (j, (comp, pin)) in n.pin_refs.iter().enumerate() {
            pin_count += 1;
            if pin_count > 512 {
                return Err(err("F007_PINREF_LIMIT"));
            }
            conditions.push((
                format!("logical_net/{i:03}/pin_ref/{j:04}/pin"),
                pin.clone(),
            ));
            if let Some(component) = comp {
                conditions.push((
                    format!("logical_net/{i:03}/pin_ref/{j:04}/component"),
                    component.clone(),
                ));
            }
        }
    }
    conditions.sort_by(|a, b| a.0.cmp(&b.0));
    source_data.conditions = conditions;
    let source = Arc::new(source_data);
    let mut board = BoardModel::empty(BoardKind::Pcb);
    board.sources.push(ImportSource {
        id: source.clone(),
        producer: "IPC-2581C Source".into(),
        producer_version: "C".into(),
        capability: ImportCapability {
            geometry: CapabilityState::Partial,
            net: CapabilityState::Partial,
            pin_pad: CapabilityState::Missing,
            stackup: CapabilityState::Partial,
            reference: CapabilityState::Missing,
            model: CapabilityState::Missing,
            object_coverage: vec![
                "IPC-2581C inline round-ended solid Line/straight Polyline SOURCE artwork".into(),
                "No pads/vias, physical net graph, material stackup or SI".into(),
            ],
        },
    });
    let frame_meta = object(&source, ObjectKind::Frame, "ipc2581c-xy-nm");
    let frame_id = frame_meta.id.clone();
    board.frames.push(CoordinateFrame {
        meta: frame_meta,
        unit_label: format!("IPC-2581C {units}, exact integer nm"),
        metres_per_tick: Rational::new(1, 1_000_000_000).map_err(|_| err("F007_BAD_UNIT"))?,
        origin_m: [Rational::integer(0); 2],
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        source_layer_order_reversed: false,
    });

    let mut layers = BTreeMap::<String, (ObjectId, LayerKind, String)>::new();
    let mut copper_count = 0usize;
    for (index, l) in inventory.layers.iter().enumerate() {
        let m = object(&source, ObjectKind::Layer, &l.name);
        if l.kind == LayerKind::Copper {
            copper_count += 1;
        }
        layers.insert(l.name.clone(), (m.id.clone(), l.kind, l.polarity.clone()));
        board.stackup.layers.push(Layer {
            meta: m,
            name: l.name.clone(),
            order: index as u32, // source declaration order; physical z still MISSING
            kind: l.kind,
            z_m: Property::Missing,
            thickness_m: Property::Missing,
            material: None,
        });
    }
    if !(2..=32).contains(&copper_count) {
        return Err(err("F007_COPPER_LAYER_LIMIT"));
    }
    let mut nets = BTreeMap::<String, ObjectId>::new();
    for n in &inventory.nets {
        let m = object(
            &source,
            ObjectKind::Net,
            &format!("{}/LogicalNet/{}", inventory.selected_step, n.net.name),
        );
        nets.insert(n.net.name.clone(), m.id.clone());
        board.nets.push(Net {
            meta: m,
            name: n.net.name.clone(),
            role: n.net.role,
        });
    }

    let mut layer: Option<ObjectId> = None;
    let mut net: Option<ObjectId> = None;
    let mut current_kind = "";
    let mut pending_points: Vec<[i64; 2]> = Vec::new();
    let mut traces = 0usize;
    let mut preview_segments = 0usize;
    let mut poly_begin_seen = false;
    let mut feature_count = 0usize;
    let mut entered_features = false;
    for tag in &tags {
        // Unqualified CAD extensions under CadData would be silently dropped.
        // Accept only the exact Source geometry tree qualified below.
        let raw = match tag.path.strip_prefix("IPC-2581/Ecad/CadData/") {
            Some(raw) => raw,
            None => continue,
        };
        match raw {
            "Layer" => {
                if tag.attrs.len() != 4
                    || tag.attrs.keys().any(|k| {
                        !matches!(k.as_str(), "name" | "layerFunction" | "side" | "polarity")
                    })
                {
                    return Err(err("F007_UNSUPPORTED_LAYER_ATTRIBUTE"));
                }
            }
            "Step" => {
                if tag.attrs.len() != 1 {
                    return Err(err("F007_UNSUPPORTED_STEP_ATTRIBUTE"));
                }
            }
            "Step/LogicalNet" => {
                if tag.attrs.len() != 1 {
                    return Err(err("F007_UNSUPPORTED_NET_ATTRIBUTE"));
                }
            }
            "Step/LogicalNet/PinRef" => {
                if tag
                    .attrs
                    .keys()
                    .any(|k| !matches!(k.as_str(), "pin" | "componentRef"))
                {
                    return Err(err("F007_UNSUPPORTED_PINREF_ATTRIBUTE"));
                }
            }
            "Step/LayerFeature" => {
                if !pending_points.is_empty() || tag.attrs.len() != 1 {
                    return Err(err("F007_UNSUPPORTED_LAYER_FEATURE"));
                }
                let name = field(&tag.attrs, "layerRef")?;
                let (id, kind, polarity) =
                    layers.get(name).ok_or_else(|| err("F007_UNKNOWN_LAYER"))?;
                if *kind != LayerKind::Copper || polarity != "POSITIVE" {
                    return Err(err("F007_UNSUPPORTED_LAYER_POLARITY"));
                }
                layer = Some(id.clone());
                net = None;
            }
            "Step/LayerFeature/Set" => {
                if !pending_points.is_empty()
                    || tag.attrs.keys().any(|k| k != "net" && k != "polarity")
                    || tag.attrs.get("polarity").is_some_and(|v| v != "POSITIVE")
                {
                    return Err(err("F007_UNSUPPORTED_SET"));
                }
                net = tag
                    .attrs
                    .get("net")
                    .map(|name| {
                        nets.get(name)
                            .cloned()
                            .ok_or_else(|| err("F007_UNKNOWN_NET"))
                    })
                    .transpose()?;
            }
            "Step/LayerFeature/Set/Features" => {
                if !tag.attrs.is_empty()
                    || layer.is_none()
                    || !current_kind.is_empty()
                    || (entered_features && feature_count != 1)
                {
                    return Err(err("F007_UNSUPPORTED_FEATURE_TRANSFORM"));
                }
                feature_count = 0;
                entered_features = true;
            }
            "Step/LayerFeature/Set/Features/Line" => {
                if !pending_points.is_empty() || tag.attrs.len() != 4 {
                    return Err(err("F007_UNSUPPORTED_LINE"));
                }
                let a = [
                    f007_nm(field(&tag.attrs, "startX")?, factor)?,
                    f007_nm(field(&tag.attrs, "startY")?, factor)?,
                ];
                let b = [
                    f007_nm(field(&tag.attrs, "endX")?, factor)?,
                    f007_nm(field(&tag.attrs, "endY")?, factor)?,
                ];
                if !entered_features || feature_count != 0 {
                    return Err(err("F007_DUPLICATE_FEATURE"));
                }
                feature_count += 1;
                pending_points = vec![a, b];
                current_kind = "Line";
            }
            "Step/LayerFeature/Set/Features/Polyline" => {
                if !pending_points.is_empty() || !tag.attrs.is_empty() {
                    return Err(err("F007_UNSUPPORTED_POLYLINE"));
                }
                if !entered_features || feature_count != 0 {
                    return Err(err("F007_DUPLICATE_FEATURE"));
                }
                feature_count += 1;
                poly_begin_seen = false;
                current_kind = "Polyline";
            }
            "Step/LayerFeature/Set/Features/Polyline/PolyBegin"
            | "Step/LayerFeature/Set/Features/Polyline/PolyStepSegment" => {
                let is_begin = tag.path.ends_with("/PolyBegin");
                if current_kind != "Polyline"
                    || tag.attrs.len() != 2
                    || (is_begin && poly_begin_seen)
                    || (!is_begin && !poly_begin_seen)
                {
                    return Err(err("F007_UNSUPPORTED_POLYLINE_VERTEX"));
                }
                if is_begin {
                    poly_begin_seen = true;
                }
                if pending_points.len() >= 128 {
                    return Err(err("F007_POLYLINE_VERTEX_LIMIT"));
                }
                pending_points.push([
                    f007_nm(field(&tag.attrs, "x")?, factor)?,
                    f007_nm(field(&tag.attrs, "y")?, factor)?,
                ]);
            }
            "Step/LayerFeature/Set/Features/Line/LineDesc"
            | "Step/LayerFeature/Set/Features/Polyline/LineDesc" => {
                if current_kind.is_empty()
                    || pending_points.len() < 2
                    || tag
                        .attrs
                        .keys()
                        .any(|k| !matches!(k.as_str(), "lineWidth" | "lineEnd" | "lineProperty"))
                    || field(&tag.attrs, "lineEnd")? != "ROUND"
                    || tag.attrs.get("lineProperty").is_some_and(|v| v != "SOLID")
                {
                    return Err(err("F007_UNSUPPORTED_LINE_DESCRIPTION"));
                }
                if pending_points.windows(2).any(|w| w[0] == w[1]) {
                    return Err(err("F007_ZERO_LENGTH_SEGMENT"));
                }
                let width = f007_nm(field(&tag.attrs, "lineWidth")?, factor)?;
                if width <= 0 || width > 1_000_000_000 {
                    return Err(err("F007_BAD_WIDTH"));
                }
                preview_segments = preview_segments
                    .checked_add(pending_points.len() - 1)
                    .ok_or_else(|| err("F007_QT_PREVIEW_LIMIT"))?;
                if traces >= 128 || preview_segments > 128 {
                    return Err(err("F007_QT_PREVIEW_LIMIT"));
                }
                let mut meta = object(
                    &source,
                    ObjectKind::Trace,
                    &format!(
                        "{}/LayerFeature/{}/Trace/{traces:05}",
                        inventory.selected_step,
                        layer
                            .as_ref()
                            .ok_or_else(|| err("F007_MISSING_LAYER"))?
                            .source_object,
                    ),
                );
                meta.provenance.source_object_type =
                    format!("IPC-2581C {current_kind} / inline ROUND SOLID LineDesc");
                meta.provenance.history[0].operation = "exact-source-decimal-to-integer-nm".into();
                board.traces.push(Trace {
                    meta,
                    layer: layer.clone().ok_or_else(|| err("F007_MISSING_LAYER"))?,
                    net: net.clone(),
                    centreline: pending_points
                        .drain(..)
                        .map(|ticks| Point {
                            frame: frame_id.clone(),
                            ticks,
                        })
                        .collect(),
                    width_m: Rational::new(i128::from(width), 1_000_000_000)
                        .map_err(|_| err("F007_BAD_WIDTH"))?,
                });
                traces += 1;
                current_kind = "";
            }
            _ => return Err(err("F007_UNSUPPORTED_CADDATA_GEOMETRY")),
        }
        // A LineDesc may only terminate the exact currently active Feature.
        // Additional unexpected children are rejected by the match above.
    }
    if !pending_points.is_empty()
        || !current_kind.is_empty()
        || board.traces.is_empty()
        || (entered_features && feature_count != 1)
    {
        return Err(err("F007_INCOMPLETE_OR_EMPTY_GEOMETRY"));
    }
    if board.validate().has_errors() {
        return Err(err("F007_INVALID_CANONICAL"));
    }
    Ok(board)
}

#[cfg(test)]
mod f007_native_tests {
    use super::*;

    fn fixture(geometry: &str, units: &str) -> String {
        format!(
            r#"<?xml version="1.0"?>
<IPC-2581 revision="C" xmlns="http://webstds.ipc.org/2581">
<Content><LayerRef name="TOP"/><LayerRef name="BOTTOM"/></Content>
<LogisticHeader/><HistoryRecord/>
<Ecad><CadHeader units="{units}"/><CadData>
<Layer name="TOP" layerFunction="SIGNAL" side="TOP" polarity="POSITIVE"/>
<Layer name="BOTTOM" layerFunction="SIGNAL" side="BOTTOM" polarity="POSITIVE"/>
<Step name="BOARD"><LogicalNet name="SIG"><PinRef componentRef="U1" pin="1"/></LogicalNet>
<LayerFeature layerRef="TOP"><Set net="SIG">{geometry}</Set></LayerFeature>
</Step></CadData></Ecad></IPC-2581>"#
        )
    }
    #[test]
    fn xsd_inline_line_and_polyline_exact_nm() {
        let geometry = r#"<Features><Line startX="1.123456" startY="-2"
            endX="3" endY="5.000001"><LineDesc lineEnd="ROUND" lineWidth="0.15"
            lineProperty="SOLID"/></Line></Features>
            <Features><Polyline><PolyBegin x="3" y="5.000001"/>
            <PolyStepSegment x="4" y="6"/><PolyStepSegment x="5" y="6"/>
            <LineDesc lineEnd="ROUND" lineWidth="0.2"/></Polyline></Features>"#;
        let input = fixture(geometry, "MILLIMETER");
        let model = into_native_canonical(input.as_bytes()).unwrap();
        assert_eq!(model.traces.len(), 2);
        assert_eq!(model.traces[0].centreline[0].ticks, [1_123_456, -2_000_000]);
        assert_eq!(model.traces[0].centreline[1].ticks, [3_000_000, 5_000_001]);
        assert_eq!(model.traces[1].centreline.len(), 3);
        assert_eq!(model.traces[0].width_m, Rational::new(15, 100_000).unwrap());
        assert!(!model.validate().has_errors());
        assert_eq!(
            model.sources[0].id.files[0].sha256,
            crate::digest::sha256(input.as_bytes())
        );
        assert_eq!(model.sources[0].capability.model, CapabilityState::Missing);
    }
    #[test]
    fn units_and_unsupported_geometry_are_closed() {
        let line = r#"<Features><Line startX="1" startY="2" endX="3" endY="4">
            <LineDesc lineEnd="ROUND" lineWidth="0.1"/></Line></Features>"#;
        let mm = into_native_canonical(fixture(line, "MILLIMETER").as_bytes()).unwrap();
        let micron = into_native_canonical(fixture(line, "MICRON").as_bytes()).unwrap();
        let inch = into_native_canonical(fixture(line, "INCH").as_bytes()).unwrap();
        assert_eq!(mm.traces[0].centreline[0].ticks[0], 1_000_000);
        assert_eq!(micron.traces[0].centreline[0].ticks[0], 1_000);
        assert_eq!(inch.traces[0].centreline[0].ticks[0], 25_400_000);
        for evil in [
            line.replace("lineEnd=\"ROUND\"", "lineEnd=\"SQUARE\""),
            line.replace("<LineDesc", "<LineDescRef"),
            line.replace("startX=\"1\"", "startX=\"1.0000000001\""),
            format!("{line}<Pad padstackDefRef=\"P\"/>"),
            format!("{line}<Features><Arc/></Features>"),
        ] {
            assert!(into_native_canonical(fixture(&evil, "MILLIMETER").as_bytes()).is_err());
        }
        assert!(into_native_canonical(fixture(line, "MM").as_bytes()).is_err());
    }
}
