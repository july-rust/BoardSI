//! E_022: ODB++ SOURCE-only local positive copper overlap, not manufactured continuity.
//!
//! This opt-in adapter translates an already validated ODB++ Canonical subset to
//! the existing D_056_02/03 evidence graph. It never mutates BoardModel.connectivity,
//! fabricates plating, certifies absent edges as Opens, or sends edges to SI.
use crate::electrical_graph_qualification::{self, QualificationReport, SourceCoverage};
use crate::layered_source_graph::{
    ContactKind, CopperFootprint, CopperNode, DeclaredNetCheck, GeometryContact, LayeredSourceGraph,
};
use crate::odb_source::{self, OdbError, OdbFileMap};
use boardsi_model::{BoardModel, ObjectId, PadRole, PadShape, Point, Rational, SourceId};
use std::collections::BTreeMap;

const MAX_NODES: usize = 50_000;
const MAX_PAIRS: usize = 2_000_000;
const MAX_EDGES: usize = 100_000;

#[derive(Debug)]
pub struct OdbLocalGraphQualification {
    /// Digest of the complete ordered source-file SHA manifest plus STEP.
    pub source_sha256: [u8; 32],
    pub source_identity: SourceId,
    pub graph: LayeredSourceGraph,
    /// SOURCE-local design edges only, never manufactured or SI edges.
    pub analysis: QualificationReport,
    /// Indeterminate narrow-phase comparisons, not negative contact proof.
    pub unknown_pairs: usize,
}

fn fail(code: &'static str) -> OdbError {
    OdbError {
        path: "E_022".into(),
        line: 0,
        code,
    }
}

fn source_digest(board: &BoardModel, step: &str) -> Result<[u8; 32], OdbError> {
    let source = board
        .sources
        .first()
        .ok_or_else(|| fail("E022_MISSING_SOURCE"))?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BOARDSI_E022_SOURCE_V1");
    bytes.extend_from_slice(&(step.len() as u64).to_le_bytes());
    bytes.extend_from_slice(step.as_bytes());
    for file in &source.id.files {
        bytes.extend_from_slice(&(file.logical_path.len() as u64).to_le_bytes());
        bytes.extend_from_slice(file.logical_path.as_bytes());
        bytes.extend_from_slice(&file.sha256);
    }
    Ok(crate::digest::sha256(&bytes))
}

