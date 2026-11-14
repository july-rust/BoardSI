//! E_026: opt-in KiCad 9 (20241229) SOURCE-only layer and via inventory.
//! No BoardModel, manufactured barrel, connected copper, Qt or SI is produced.
use boardsi_model::{
    CapabilityState, ConversionQuality, ConversionStep, GeometryProvenance, ImportCapability,
    LayerKind, ObjectId, ObjectKind, ObjectMeta, Rational, SourceFile, SourceFormat, SourceId,
    SourceIdentity, ViaKind, ViaSpan,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub const SUPPORTED_PCB_VERSION: &str = "20241229";
const MAX_INPUT: usize = 8 * 1024 * 1024;
const MAX_TOKENS: usize = 250_000;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 64;
const MAX_ATOM: usize = 8192;
const MAX_VIAS: usize = 20_000;

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
    pub ordinal: u32,
    pub canonical_name: String,
    pub user_name: Option<String>,
    pub layer_type: String,
    pub kind: LayerKind,
    /// Source-backed copper order, F.Cu=0, In1...In30, B.Cu=last.
    /// This is not z, thickness or verified physical connectivity.
    pub copper_order: Option<usize>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceVia {
    pub meta: ObjectMeta,
    pub source_offset: usize,
    pub source_uuid: Option<String>,
    pub kind: ViaKind,
    /// Exact source decimal millimetres converted to integer nanometres.
    pub centre_nm: [i64; 2],
    pub diameter_nm: i64,
    pub drill_nm: i64,
    pub source_first_layer: String,
    pub source_last_layer: String,
    pub span: ViaSpan,
    /// A layer-envelope witness, NOT a conductive contact set.
    pub envelope_layers: Vec<ObjectId>,
    pub net_ordinal: u32,
    pub remove_unused_layers: bool,
    pub keep_end_layers: bool,
    /// KiCad source UI flags; neither conveys electrical connectivity.
    pub locked: bool,
    pub free: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Inventory {
    pub identity: SourceId,
    pub layers: Vec<SourceLayer>,
    pub vias: Vec<SourceVia>,
    pub capability: ImportCapability,
    pub solver_ready: bool,
}
impl Inventory {
    pub fn diagnostic(&self) -> String {
        format!(
            "format=KiCad;version={};scope=SOURCE_PARTIAL;layers={};vias={};solver_ready=false;physical_si=NOT_RUN",
            SUPPORTED_PCB_VERSION, self.layers.len(), self.vias.len()
        )
    }
}

#[derive(Clone, Debug)]
enum Token {
    Open(usize),
    Close,
    Text(String),
}
#[derive(Clone, Debug)]
enum Item {
    Text(String),
    Node(Node),
}
#[derive(Clone, Debug)]
struct Node {
    offset: usize,
    items: Vec<Item>,
}
impl Node {
    fn head(&self) -> Result<&str, ImportError> {
        match self.items.first() {
            Some(Item::Text(s)) => Ok(s),
            _ => Err(err("BAD_SEXPR_HEAD")),
        }
    }
    fn words(&self) -> Result<Vec<&str>, ImportError> {
        self.items
            .iter()
            .map(|it| match it {
                Item::Text(s) => Ok(s.as_str()),
                Item::Node(_) => Err(err("UNEXPECTED_NESTED_ITEM")),
            })
            .collect()
    }
    fn children(&self) -> impl Iterator<Item = &Node> {
        self.items.iter().filter_map(|it| match it {
            Item::Node(n) => Some(n),
            Item::Text(_) => None,
        })
    }
    fn child(&self, name: &str) -> Result<Option<&Node>, ImportError> {
        let mut result = None;
        for node in self.children() {
            if node.head()? == name && result.replace(node).is_some() {
                return Err(err("DUPLICATE_FIELD"));
            }
        }
        Ok(result)
    }
    fn required(&self, name: &str) -> Result<&Node, ImportError> {
        self.child(name)?.ok_or_else(|| err("MISSING_FIELD"))
    }
    fn scalar(&self, name: &str) -> Result<&str, ImportError> {
        let words = self.required(name)?.words()?;
        if words.len() != 2 {
            return Err(err("BAD_SCALAR"));
        }
        // This lifetime is tied to the Node rather than the temporary Vec.
        if let Some(Item::Text(ref value)) = self.required(name)?.items.get(1) {
            Ok(value)
        } else {
            Err(err("BAD_SCALAR"))
        }
    }
}

fn tokenize(source: &str) -> Result<Vec<Token>, ImportError> {
    let b = source.as_bytes();
    let mut i = 0;
    let mut tokens = Vec::new();
    while i < b.len() {
        let current = b[i];
        if current.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if tokens.len() >= MAX_TOKENS {
            return Err(err("TOKEN_LIMIT"));
        }
        match current {
            b'(' => {
                tokens.push(Token::Open(i));
                i += 1;
            }
            b')' => {
                tokens.push(Token::Close);
                i += 1;
            }
            b'"' => {
                i += 1;
                let mut value = String::new();
                let mut start = i;
                loop {
                    if i >= b.len() {
                        return Err(err("UNCLOSED_STRING"));
                    }
                    if b[i] == b'"' {
                        value.push_str(&source[start..i]);
                        i += 1;
                        break;
                    }
                    if b[i] == b'\\' {
                        value.push_str(&source[start..i]);
                        i += 1;
                        let c = *b.get(i).ok_or_else(|| err("BAD_ESCAPE"))?;
                        value.push(match c {
                            b'\\' => '\\',
                            b'"' => '"',
                            b'n' => '\n',
                            b'r' => '\r',
                            b't' => '\t',
                            _ => return Err(err("BAD_ESCAPE")),
                        });
                        i += 1;
                        start = i;
                    } else {
                        if b[i] < 0x20 {
                            return Err(err("CONTROL_IN_STRING"));
                        }
                        i += 1;
                    }
                    if i - start > MAX_ATOM || value.len() > MAX_ATOM {
                        return Err(err("STRING_LIMIT"));
                    }
                }
                if value.len() > MAX_ATOM {
                    return Err(err("STRING_LIMIT"));
                }
                tokens.push(Token::Text(value));
            }
            b';' => return Err(err("COMMENT_UNSUPPORTED")),
            b'\\' | b'\'' => return Err(err("UNEXPECTED_TOKEN")),
            _ => {
                let start = i;
                while i < b.len()
                    && !b[i].is_ascii_whitespace()
                    && !matches!(b[i], b'(' | b')' | b'"')
                {
                    if b[i] < 0x20 || b[i] == b';' || b[i] == b'\\' {
                        return Err(err("BAD_ATOM"));
                    }
                    i += 1;
                    if i - start > MAX_ATOM {
                        return Err(err("ATOM_LIMIT"));
                    }
                }
                if i == start {
                    return Err(err("BAD_ATOM"));
                }
                tokens.push(Token::Text(source[start..i].to_owned()));
            }
        }
    }
    Ok(tokens)
}
fn parse_node(
    t: &[Token],
    pos: &mut usize,
    nodes: &mut usize,
    depth: usize,
) -> Result<Node, ImportError> {
    if depth > MAX_DEPTH {
        return Err(err("DEPTH_LIMIT"));
    }
    let offset = match t.get(*pos) {
        Some(Token::Open(offset)) => *offset,
        _ => return Err(err("EXPECTED_OPEN")),
    };
    *pos += 1;
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(err("NODE_LIMIT"));
    }
    let mut items = Vec::new();
    loop {
        match t.get(*pos) {
            Some(Token::Close) => {
                *pos += 1;
                break;
            }
            Some(Token::Open(_)) => items.push(Item::Node(parse_node(t, pos, nodes, depth + 1)?)),
            Some(Token::Text(value)) => {
                items.push(Item::Text(value.clone()));
                *pos += 1;
            }
            None => return Err(err("UNCLOSED_SEXPR")),
        }
    }
    let node = Node { offset, items };
    node.head()?;
    Ok(node)
}
fn parse(source: &str) -> Result<Node, ImportError> {
    let tokens = tokenize(source)?;
    let mut pos = 0;
    let mut nodes = 0;
    let root = parse_node(&tokens, &mut pos, &mut nodes, 1)?;
    if pos != tokens.len() {
        return Err(err("TRAILING_TOKENS"));
    }
    Ok(root)
}

/// This is the KiCad v9 canonical copper identity, NOT numerical sort of
/// source ORDINAL values: 0=F.Cu, 2=B.Cu, 4=In1.Cu, 6=In2.Cu ...
fn copper_identity(name: &str) -> Result<Option<(usize, u32)>, ImportError> {
    if name == "F.Cu" {
        return Ok(Some((0, 0)));
    }
    if name == "B.Cu" {
        return Ok(Some((usize::MAX, 2)));
    }
    if let Some(num) = name.strip_prefix("In").and_then(|s| s.strip_suffix(".Cu")) {
        if num.is_empty() || num.starts_with('0') || !num.bytes().all(|b| b.is_ascii_digit()) {
            return Err(err("UNKNOWN_COPPER_LAYER"));
        }
        let n = num
            .parse::<usize>()
            .map_err(|_| err("UNKNOWN_COPPER_LAYER"))?;
        if !(1..=30).contains(&n) {
            return Err(err("UNKNOWN_COPPER_LAYER"));
        }
        return Ok(Some((n, 2 * n as u32 + 2)));
    }
    if name.ends_with(".Cu") {
        return Err(err("UNKNOWN_COPPER_LAYER"));
    }
    Ok(None)
}

fn millimetres_to_nm(raw: &str) -> Result<i64, ImportError> {
    if raw.is_empty() || raw.len() > 40 {
        return Err(err("BAD_DECIMAL"));
    }
    let (neg, value) = if let Some(v) = raw.strip_prefix('-') {
        (true, v)
    } else {
        (false, raw)
    };
    let (integer, fraction) = value.split_once('.').unwrap_or((value, ""));
    if integer.is_empty()
        || !integer.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > 6
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || (value.contains('.') && fraction.is_empty())
    {
        return Err(err("BAD_DECIMAL"));
    }
    let int = integer
        .parse::<i64>()
        .map_err(|_| err("COORDINATE_OVERFLOW"))?;
    let frac = if fraction.is_empty() {
        0
    } else {
        let n = fraction.parse::<i64>().map_err(|_| err("BAD_DECIMAL"))?;
        n * 10_i64.pow((6 - fraction.len()) as u32)
    };
    let num = int
        .checked_mul(1_000_000)
        .and_then(|v| v.checked_add(frac))
        .ok_or_else(|| err("COORDINATE_OVERFLOW"))?;
    if neg {
        num.checked_neg().ok_or_else(|| err("COORDINATE_OVERFLOW"))
    } else {
        Ok(num)
    }
}
fn positive_mm(n: &str) -> Result<i64, ImportError> {
    let value = millimetres_to_nm(n)?;
    if value <= 0 {
        return Err(err("INVALID_DIMENSION"));
    }
    Ok(value)
}
fn uuid_valid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
fn meta(identity: &SourceId, kind: ObjectKind, id: &str, source_type: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(identity, kind, id, ""),
        provenance: GeometryProvenance {
            source_object_type: source_type.into(),
            history: vec![ConversionStep {
                operation: "preserve-kicad9-source".into(),
                quality: ConversionQuality::Exact,
                max_error_m: Some(Rational::integer(0)),
                note: "Source declaration only; no pad conductivity, plating or solver proof"
                    .into(),
            }],
        },
    }
}
fn checked_field<'a>(node: &'a Node, key: &str) -> Result<&'a Node, ImportError> {
    node.required(key)
}
fn arguments<'a>(n: &'a Node, key: &str, count: usize) -> Result<Vec<&'a str>, ImportError> {
    let words = checked_field(n, key)?.words()?;
    if words.len() != count + 1 {
        return Err(err("BAD_FIELD_ARITY"));
    }
    Ok(words.into_iter().skip(1).collect())
}

