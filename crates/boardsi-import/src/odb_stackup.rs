//! E_020: bounded ODB++Design 8.1-U4 EDA stackup observation.
//! This is source-only, not manufactured dimensions, validated loss or SI.
//! Schema: matrix/stackup.xml (StackupFile/EdaData/Specs, Stackup/Group).
use crate::odb_source::{
    declared_board_copper, import_unpacked, read_source_path_file_map, OdbError, OdbFileMap,
};
use boardsi_model::{Rational, SourceId};
use std::collections::{BTreeMap, BTreeSet};

const PATH: &str = "matrix/stackup.xml";
const MAX_XML_BYTES: usize = 2 * 1024 * 1024;
const MAX_XML_NODES: usize = 8192;
const MAX_XML_DEPTH: usize = 24;

fn fail(line: usize, code: &'static str) -> OdbError {
    OdbError {
        path: PATH.into(),
        line,
        code,
    }
}

#[derive(Clone, Debug)]
struct Element {
    name: String,
    attrs: BTreeMap<String, String>,
    children: Vec<usize>,
    line: usize,
}
impl Element {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).map(String::as_str)
    }
    fn require(&self, name: &str) -> Result<&str, OdbError> {
        self.attr(name)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| fail(self.line, "ODB_STACKUP_MISSING_ATTRIBUTE"))
    }
}
fn decode(raw: &str, line: usize) -> Result<String, OdbError> {
    let mut out = String::new();
    let mut rest = raw;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        rest = &rest[pos..];
        let (replacement, consumed) = if rest.starts_with("&amp;") {
            ("&", 5)
        } else if rest.starts_with("&lt;") {
            ("<", 4)
        } else if rest.starts_with("&gt;") {
            (">", 4)
        } else if rest.starts_with("&quot;") {
            ("\"", 6)
        } else if rest.starts_with("&apos;") {
            ("'", 6)
        } else {
            return Err(fail(line, "ODB_STACKUP_XML_ENTITY"));
        };
        out.push_str(replacement);
        rest = &rest[consumed..];
    }
    out.push_str(rest);
    if out.len() > 1024 || out.contains('\0') {
        return Err(fail(line, "ODB_STACKUP_XML_VALUE"));
    }
    Ok(out)
}
fn xml_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-' || byte == b':'
}
fn parse_tag(input: &str, line: usize) -> Result<(String, BTreeMap<String, String>), OdbError> {
    let bytes = input.as_bytes();
    let mut p = 0;
    while p < bytes.len() && xml_name(bytes[p]) {
        p += 1;
    }
    if p == 0 {
        return Err(fail(line, "ODB_STACKUP_XML_TAG"));
    }
    let name = input[..p].to_owned();
    let mut attrs = BTreeMap::new();
    while p < bytes.len() {
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        if p == bytes.len() {
            break;
        }
        let start = p;
        while p < bytes.len() && xml_name(bytes[p]) {
            p += 1;
        }
        if start == p {
            return Err(fail(line, "ODB_STACKUP_XML_ATTRIBUTE"));
        }
        let key = &input[start..p];
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        if bytes.get(p) != Some(&b'=') {
            return Err(fail(line, "ODB_STACKUP_XML_ATTRIBUTE"));
        }
        p += 1;
        while p < bytes.len() && bytes[p].is_ascii_whitespace() {
            p += 1;
        }
        let quote = *bytes
            .get(p)
            .ok_or_else(|| fail(line, "ODB_STACKUP_XML_QUOTE"))?;
        if quote != b'\'' && quote != b'"' {
            return Err(fail(line, "ODB_STACKUP_XML_QUOTE"));
        }
        p += 1;
        let start = p;
        while p < bytes.len() && bytes[p] != quote {
            if bytes[p] == b'<' {
                return Err(fail(line, "ODB_STACKUP_XML_VALUE"));
            }
            p += 1;
        }
        if p == bytes.len() {
            return Err(fail(line, "ODB_STACKUP_XML_QUOTE"));
        }
        let value = decode(&input[start..p], line)?;
        p += 1;
        if attrs.insert(key.to_owned(), value).is_some() {
            return Err(fail(line, "ODB_STACKUP_DUPLICATE_ATTRIBUTE"));
        }
    }
    Ok((name, attrs))
}
fn xml_tree(bytes: &[u8]) -> Result<Vec<Element>, OdbError> {
    if bytes.len() > MAX_XML_BYTES {
        return Err(fail(0, "ODB_STACKUP_XML_TOO_LARGE"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| fail(0, "ODB_STACKUP_XML_UTF8"))?;
    let mut nodes = Vec::<Element>::new();
    let mut stack = Vec::<usize>::new();
    let mut p = 0;
    let mut line = 1;
    let mut declaration_seen = false;
    while p < text.len() {
        if text.as_bytes()[p] != b'<' {
            let end = text[p..].find('<').map_or(text.len(), |q| p + q);
            if !text[p..end].trim().is_empty() {
                return Err(fail(line, "ODB_STACKUP_XML_TEXT"));
            }
            line += text[p..end].bytes().filter(|x| *x == b'\n').count();
            p = end;
            continue;
        }
        let start_line = line;
        if text[p..].starts_with("<?xml ") && nodes.is_empty() && !declaration_seen {
            let end = text[p..]
                .find("?>")
                .ok_or_else(|| fail(line, "ODB_STACKUP_XML_DECLARATION"))?
                + p
                + 2;
            declaration_seen = true;
            line += text[p..end].bytes().filter(|x| *x == b'\n').count();
            p = end;
            continue;
        }
        if text[p..].starts_with("<!") || text[p..].starts_with("<?") {
            return Err(fail(line, "ODB_STACKUP_XML_EXTERNAL_OR_EXTENSION"));
        }
        let mut end = p + 1;
        let mut quote = None;
        while end < text.len() {
            let b = text.as_bytes()[end];
            if let Some(q) = quote {
                if b == q {
                    quote = None;
                }
            } else if b == b'\'' || b == b'"' {
                quote = Some(b);
            } else if b == b'>' {
                break;
            }
            end += 1;
        }
        if end == text.len() {
            return Err(fail(line, "ODB_STACKUP_XML_UNTERMINATED"));
        }
        let raw = text[p + 1..end].trim();
        if let Some(closing) = raw.strip_prefix('/') {
            if closing.is_empty() || !closing.bytes().all(xml_name) {
                return Err(fail(line, "ODB_STACKUP_XML_CLOSE"));
            }
            let open = stack
                .pop()
                .ok_or_else(|| fail(line, "ODB_STACKUP_XML_CLOSE"))?;
            if nodes[open].name != closing {
                return Err(fail(line, "ODB_STACKUP_XML_CLOSE"));
            }
        } else {
            let self_closing = raw.ends_with('/');
            let inner = if self_closing {
                raw[..raw.len() - 1].trim_end()
            } else {
                raw
            };
            let (name, attrs) = parse_tag(inner, start_line)?;
            if nodes.len() >= MAX_XML_NODES || stack.len() >= MAX_XML_DEPTH {
                return Err(fail(line, "ODB_STACKUP_XML_BUDGET"));
            }
            if stack.is_empty() && !nodes.is_empty() {
                return Err(fail(line, "ODB_STACKUP_XML_MULTIPLE_ROOTS"));
            }
            let id = nodes.len();
            nodes.push(Element {
                name,
                attrs,
                children: Vec::new(),
                line: start_line,
            });
            if let Some(parent) = stack.last() {
                nodes[*parent].children.push(id);
            }
            if !self_closing {
                stack.push(id);
            }
        }
        let next = end + 1;
        line += text[p..next].bytes().filter(|x| *x == b'\n').count();
        p = next;
    }
    if !stack.is_empty() || nodes.is_empty() || nodes[0].name != "StackupFile" {
        return Err(fail(line, "ODB_STACKUP_XML_STRUCTURE"));
    }
    Ok(nodes)
}
fn children(nodes: &[Element], parent: usize, tag: &str) -> Vec<usize> {
    nodes[parent]
        .children
        .iter()
        .copied()
        .filter(|i| nodes[*i].name == tag)
        .collect()
}
fn one(nodes: &[Element], parent: usize, tag: &str) -> Result<usize, OdbError> {
    let found = children(nodes, parent, tag);
    if found.len() != 1 {
        return Err(fail(nodes[parent].line, "ODB_STACKUP_EXPECT_SINGLE"));
    }
    Ok(found[0])
}
fn number(s: &str, line: usize, positive: bool) -> Result<f64, OdbError> {
    let n: f64 = s.parse().map_err(|_| fail(line, "ODB_STACKUP_NUMBER"))?;
    if !n.is_finite() || (positive && n <= 0.0) || (!positive && n < 0.0) {
        return Err(fail(line, "ODB_STACKUP_NUMBER"));
    }
    Ok(n)
}
fn exact_length(s: &str, unit: &str, line: usize) -> Result<Rational, OdbError> {
    let (a, b) = match unit {
        "MM" => (1_i128, 1000_i128),
        "MIL" => (254, 10_000_000),
        "INCH" => (254, 10_000),
        _ => return Err(fail(line, "ODB_STACKUP_UNIT")),
    };
    if s.is_empty() || s.len() > 24 || s.starts_with('-') {
        return Err(fail(line, "ODB_STACKUP_THICKNESS"));
    }
    let parts: Vec<_> = s.split('.').collect();
    if parts.len() > 2
        || parts[0].is_empty()
        || !parts.iter().all(|x| x.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(fail(line, "ODB_STACKUP_THICKNESS"));
    }
    let fraction = if parts.len() == 2 { parts[1].len() } else { 0 };
    if fraction > 9 {
        return Err(fail(line, "ODB_STACKUP_PRECISION"));
    }
    let digits = parts.join("");
    let n = digits
        .parse::<i128>()
        .map_err(|_| fail(line, "ODB_STACKUP_THICKNESS"))?;
    let denom = 10_i128
        .checked_pow(fraction as u32)
        .and_then(|q| q.checked_mul(b))
        .ok_or_else(|| fail(line, "ODB_STACKUP_OVERFLOW"))?;
    let numer = n
        .checked_mul(a)
        .ok_or_else(|| fail(line, "ODB_STACKUP_OVERFLOW"))?;
    let r = Rational::new(numer, denom).map_err(|_| fail(line, "ODB_STACKUP_OVERFLOW"))?;
    if !r.positive() {
        return Err(fail(line, "ODB_STACKUP_THICKNESS"));
    }
    Ok(r)
}
fn thickness(
    n: &Element,
    key: &str,
    default_unit: &str,
    unit_key: &str,
) -> Result<Option<Rational>, OdbError> {
    n.attr(key)
        .map(|s| exact_length(s, n.attr(unit_key).unwrap_or(default_unit), n.line))
        .transpose()
}
fn frequency_hz(n: &Element) -> Result<Option<f64>, OdbError> {
    let Some(s) = n.attr("FrequencyVal") else {
        return Ok(None);
    };
    let unit = n.attr("Units").unwrap_or("GHz");
    let mul = match unit {
        "GHz" => 1e9,
        "MHz" => 1e6,
        "KHz" => 1e3,
        "Hz" => 1.0,
        _ => return Err(fail(n.line, "ODB_STACKUP_FREQUENCY_UNIT")),
    };
    let hz = number(s, n.line, true)? * mul;
    if !hz.is_finite() {
        return Err(fail(n.line, "ODB_STACKUP_FREQUENCY"));
    }
    Ok(Some(hz))
}
#[derive(Clone, Debug, PartialEq)]
pub struct SourceDielectricProperty {
    pub property_name: String,
    pub frequency_hz: Option<f64>,
    pub dk: Option<f64>,
    pub df: Option<f64>,
    pub source_line: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SourceStackupLayer {
    pub layer_name: String,
    pub layer_type: String,
    pub material_spec_name: String,
    pub material_name: String,
    pub thickness_m: Option<Rational>,
    pub finished_thickness_m: Option<Rational>,
    /// BoardSI Canonical z is top-face, increasing from top to bottom.
    /// None unless all layer thicknesses have the same complete basis.
    pub z_top_m: Option<Rational>,
    pub dielectric_properties: Vec<SourceDielectricProperty>,
    pub source_line: usize,
}
#[derive(Debug)]
pub struct OdbSourceStackup {
    pub source_identity: SourceId,
    pub matrix_sha256: [u8; 32],
    pub xml_sha256: [u8; 32],
    pub layers: Vec<SourceStackupLayer>,
    pub complete_z: bool,
}
type SelectedMaterial = (
    Option<Rational>,
    Option<Rational>,
    Vec<SourceDielectricProperty>,
);
fn selected_material(
    nodes: &[Element],
    material: usize,
    selected: Option<&str>,
    default_unit: &str,
) -> Result<SelectedMaterial, OdbError> {
    let bases = children(nodes, material, "Default_Thickness");
    if bases.len() > 1 {
        return Err(fail(
            nodes[material].line,
            "ODB_STACKUP_DUPLICATE_DEFAULT_THICKNESS",
        ));
    }
    // A source may specify a material but omit its thickness. This is UNKNOWN,
    // not a guessed zero or a reason to publish a complete-z receipt.
    let mut nominal = bases
        .first()
        .map(|base| thickness(&nodes[*base], "Thickness", default_unit, "Units"))
        .transpose()?
        .flatten();
    let mut finished = bases
        .first()
        .map(|base| thickness(&nodes[*base], "Finished_Thickness", default_unit, "Units"))
        .transpose()?
        .flatten();
    let dielectric = children(nodes, material, "Dielectric");
    let conductor = children(nodes, material, "Conductor");
    if dielectric.len() + conductor.len() != 1 {
        return Err(fail(nodes[material].line, "ODB_STACKUP_MATERIAL_KIND"));
    }
    let entity = dielectric.first().or(conductor.first()).copied().unwrap();
    // Conductor resin values are not dielectric layer Dk/Df in this bounded slice.
    if !conductor.is_empty() && !children(nodes, entity, "Properties").is_empty() {
        return Err(fail(
            nodes[entity].line,
            "ODB_STACKUP_CONDUCTOR_PROPERTIES_UNSUPPORTED",
        ));
    }
    let mut props = Vec::new();
    let mut selected_group = false;
    let mut override_line = None;
    let mut frequencies = BTreeSet::new();
    for group in children(nodes, entity, "Properties") {
        let name = nodes[group].require("PropertyName")?.to_owned();
        if selected.is_some_and(|s| s != name) {
            continue;
        }
        if selected_group {
            return Err(fail(
                nodes[group].line,
                "ODB_STACKUP_DUPLICATE_PROPERTY_GROUP",
            ));
        }
        selected_group = true;
        for property in children(nodes, group, "Property") {
            let n = &nodes[property];
            let dk = n
                .attr("DielectricConstant_Dk")
                .map(|s| number(s, n.line, true))
                .transpose()?;
            let df = n
                .attr("LossTangent_Df")
                .map(|s| number(s, n.line, false))
                .transpose()?;
            let frequency = frequency_hz(n)?;
            if !frequencies.insert(frequency.map(f64::to_bits)) {
                return Err(fail(n.line, "ODB_STACKUP_DUPLICATE_FREQUENCY"));
            }
            props.push(SourceDielectricProperty {
                property_name: name.clone(),
                frequency_hz: frequency,
                dk,
                df,
                source_line: n.line,
            });
            let overridden = thickness(n, "Thickness", default_unit, "Units_Thickness")?;
            let finished_override =
                thickness(n, "Finished_Thickness", default_unit, "Units_Thickness")?;
            if overridden.is_some() || finished_override.is_some() {
                override_line = Some(n.line);
                if overridden.is_some() {
                    nominal = overridden;
                }
                if finished_override.is_some() {
                    finished = finished_override;
                }
            }
        }
    }
    if let Some(line) = override_line {
        if props.len() != 1 {
            return Err(fail(line, "ODB_STACKUP_AMBIGUOUS_THICKNESS"));
        }
    }
    if selected.is_some() && props.is_empty() {
        return Err(fail(nodes[material].line, "ODB_STACKUP_PROPERTY_NOT_FOUND"));
    }
    if selected.is_none() && children(nodes, entity, "Properties").len() > 1 {
        return Err(fail(nodes[entity].line, "ODB_STACKUP_AMBIGUOUS_PROPERTY"));
    }
    Ok((nominal, finished, props))
}
/// Transactional Source-only EDA stackup audit. An absent optional XML is not an
/// error; SupplierData alone is never promoted into design stackup.
pub fn audit_unpacked_stackup(
    files: &OdbFileMap,
    step: &str,
) -> Result<Option<OdbSourceStackup>, OdbError> {
    let board = import_unpacked(files, step)?;
    let Some(xml) = files.get(PATH) else {
        return Ok(None);
    };
    let nodes = xml_tree(xml)?;
    let root = &nodes[0];
    if root.require("Version")? != "8.1" {
        return Err(fail(1, "ODB_STACKUP_VERSION"));
    }
    let default_unit = root.attr("DefaultUnits").unwrap_or("MIL");
    if !matches!(default_unit, "MM" | "MIL" | "INCH") {
        return Err(fail(1, "ODB_STACKUP_UNIT"));
    }
    let eda = children(&nodes, 0, "EdaData");
    if eda.is_empty() {
        return Ok(None);
    }
    if eda.len() != 1 {
        return Err(fail(1, "ODB_STACKUP_EDA_AMBIGUOUS"));
    }
    let spec_root = one(&nodes, eda[0], "Specs")?;
    let layer_root = one(&nodes, eda[0], "Stackup")?;
    if let Some(target) = nodes[layer_root].attr("StackupThickness") {
        let unit = nodes[layer_root].attr("Units").unwrap_or(default_unit);
        let _target_thickness = exact_length(target, unit, nodes[layer_root].line)?;
        // Target thickness may differ from nominal layer sum; no silent equality assumption.
    }
    let groups = children(&nodes, layer_root, "Group");
    if groups.len() != 1 {
        return Err(fail(nodes[layer_root].line, "ODB_STACKUP_GROUP_AMBIGUOUS"));
    }
    let mut materials = BTreeMap::<(String, String), usize>::new();
    for spec in children(&nodes, spec_root, "Spec") {
        let spec_name = nodes[spec].require("SpecName")?;
        for material in children(&nodes, spec, "Material") {
            let name = nodes[material].require("MaterialName")?;
            if materials
                .insert((spec_name.to_owned(), name.to_owned()), material)
                .is_some()
            {
                return Err(fail(nodes[material].line, "ODB_STACKUP_DUPLICATE_MATERIAL"));
            }
        }
    }
    // A nested Group can hide dielectric layers from the copper-only Matrix check.
    // Reject it rather than calculate a misleading complete-z stackup.
    if nodes[groups[0]]
        .children
        .iter()
        .any(|id| nodes[*id].name != "Layer")
    {
        return Err(fail(
            nodes[groups[0]].line,
            "ODB_STACKUP_NESTED_OR_UNKNOWN_GROUP_CHILD",
        ));
    }
    let mut layers = Vec::new();
    let mut names = BTreeSet::new();
    let mut copper = Vec::new();
    for id in children(&nodes, groups[0], "Layer") {
        let e = &nodes[id];
        if e.children.iter().any(|i| nodes[*i].name != "SpecRef") {
            return Err(fail(e.line, "ODB_STACKUP_LAYER_CHILD_UNSUPPORTED"));
        }
        let name = e.require("LayerName")?.to_owned();
        if !names.insert(name.clone()) {
            return Err(fail(e.line, "ODB_STACKUP_DUPLICATE_LAYER"));
        }
        let kind = e.require("LayerType")?;
        let is_copper = matches!(kind, "SIGNAL" | "POWER_GROUND" | "MIXED");
        if !is_copper && kind != "DIELECTRIC" {
            return Err(fail(e.line, "ODB_STACKUP_LAYER_TYPE_UNSUPPORTED"));
        }
        if is_copper {
            copper.push(name.clone());
        }
        let refs = children(&nodes, id, "SpecRef");
        if refs.len() != 1 {
            return Err(fail(e.line, "ODB_STACKUP_REF_AMBIGUOUS"));
        }
        let spec_name = nodes[refs[0]].require("MaterialSpecName")?.to_owned();
        let mat_ref = one(&nodes, refs[0], "Material")?;
        let mat_name = nodes[mat_ref].require("MaterialName")?.to_owned();
        let material = *materials
            .get(&(spec_name.clone(), mat_name.clone()))
            .ok_or_else(|| fail(e.line, "ODB_STACKUP_UNKNOWN_MATERIAL"))?;
        let expected = if is_copper { "Conductor" } else { "Dielectric" };
        if children(&nodes, material, expected).len() != 1 {
            return Err(fail(e.line, "ODB_STACKUP_MATERIAL_LAYER_CONFLICT"));
        }
        let selected = nodes[mat_ref].attr("PropertyName");
        let (thick, finished, props) = selected_material(&nodes, material, selected, default_unit)?;
        layers.push(SourceStackupLayer {
            layer_name: name,
            layer_type: kind.into(),
            material_spec_name: spec_name,
            material_name: mat_name,
            thickness_m: thick,
            finished_thickness_m: finished,
            z_top_m: None,
            dielectric_properties: props,
            source_line: e.line,
        });
    }
    let matrix_copper: Vec<_> = board
        .stackup
        .layers
        .iter()
        .map(|l| l.name.clone())
        .collect();
    if copper != matrix_copper {
        return Err(fail(
            nodes[groups[0]].line,
            "ODB_STACKUP_MATRIX_LAYER_MISMATCH",
        ));
    }
    let xml_types: Vec<_> = layers
        .iter()
        .filter(|layer| {
            matches!(
                layer.layer_type.as_str(),
                "SIGNAL" | "POWER_GROUND" | "MIXED"
            )
        })
        .map(|layer| (layer.layer_name.clone(), layer.layer_type.clone()))
        .collect();
    if xml_types != declared_board_copper(files)? {
        return Err(fail(
            nodes[groups[0]].line,
            "ODB_STACKUP_MATRIX_LAYER_TYPE_MISMATCH",
        ));
    }
    let complete = !layers.is_empty() && layers.iter().all(|l| l.thickness_m.is_some());
    if complete {
        let mut z = Rational::integer(0);
        for layer in &mut layers {
            layer.z_top_m = Some(z);
            z = z
                .checked_add(layer.thickness_m.unwrap())
                .map_err(|_| fail(layer.source_line, "ODB_STACKUP_OVERFLOW"))?;
        }
    }
    let matrix = files
        .get("matrix/matrix")
        .ok_or_else(|| fail(0, "ODB_STACKUP_MISSING_MATRIX"))?;
    Ok(Some(OdbSourceStackup {
        source_identity: board.sources[0].id.clone(),
        matrix_sha256: crate::digest::sha256(matrix),
        xml_sha256: crate::digest::sha256(xml),
        layers,
        complete_z: complete,
    }))
}
/// Existing bounded directory/USTAR/ZIP/GZIP FileMap loaders; no extraction.
pub fn audit_path_stackup(
    path: &std::path::Path,
    step: &str,
) -> Result<Option<OdbSourceStackup>, OdbError> {
    audit_unpacked_stackup(&read_source_path_file_map(path)?, step)
}