fn net_name(board: &BoardModel, id: &Option<ObjectId>) -> Option<String> {
    id.as_ref().and_then(|key| {
        board
            .nets
            .iter()
            .find(|net| &net.meta.id == key)
            .map(|net| net.name.clone())
    })
}
fn line_number(provenance: &str) -> Result<usize, OdbError> {
    provenance
        .rsplit_once(":line:")
        .and_then(|(_, line)| line.parse::<usize>().ok())
        .filter(|line| *line > 0)
        .ok_or_else(|| fail("E022_MISSING_SOURCE_LINE"))
}
fn width_ticks(value: Rational, tick_m: Rational) -> Result<i64, OdbError> {
    if !value.positive() || !tick_m.positive() {
        return Err(fail("E022_BAD_WIDTH"));
    }
    let numerator = value
        .numerator()
        .checked_mul(tick_m.denominator())
        .ok_or_else(|| fail("E022_DIMENSION_OVERFLOW"))?;
    let denominator = value
        .denominator()
        .checked_mul(tick_m.numerator())
        .ok_or_else(|| fail("E022_DIMENSION_OVERFLOW"))?;
    let ratio =
        Rational::new(numerator, denominator).map_err(|_| fail("E022_DIMENSION_OVERFLOW"))?;
    if ratio.denominator() != 1 {
        return Err(fail("E022_NONINTEGRAL_WIDTH_TICKS"));
    }
    let value = i64::try_from(ratio.numerator()).map_err(|_| fail("E022_WIDTH_OVERFLOW"))?;
    if value == 0 {
        return Err(fail("E022_BAD_WIDTH"));
    }
    Ok(value)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Contact {
    Positive,
    Separate,
    Uncertain,
}

/// Strict positive overlap between the closed round-cap stroke (capsule)
/// and a solid circular pad. Compute relative coordinates in source ticks.
/// The conservative binary64 guard prevents near-tangent/precision-loss
/// comparisons from being promoted to copper edges. An undecidable comparison
/// is UNKNOWN, not an Open. No numerical tolerance creates a positive edge.
fn capsule_circle(a: &Point, b: &Point, p: &Point, trace_width: i64, pad_diameter: i64) -> Contact {
    if a.frame != b.frame || a.frame != p.frame || trace_width <= 0 || pad_diameter <= 0 {
        return Contact::Uncertain;
    }
    let dx = (i128::from(b.ticks[0]) - i128::from(a.ticks[0])) as f64;
    let dy = (i128::from(b.ticks[1]) - i128::from(a.ticks[1])) as f64;
    let px = (i128::from(p.ticks[0]) - i128::from(a.ticks[0])) as f64;
    let py = (i128::from(p.ticks[1]) - i128::from(a.ticks[1])) as f64;
    let radius = (trace_width as f64 + pad_diameter as f64) * 0.5;
    let scale = dx
        .abs()
        .max(dy.abs())
        .max(px.abs())
        .max(py.abs())
        .max(radius);
    if !scale.is_finite() || scale > 1.0e13 || dx == 0.0 && dy == 0.0 {
        return Contact::Uncertain;
    }
    let len2 = dx * dx + dy * dy;
    if !len2.is_finite() || len2 == 0.0 {
        return Contact::Uncertain;
    }
    let t = ((px * dx + py * dy) / len2).clamp(0.0, 1.0);
    let dist = (px - t * dx).hypot(py - t * dy);
    let guard = 1024.0 * f64::EPSILON * scale.max(1.0);
    if !dist.is_finite() || !guard.is_finite() {
        return Contact::Uncertain;
    }
    if dist + guard < radius {
        Contact::Positive
    } else if dist > radius + guard {
        Contact::Separate
    } else {
        Contact::Uncertain
    }
}

/// Rebuild from one immutable logical ODB++ source snapshot. Callers may pass
/// the digest returned by the prior accepted import to reject stale re-import.
pub fn qualify_unpacked(
    files: &OdbFileMap,
    step: &str,
    with_eda: bool,
    expected_source_sha256: Option<[u8; 32]>,
) -> Result<OdbLocalGraphQualification, OdbError> {
    let board = if with_eda {
        odb_source::import_unpacked_with_eda(files, step)?
    } else {
        odb_source::import_unpacked(files, step)?
    };
    if board.sources.len() != 1 || board.validate().has_errors() {
        return Err(fail("E022_CANONICAL_INVALID"));
    }
    // E_022 intentionally excludes Step Repeat, arcs, surfaces, vias/drills,
    // unverified layer transformations and mixed/unsupported pad shapes.
    let layers = board.stackup.layers.len();
    if !(2..=64).contains(&layers)
        || board.frames.len() != layers
        || !board.arcs.is_empty()
        || !board.areas.is_empty()
        || !board.vias.is_empty()
        || !board.drills.is_empty()
    {
        return Err(fail("E022_UNSUPPORTED_COPPER_SCOPE"));
    }
    let digest = source_digest(&board, step)?;
    if expected_source_sha256.is_some_and(|expected| expected != digest) {
        return Err(fail("E022_STALE_SOURCE_SHA"));
    }

    let mut nodes = Vec::new();
    let mut trace_nodes = Vec::new();
    for (index, trace) in board.traces.iter().enumerate() {
        if trace.centreline.len() != 2 || trace.centreline[0].frame != trace.centreline[1].frame {
            return Err(fail("E022_UNSUPPORTED_TRACE"));
        }
        let layer_index = board
            .stackup
            .layers
            .iter()
            .position(|l| l.meta.id == trace.layer)
            .ok_or_else(|| fail("E022_INVALID_LAYER"))?;
        let frame = board
            .frames
            .iter()
            .find(|f| f.meta.id == trace.centreline[0].frame)
            .ok_or_else(|| fail("E022_INVALID_FRAME"))?;
        let width = width_ticks(trace.width_m, frame.metres_per_tick)?;
        if nodes.len() >= MAX_NODES {
            return Err(fail("E022_NODE_LIMIT"));
        }
        trace_nodes.push((nodes.len(), index, width));
        nodes.push(CopperNode {
            source_key: format!("odb:trace:{}", trace.meta.id.source_object),
            source_line: Some(line_number(&trace.meta.provenance.source_object_type)?),
            physical_copper_layer: layer_index as u32 + 1,
            declared_net: net_name(&board, &trace.net),
            footprint: CopperFootprint::Stroke { width_ticks: width },
        });
    }
    let mut pad_nodes = Vec::new();
    for (index, pad) in board.pads.iter().enumerate() {
        let stack = board
            .padstacks
            .iter()
            .find(|s| s.meta.id == pad.padstack)
            .ok_or_else(|| fail("E022_MISSING_PADSTACK"))?;
        if stack.entries.len() != 1 || stack.entries[0].role != PadRole::Conductive {
            return Err(fail("E022_UNSUPPORTED_PADSTACK"));
        }
        let entry = &stack.entries[0];
        let PadShape::Circle { diameter_m } = &entry.shape else {
            return Err(fail("E022_UNSUPPORTED_PAD_SHAPE"));
        };
        let layer_index = board
            .stackup
            .layers
            .iter()
            .position(|l| l.meta.id == entry.layer)
            .ok_or_else(|| fail("E022_INVALID_LAYER"))?;
        let frame = board
            .frames
            .iter()
            .find(|f| f.meta.id == pad.position.frame)
            .ok_or_else(|| fail("E022_INVALID_FRAME"))?;
        let diameter = width_ticks(*diameter_m, frame.metres_per_tick)?;
        if nodes.len() >= MAX_NODES {
            return Err(fail("E022_NODE_LIMIT"));
        }
        pad_nodes.push((nodes.len(), index, diameter));
        nodes.push(CopperNode {
            source_key: format!("odb:pad:{}", pad.meta.id.source_object),
            source_line: Some(line_number(&pad.meta.provenance.source_object_type)?),
            physical_copper_layer: layer_index as u32 + 1,
            declared_net: net_name(&board, &pad.net),
            footprint: CopperFootprint::SmdPad {
                centre: pad.position.ticks,
                width_ticks: diameter,
                height_ticks: diameter,
                rectangular: false,
            },
        });
    }

    let mut contacts = Vec::new();
    let mut pairs = 0usize;
    let mut unknown_pairs = 0usize;
    for (trace_node, trace_index, width) in trace_nodes {
        let trace = &board.traces[trace_index];
        for (pad_node, pad_index, diameter) in &pad_nodes {
            if nodes[trace_node].physical_copper_layer != nodes[*pad_node].physical_copper_layer {
                continue;
            }
            pairs = pairs
                .checked_add(1)
                .ok_or_else(|| fail("E022_PAIR_LIMIT"))?;
            if pairs > MAX_PAIRS {
                return Err(fail("E022_PAIR_LIMIT"));
            }
            let pad = &board.pads[*pad_index];
            match capsule_circle(
                &trace.centreline[0],
                &trace.centreline[1],
                &pad.position,
                width,
                *diameter,
            ) {
                Contact::Positive => {
                    if contacts.len() >= MAX_EDGES {
                        return Err(fail("E022_CONTACT_LIMIT"));
                    }
                    let relation = match (
                        &nodes[trace_node].declared_net,
                        &nodes[*pad_node].declared_net,
                    ) {
                        (Some(a), Some(b)) if a == b => DeclaredNetCheck::Same,
                        (Some(_), Some(_)) => DeclaredNetCheck::Different,
                        _ => DeclaredNetCheck::Unknown,
                    };
                    contacts.push(GeometryContact {
                        first_node: trace_node,
                        second_node: *pad_node,
                        layer: nodes[trace_node].physical_copper_layer,
                        kind: ContactKind::TraceSmdPad,
                        declared_net_check: relation,
                    });
                }
                Contact::Separate => {}
                Contact::Uncertain => unknown_pairs += 1,
            }
        }
    }
    let graph = LayeredSourceGraph {
        source_sha256: digest,
        declared_copper_layers: layers as u32,
        nodes,
        geometry_contacts: contacts,
        unverified_barrels: Vec::new(),
        verified_electrical_edges: Vec::new(),
    };
    // Partial source scope prevents global Open assertions. No barrel witnesses.
    let analysis =
        electrical_graph_qualification::qualify(&graph, digest, &SourceCoverage::Partial, &[], &[])
            .map_err(fail)?;
    Ok(OdbLocalGraphQualification {
        source_sha256: digest,
        source_identity: board.sources[0].id.clone(),
        graph,
        analysis,
        unknown_pairs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(layers: usize, trace: &str, pad: &str) -> OdbFileMap {
        let mut files = OdbFileMap::new();
        let mut matrix = "STEP {\nCOL=1\nNAME=pcb\n}\n".to_owned();
        for layer in 1..=layers {
            let name = format!("sig{layer}");
            matrix.push_str(&format!(
                "LAYER {{\nROW={layer}\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME={name}\nPOLARITY=POSITIVE\n}}\n"
            ));
            let features = format!("UNITS=MM\nF 2\n$0 r100 M\nL {trace} 0 P 0\nP {pad} 0 P 0 0\n");
            files.insert(
                format!("steps/pcb/layers/{name}/features"),
                features.into_bytes(),
            );
        }
        files.insert("matrix/matrix".into(), matrix.into_bytes());
        files
    }
    fn run(files: &OdbFileMap) -> OdbLocalGraphQualification {
        qualify_unpacked(files, "pcb", false, None).unwrap()
    }
    #[test]
    fn independent_2_4_6_8_64_layer_positive_oracle() {
        for layers in [2, 4, 6, 8, 64] {
            let files = fixture(layers, "0 0 2 0", "1 0.05");
            let q = run(&files);
            assert_eq!(q.analysis.edges.len(), layers);
            assert_eq!(q.graph.geometry_contacts.len(), layers);
            assert_eq!(q.analysis.components.len(), layers);
            assert!(q.analysis.net_conflicts.is_empty());
            assert!(q.graph.verified_electrical_edges.is_empty());
            assert!(!q.analysis.source_scope_qualified);
            assert_eq!(q.unknown_pairs, 0);
            assert!(q.analysis.open_witnesses.is_empty());
        }
    }
    #[test]
    fn offcentre_endpoint_and_midspan_contacts() {
        for (trace, pad) in [
            ("0 0 2 0", "0 0.05"),
            ("0 0 2 0", "1 0.09"),
            ("0 0 1 1", "0.65 0.6"),
            ("0 0 1 1", "1.03 1.02"),
        ] {
            assert_eq!(run(&fixture(2, trace, pad)).analysis.edges.len(), 2);
        }
    }
    #[test]
    fn separate_and_tangent_are_not_electrical_edges() {
        assert!(run(&fixture(2, "0 0 2 0", "1 0.5"))
            .analysis
            .edges
            .is_empty());
        let tangent = run(&fixture(2, "0 0 2 0", "1 0.1"));
        assert!(tangent.analysis.edges.is_empty());
        assert_eq!(tangent.unknown_pairs, 2);
        assert!(!tangent.analysis.source_scope_qualified);
    }
    #[test]
    fn same_xy_other_layer_is_not_contact() {
        let mut files = fixture(2, "0 0 2 0", "1 0");
        files.insert(
            "steps/pcb/layers/sig1/features".into(),
            b"UNITS=MM\nF 1\n$0 r100 M\nL 0 0 2 0 0 P 0\n".to_vec(),
        );
        files.insert(
            "steps/pcb/layers/sig2/features".into(),
            b"UNITS=MM\nF 1\n$0 r100 M\nP 1 0 0 P 0 0\n".to_vec(),
        );
        let result = run(&files);
        assert!(result.analysis.edges.is_empty());
        assert_eq!(result.graph.nodes.len(), 2);
    }
    #[test]
    fn declared_cross_net_conflict_and_unknown_net() {
        let mut files = fixture(2, "0 0 2 0", "1 0");
        files.insert("steps/pcb/eda/data".into(), b"HDR test\nUNITS=MM\nLYR sig1 sig2\nNET A\nSNT TRC\nFID C 0 0\nNET B\nSNT VIA\nFID C 0 1\n".to_vec());
        let conflict = qualify_unpacked(&files, "pcb", true, None).unwrap();
        assert_eq!(conflict.analysis.net_conflicts.len(), 1);
        assert_eq!(
            conflict.graph.geometry_contacts[0].declared_net_check,
            DeclaredNetCheck::Different
        );
        assert!(conflict.graph.verified_electrical_edges.is_empty());
        let unknown = run(&files);
        assert_eq!(
            unknown.graph.geometry_contacts[0].declared_net_check,
            DeclaredNetCheck::Unknown
        );
        assert!(unknown.analysis.net_conflicts.is_empty());
    }
    #[test]
    fn same_net_does_not_invent_separated_edge() {
        let mut files = fixture(2, "0 0 2 0", "1 0.5");
        files.insert(
            "steps/pcb/eda/data".into(),
            b"HDR test\nUNITS=MM\nLYR sig1 sig2\nNET A\nSNT TRC\nFID C 0 0\nSNT VIA\nFID C 0 1\n"
                .to_vec(),
        );
        let q = qualify_unpacked(&files, "pcb", true, None).unwrap();
        assert!(q.analysis.edges.is_empty());
        assert_eq!(q.analysis.components.len(), 4);
    }
    #[test]
    fn stale_sha_reimport_and_bad_eda_fail_closed() {
        let mut files = fixture(2, "0 0 2 0", "1 0");
        let original = run(&files).source_sha256;
        files
            .get_mut("matrix/matrix")
            .unwrap()
            .extend_from_slice(b"# mutation\n");
        assert_eq!(
            qualify_unpacked(&files, "pcb", false, Some(original))
                .unwrap_err()
                .code,
            "E022_STALE_SOURCE_SHA"
        );
        let revised = run(&files);
        assert_ne!(revised.source_sha256, original);
        files.insert(
            "steps/pcb/eda/data".into(),
            b"HDR test\nUNITS=MM\nLYR sig1 sig2\nNET A\nSNT TRC\nFID C 0 98\n".to_vec(),
        );
        assert_eq!(
            qualify_unpacked(&files, "pcb", true, None)
                .unwrap_err()
                .code,
            "ODB_EDA_UNRESOLVED_FID"
        );
    }
    #[test]
    fn unsupported_shapes_and_conductors_fail_closed() {
        let mut files = fixture(2, "0 0 2 0", "1 0");
        files
            .get_mut("steps/pcb/layers/sig1/features")
            .unwrap()
            .extend_from_slice(b"A 1 0 0 1 0 0 0 P 0 Y\n");
        assert!(qualify_unpacked(&files, "pcb", false, None).is_err());
        let mut files = fixture(2, "0 0 2 0", "1 0");
        let path = "steps/pcb/layers/sig1/features";
        let s = String::from_utf8(files[path].clone())
            .unwrap()
            .replace("$0 r100 M", "$0 rect100x100 M");
        files.insert(path.into(), s.into_bytes());
        assert!(qualify_unpacked(&files, "pcb", false, None).is_err());
    }
    #[test]
    fn exact_source_identity_and_repeat_determinism() {
        let files = fixture(2, "0 0 2 0", "1 0");
        let a = run(&files);
        let b = run(&files);
        assert_eq!(a.source_sha256, b.source_sha256);
        assert_eq!(a.source_identity, b.source_identity);
        assert_eq!(a.graph.geometry_contacts, b.graph.geometry_contacts);
        assert_eq!(a.analysis.edges, b.analysis.edges);
        assert_eq!(a.analysis.unverified_barrel_keys.len(), 0);
    }
}