/// KiCad 9 permits legacy presence-only flags and explicit (flag yes/no).
/// Keep the value (not merely field presence); malformed values fail closed.
fn via_flag(node: &Node, key: &str) -> Result<bool, ImportError> {
    let Some(field) = node.child(key)? else {
        return Ok(false);
    };
    let words = field.words()?;
    match words.as_slice() {
        [_] | [_, "yes"] => Ok(true),
        [_, "no"] => Ok(false),
        _ => Err(err("BAD_VIA_FLAG")),
    }
}

pub fn inspect(bytes: &[u8]) -> Result<Inventory, ImportError> {
    if bytes.len() > MAX_INPUT {
        return Err(err("INPUT_TOO_LARGE"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| err("INVALID_UTF8"))?;
    let root = parse(text)?;
    if root.head()? != "kicad_pcb" {
        return Err(err("BAD_ROOT"));
    }
    if root.scalar("version")? != SUPPORTED_PCB_VERSION {
        return Err(err("UNSUPPORTED_VERSION"));
    }
    if root.scalar("generator")?.is_empty() {
        return Err(err("MISSING_GENERATOR"));
    }
    let layer_node = root.required("layers")?;
    // Layer elements use a numeric head, not a "layer" name.
    let mut defined = Vec::<(u32, String, String, Option<String>, Option<usize>, usize)>::new();
    let mut ordinals = BTreeSet::new();
    let mut names = BTreeSet::new();
    for item in layer_node.children() {
        if defined.len() >= 60 {
            return Err(err("LAYER_LIMIT"));
        }
        let words = item.words()?;
        if !(3..=4).contains(&words.len()) {
            return Err(err("BAD_LAYER_DECLARATION"));
        }
        let ordinal = words[0]
            .parse::<u32>()
            .map_err(|_| err("BAD_LAYER_ORDINAL"))?;
        let name = words[1];
        let typ = words[2];
        if name.is_empty()
            || name.len() > 128
            || !ordinals.insert(ordinal)
            || !names.insert(name.to_owned())
        {
            return Err(err("DUPLICATE_OR_INVALID_LAYER"));
        }
        let copper = copper_identity(name)?;
        let order = if let Some((index, expected)) = copper {
            if expected != ordinal || !matches!(typ, "signal" | "power" | "mixed" | "jumper") {
                return Err(err("COPPER_DECLARATION_MISMATCH"));
            }
            Some(index)
        } else {
            if typ != "user" {
                return Err(err("UNSUPPORTED_TECH_LAYER_TYPE"));
            }
            None
        };
        defined.push((
            ordinal,
            name.into(),
            typ.into(),
            words.get(3).map(|s| (*s).to_string()),
            order,
            item.offset,
        ));
    }
    let count = defined.iter().filter(|x| x.4.is_some()).count();
    if !(2..=32).contains(&count)
        || count % 2 != 0
        || !names.contains("F.Cu")
        || !names.contains("B.Cu")
    {
        return Err(err("INVALID_COPPER_COUNT"));
    }
    let mut expected_order = BTreeSet::new();
    for entry in &mut defined {
        if let Some(order) = entry.4.as_mut() {
            if *order == usize::MAX {
                *order = count - 1;
            }
            if *order >= count || !expected_order.insert(*order) {
                return Err(err("INVALID_COPPER_ORDER"));
            }
        }
    }
    if expected_order.len() != count || expected_order.iter().copied().ne(0..count) {
        return Err(err("MISSING_COPPER_LAYER"));
    }
    let source = Arc::new(SourceIdentity {
        format: SourceFormat::KiCad,
        format_version: SUPPORTED_PCB_VERSION.into(),
        files: vec![SourceFile {
            logical_path: "input.kicad_pcb".into(),
            sha256: crate::digest::sha256(bytes),
        }],
        importer_version: env!("CARGO_PKG_VERSION").into(),
        conditions: vec![("scope".into(), "source-layer-via-only".into())],
    });
    let mut layers: Vec<SourceLayer> = defined
        .into_iter()
        .map(|(ordinal, name, typ, user_name, order, offset)| {
            let kind = if order.is_some() {
                LayerKind::Copper
            } else if name.ends_with(".Mask") {
                LayerKind::Mask
            } else {
                LayerKind::Other
            };
            SourceLayer {
                meta: meta(
                    &source,
                    ObjectKind::Layer,
                    &format!("layers/{ordinal}/{name}"),
                    "KiCad9 layer declaration",
                ),
                ordinal,
                canonical_name: name,
                user_name,
                layer_type: typ,
                kind,
                copper_order: order,
                source_offset: offset,
            }
        })
        .collect();
    // Maintain the original layer declaration order. Only this mapping is ordered.
    let mut by_name = BTreeMap::<String, (usize, ObjectId)>::new();
    let mut by_order = BTreeMap::<usize, ObjectId>::new();
    for l in &layers {
        if let Some(o) = l.copper_order {
            by_order.insert(o, l.meta.id.clone());
            by_name.insert(l.canonical_name.clone(), (o, l.meta.id.clone()));
        }
    }
    let mut nets = BTreeSet::<u32>::new();
    let mut net_names = BTreeSet::<String>::new();
    for node in root.children().filter(|n| n.head().ok() == Some("net")) {
        let words = node.words()?;
        if words.len() != 3 {
            return Err(err("BAD_NET_DECLARATION"));
        }
        let number = words[1]
            .parse::<u32>()
            .map_err(|_| err("BAD_NET_DECLARATION"))?;
        if !nets.insert(number) || !net_names.insert(words[2].into()) {
            return Err(err("DUPLICATE_NET"));
        }
    }
    let mut seen_uuids = BTreeSet::new();
    let mut vias = Vec::new();
    for node in root.children().filter(|n| n.head().ok() == Some("via")) {
        if vias.len() >= MAX_VIAS {
            return Err(err("VIA_LIMIT"));
        }
        let mut declared_kind = "through";
        let mut locked = false;
        for item in node.items.iter().skip(1) {
            match item {
                Item::Text(s) if s == "blind" || s == "micro" => {
                    if declared_kind != "through" {
                        return Err(err("DUPLICATE_VIA_TYPE"));
                    }
                    declared_kind = s;
                }
                Item::Text(s) if s == "locked" => {
                    if locked {
                        return Err(err("DUPLICATE_VIA_FLAG"));
                    }
                    locked = true;
                }
                Item::Text(_) => return Err(err("UNSUPPORTED_VIA_ATOM")),
                Item::Node(_) => (),
            }
        }
        let mut child_keys = BTreeSet::new();
        for child in node.children() {
            let key = child.head()?;
            if !matches!(
                key,
                "at" | "size"
                    | "drill"
                    | "layers"
                    | "net"
                    | "uuid"
                    | "tstamp"
                    | "remove_unused_layers"
                    | "keep_end_layers"
                    | "free"
                    | "locked"
            ) {
                return Err(err("UNSUPPORTED_VIA_FIELD"));
            }
            if !child_keys.insert(key.to_owned()) {
                return Err(err("DUPLICATE_VIA_FIELD"));
            }
        }
        if locked && child_keys.contains("locked") {
            return Err(err("DUPLICATE_VIA_FLAG"));
        }
        let remove_unused = via_flag(node, "remove_unused_layers")?;
        let keep_ends = via_flag(node, "keep_end_layers")?;
        let free = via_flag(node, "free")?;
        let locked = locked || via_flag(node, "locked")?;
        if keep_ends && !remove_unused {
            return Err(err("INVALID_KEEP_END_LAYERS"));
        }
        let at = arguments(node, "at", 2)?;
        let diameter = positive_mm(node.scalar("size")?)?;
        let drill = positive_mm(node.scalar("drill")?)?;
        if drill >= diameter {
            return Err(err("DRILL_EXCEEDS_PAD"));
        }
        let names_field = arguments(node, "layers", 2)?;
        let first_name = names_field[0];
        let last_name = names_field[1];
        let (first, first_id) = by_name
            .get(first_name)
            .ok_or_else(|| err("UNKNOWN_VIA_LAYER"))?;
        let (last, last_id) = by_name
            .get(last_name)
            .ok_or_else(|| err("UNKNOWN_VIA_LAYER"))?;
        if first >= last {
            return Err(err("REVERSED_OR_ZERO_SPAN"));
        }
        let outer_first = *first == 0;
        let outer_last = *last == count - 1;
        let kind = match declared_kind {
            "through" if outer_first && outer_last => ViaKind::Through,
            "through" => return Err(err("THROUGH_SPAN_MISMATCH")),
            "blind" if !outer_first && !outer_last => ViaKind::Buried,
            "blind" if outer_first ^ outer_last => ViaKind::Blind,
            "blind" => return Err(err("BLIND_SPAN_MISMATCH")),
            "micro" if (*last - *first) == 1 && (outer_first || outer_last) => ViaKind::Microvia,
            "micro" => return Err(err("MICROVIA_SPAN_MISMATCH")),
            _ => return Err(err("BAD_VIA_TYPE")),
        };
        let net = node
            .scalar("net")?
            .parse::<u32>()
            .map_err(|_| err("INVALID_VIA_NET"))?;
        if net != 0 && !nets.contains(&net) {
            return Err(err("UNKNOWN_VIA_NET"));
        }
        let uuid: Option<String> = match (node.child("uuid")?, node.child("tstamp")?) {
            (Some(_), Some(_)) => return Err(err("AMBIGUOUS_VIA_ID")),
            (Some(n), None) => {
                let words = n.words()?;
                if words.len() != 2 || !uuid_valid(words[1]) {
                    return Err(err("INVALID_UUID"));
                }
                Some(words[1].into())
            }
            (None, Some(n)) => {
                let words = n.words()?;
                if words.len() != 2
                    || words[1].is_empty()
                    || words[1].len() > 64
                    || !words[1].bytes().all(|c| c.is_ascii_hexdigit())
                {
                    return Err(err("INVALID_TSTAMP"));
                }
                Some(words[1].into())
            }
            (None, None) => None,
        };
        if let Some(ref value) = uuid {
            if !seen_uuids.insert(value.to_ascii_lowercase()) {
                return Err(err("DUPLICATE_UUID"));
            }
        }
        let id = match &uuid {
            Some(v) => format!("via/{v}"),
            None => format!("via@byte:{}", node.offset),
        };
        let envelope = (*first..=*last)
            .map(|o| {
                by_order
                    .get(&o)
                    .cloned()
                    .ok_or_else(|| err("MISSING_ENVELOPE_LAYER"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        vias.push(SourceVia {
            meta: meta(&source, ObjectKind::Via, &id, "KiCad9 via declaration"),
            source_offset: node.offset,
            source_uuid: uuid,
            kind,
            centre_nm: [millimetres_to_nm(at[0])?, millimetres_to_nm(at[1])?],
            diameter_nm: diameter,
            drill_nm: drill,
            source_first_layer: first_name.into(),
            source_last_layer: last_name.into(),
            span: ViaSpan {
                first_layer: first_id.clone(),
                last_layer: last_id.clone(),
            },
            envelope_layers: envelope,
            net_ordinal: net,
            remove_unused_layers: remove_unused,
            keep_end_layers: keep_ends,
            locked,
            free,
        });
    }
    Ok(Inventory {
        identity: source,
        layers: std::mem::take(&mut layers),
        vias,
        capability: ImportCapability {
            geometry: CapabilityState::Partial,
            net: if nets.is_empty() {
                CapabilityState::Missing
            } else {
                CapabilityState::Partial
            },
            pin_pad: CapabilityState::Missing,
            stackup: CapabilityState::Partial,
            reference: CapabilityState::Missing,
            model: CapabilityState::Missing,
            object_coverage: vec![
                "KiCad9 layer declaration".into(),
                "KiCad9 track via source".into(),
            ],
        },
        solver_ready: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn board(n: usize, vias: &str) -> String {
        let mut layers = String::from(r#"(0 "F.Cu" signal) (2 "B.Cu" signal) (1 "F.Mask" user)"#);
        for i in 1..n - 1 {
            layers.push_str(&format!(r#"( {} "In{}.Cu" signal)"#, 2 * i + 2, i));
        }
        format!(
            r#"(kicad_pcb (version 20241229) (generator "boardsi-e026-test")
              (general (thickness 1.6)) (paper "A4")
              (layers {layers}) (setup) (net 0 "") (net 1 "GND") {vias})"#
        )
    }
    fn via(kind: &str, start: &str, end: &str) -> String {
        format!(
            r#"(via {kind} (at 12.345678 -3.000001) (size 0.6) (drill 0.3)
                       (layers "{start}" "{end}") (net 1)
                       (uuid "123e4567-e89b-12d3-a456-426614174000"))"#
        )
    }
    #[test]
    fn two_through_and_order_not_ordinal_sort() {
        for n in [2, 4, 6, 8, 32] {
            let file = board(n, &via("", "F.Cu", "B.Cu"));
            let inv = inspect(file.as_bytes()).unwrap();
            assert_eq!(inv.vias.len(), 1);
            assert_eq!(inv.vias[0].kind, ViaKind::Through);
            assert_eq!(inv.vias[0].envelope_layers.len(), n);
            assert_eq!(inv.vias[0].centre_nm, [12_345_678, -3_000_001]);
            assert_eq!(inv.vias[0].diameter_nm, 600_000);
            assert_eq!(inv.vias[0].drill_nm, 300_000);
            assert!(!inv.solver_ready);
            assert_eq!(
                inv.layers
                    .iter()
                    .find(|l| l.canonical_name == "B.Cu")
                    .unwrap()
                    .ordinal,
                2
            );
        }
    }
    #[test]
    fn blind_buried_and_micro_envelopes() {
        let tests = [
            (4, "blind", "F.Cu", "In1.Cu", ViaKind::Blind, 2),
            (4, "blind", "In1.Cu", "In2.Cu", ViaKind::Buried, 2),
            (6, "blind", "In2.Cu", "In3.Cu", ViaKind::Buried, 2),
            (6, "micro", "F.Cu", "In1.Cu", ViaKind::Microvia, 2),
            (8, "blind", "F.Cu", "In5.Cu", ViaKind::Blind, 6),
        ];
        for (count, token, a, b, kind, envelope) in tests {
            let inv = inspect(board(count, &via(token, a, b)).as_bytes()).unwrap();
            assert_eq!(inv.vias[0].kind, kind);
            assert_eq!(inv.vias[0].envelope_layers.len(), envelope);
        }
    }
    #[test]
    fn rejection_mutations_are_closed() {
        let good = board(4, &via("", "F.Cu", "B.Cu"));
        let bad = [
            good.replace("20241229", "20260206"),
            good.replace("(layers", "(oops"),
            good.replace(r#"(2 "B.Cu" signal)"#, r#"(0 "B.Cu" signal)"#),
            good.replace(r#"(2 "B.Cu" signal)"#, ""),
            good.replace(r#""B.Cu") (net"#, r#""None.Cu") (net"#),
            good.replace("(drill 0.3)", "(drill 0.7)"),
            good.replace("(at 12.345678", "(at 12.3456789"),
            good.replace("(net 1)", "(net 999)"),
            good.replace(" (generator", " (unknown"),
            good.replace("(uuid ", "(extra "),
            good.replace("(via ", "(via micro "),
            good.replace("(version 20241229)", "(version 20241229"),
            good.clone() + ")",
        ];
        for bad_input in bad {
            assert!(inspect(bad_input.as_bytes()).is_err(), "mutation accepted");
        }
    }
    #[test]
    fn uuid_duplicate_and_source_hash_invalidation() {
        let v = via("", "F.Cu", "B.Cu");
        let bad = board(4, &format!("{v}{v}"));
        assert_eq!(inspect(bad.as_bytes()).unwrap_err().code, "DUPLICATE_UUID");
        let one = inspect(board(4, &v).as_bytes()).unwrap();
        let another = inspect(board(4, &v.replace("12.345678", "12.345677")).as_bytes()).unwrap();
        assert_ne!(
            one.identity.files[0].sha256,
            another.identity.files[0].sha256
        );
        assert_ne!(one.vias[0].meta.id, another.vias[0].meta.id);
    }
    #[test]
    fn malformed_and_resource_caps() {
        assert!(inspect(&[0xff]).is_err());
        assert_eq!(
            inspect(&vec![b'X'; MAX_INPUT + 1]).unwrap_err().code,
            "INPUT_TOO_LARGE"
        );
        let nested = format!("{}X{}", "(".repeat(70), ")".repeat(70));
        assert_eq!(inspect(nested.as_bytes()).unwrap_err().code, "DEPTH_LIMIT");
        assert!(inspect(br#"(kicad_pcb (version 20241229) (layers (0 "F.Cu" signal) "#).is_err());
    }
    #[test]
    fn explicit_kicad9_boolean_via_flags_are_preserved() {
        let original = via("", "F.Cu", "B.Cu");
        let yes = original.replace(
            "(net 1)",
            "(locked yes) (free yes) (remove_unused_layers yes) (keep_end_layers yes) (net 1)",
        );
        let yes_inventory = inspect(board(2, &yes).as_bytes()).unwrap();
        let flags = &yes_inventory.vias[0];
        assert!(flags.locked);
        assert!(flags.free);
        assert!(flags.remove_unused_layers);
        assert!(flags.keep_end_layers);

        let no = original.replace(
            "(net 1)",
            "(locked no) (free no) (remove_unused_layers no) (keep_end_layers no) (net 1)",
        );
        let no_inventory = inspect(board(2, &no).as_bytes()).unwrap();
        let flags = &no_inventory.vias[0];
        assert!(!flags.locked);
        assert!(!flags.free);
        assert!(!flags.remove_unused_layers);
        assert!(!flags.keep_end_layers);

        for invalid in [
            original.replace("(net 1)", "(locked maybe) (net 1)"),
            original.replace("(net 1)", "(free true) (net 1)"),
            original.replace("(net 1)", "(remove_unused_layers yes no) (net 1)"),
            original.replace("(net 1)", "(keep_end_layers 1) (net 1)"),
            original.replace("(net 1)", "(locked yes) (locked no) (net 1)"),
            original.replace("(net 1)", "(keep_end_layers yes) (net 1)"),
        ] {
            assert!(inspect(board(2, &invalid).as_bytes()).is_err());
        }
        // An obsolete positional locked atom and modern child may not coexist.
        let mixed = original
            .replacen("(via ", "(via locked ", 1)
            .replace("(net 1)", "(locked no) (net 1)");
        assert_eq!(
            inspect(board(2, &mixed).as_bytes()).unwrap_err().code,
            "DUPLICATE_VIA_FLAG"
        );
    }

    #[test]
    fn flags_and_unsupported_field_are_safe() {
        let v = via("", "F.Cu", "B.Cu");
        let flagged = v.replace(
            "(net 1)",
            "(remove_unused_layers) (keep_end_layers) (net 1)",
        );
        let inv = inspect(board(2, &flagged).as_bytes()).unwrap();
        assert!(inv.vias[0].remove_unused_layers);
        assert!(inv.vias[0].keep_end_layers);
        let invalid = v.replace("(net 1)", "(keep_end_layers) (net 1)");
        assert_eq!(
            inspect(board(2, &invalid).as_bytes()).unwrap_err().code,
            "INVALID_KEEP_END_LAYERS"
        );
        let unknown = v.replace("(net 1)", "(backdrill 0.2) (net 1)");
        assert_eq!(
            inspect(board(2, &unknown).as_bytes()).unwrap_err().code,
            "UNSUPPORTED_VIA_FIELD"
        );
    }
}
