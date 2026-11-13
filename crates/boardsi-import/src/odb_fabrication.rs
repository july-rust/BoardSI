//! D_056_08: opt-in, source-qualified drill-hit fabrication observations.
//! This is not manufactured-hole qualification, conductive Via creation, or SI.
use std::{collections::BTreeMap, path::Path};

use boardsi_model::{
    BoardModel, FabricationHole, FabricationHoleShape, FabricationOperation, FabricationPlating,
    FabricationSourceEvidence, PhysicalLayerSpan, Rational, SourceId,
    SourceStackupEvidence, SourceViaEvidenceState, SourceViaHoleObservation,
};

use crate::odb_drill::{DrillIntent, SourceDrillLayer};
use crate::odb_drill_hits::SourceDrillHit;
use crate::odb_source::{
    audit_unpacked_drill_hits, audit_unpacked_drill_tools, import_unpacked,
    import_unpacked_with_eda, OdbError, OdbFileMap,
};

#[derive(Debug)]
pub struct OdbSourceHole {
    /// Exact hole centre in board XY metres; never rounded via f64.
    pub xy_m: [Rational; 2],
    /// Format-neutral fabrication intent. NOT a conductive Canonical Via.
    pub fabrication: FabricationHole,
    /// Original hit/tool/matrix SHA and line witnesses, including tool number.
    pub hit: SourceDrillHit,
}

/// E_019: exact source-backed, nonconductive via candidate, never an actual
/// Canonical Via/Drill, proved barrel, connected-layer set, or SI input.
#[derive(Clone, Debug)]
pub struct OdbPlatedViaCandidate {
    /// Canonical source identity includes verified hashes of all source files.
    pub source_identity: SourceId,
    /// Stable within one SourceIdentity; pair both to identify a candidate.
    pub candidate_key: String,
    pub xy_m: [Rational; 2],
    pub fabrication: FabricationHole,
    /// Finished-hole and drilling-tool diameters MUST remain separate.
    pub finished_diameter_m: Rational,
    pub tool_diameter_m: Option<Rational>,
    /// E_004 observation: physical envelope, NOT a connected-layer set.
    pub source_observation: SourceViaHoleObservation,
    /// Original SHA/path/line, tool number and feature ordinal witnesses.
    pub hit: SourceDrillHit,
}

impl OdbPlatedViaCandidate {
    /// A source PLATED declaration is not manufactured electrical proof.
    pub fn has_verified_barrel(&self) -> bool {
        false
    }
    pub fn is_solver_ready(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct OdbSourceImport {
    /// Original canonical copper/optional design-net import, unchanged.
    pub board: BoardModel,
    /// All declared drill hits that can be represented WITHOUT guessing span,
    /// diameter, backdrill removal, or manufactured interconnect.
    pub source_holes: Vec<OdbSourceHole>,
    /// Explicit TYPE=PLATED, TYPE2=STANDARD round-hole candidates only.
    /// This sidecar does not mutate BoardModel::vias/drills/connectivity.
    pub via_candidates: Vec<OdbPlatedViaCandidate>,
}

fn error(hit: &SourceDrillHit, code: &'static str) -> OdbError {
    OdbError {
        path: hit.source_file.clone(),
        line: hit.source_line,
        code,
    }
}

fn xy_m(hit: &SourceDrillHit) -> Result<[Rational; 2], OdbError> {
    let (mul, denom): (i128, i128) = match hit.units.as_str() {
        "MM" => (1, 1_000_000_000_000),
        "INCH" => (254, 10_000_000_000_000),
        _ => return Err(error(hit, "ODB_HOLE_BAD_UNITS")),
    };
    let convert = |value: &str| {
        let ticks = crate::odb_source::decimal_ticks(value, &hit.source_file, hit.source_line)?;
        Rational::new(i128::from(ticks) * mul, denom)
            .map_err(|_| error(hit, "ODB_HOLE_COORDINATE_OVERFLOW"))
    };
    Ok([convert(&hit.x)?, convert(&hit.y)?])
}

fn source_holes(
    board: &BoardModel,
    layers: &[SourceDrillLayer],
    hits: Vec<SourceDrillHit>,
) -> Result<Vec<OdbSourceHole>, OdbError> {
    let mut by_layer = BTreeMap::new();
    for layer in layers {
        if by_layer.insert(layer.name.as_str(), layer).is_some() {
            return Err(OdbError {
                path: "matrix/matrix".into(),
                line: layer.matrix_source_line,
                code: "ODB_HOLE_DUPLICATE_DRILL_LAYER",
            });
        }
    }
    let copper: BTreeMap<&str, u32> = board
        .stackup
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| (layer.name.as_str(), (index + 1) as u32))
        .collect();
    let copper_count = board.stackup.layers.len() as u32;
    let mut out = Vec::with_capacity(hits.len());
    for hit in hits {
        let layer = by_layer
            .get(hit.layer_name.as_str())
            .ok_or_else(|| error(&hit, "ODB_HOLE_UNDECLARED_LAYER"))?;
        // Drill-layer span absence must not be interpreted as through.
        let (Some(start), Some(end)) = (&layer.explicit_start_layer, &layer.explicit_end_layer)
        else {
            return Err(error(&hit, "ODB_HOLE_UNQUALIFIED_SPAN"));
        };
        let first = *copper
            .get(start.as_str())
            .ok_or_else(|| error(&hit, "ODB_HOLE_UNQUALIFIED_SPAN"))?;
        let last = *copper
            .get(end.as_str())
            .ok_or_else(|| error(&hit, "ODB_HOLE_UNQUALIFIED_SPAN"))?;
        let span = PhysicalLayerSpan { first, last };
        span.validate(copper_count)
            .map_err(|_| error(&hit, "ODB_HOLE_INVALID_SPAN"))?;
        // Tool subtypes are not sufficient to reconstruct backdrill depth,
        // remaining barrels, dual-diameter steps, or stacked transitions.
        if !layer.subtype.is_empty() {
            return Err(error(&hit, "ODB_HOLE_SPECIAL_OPERATION_UNSUPPORTED"));
        }
        let tool = layer
            .tools
            .iter()
            .find(|t| t.number == hit.tool_number)
            .ok_or_else(|| error(&hit, "ODB_HOLE_UNKNOWN_TOOL"))?;
        let stated_diameter = tool.drill_size.as_deref().unwrap_or(&tool.finish_size);
        if stated_diameter == "-1" {
            return Err(error(&hit, "ODB_HOLE_MISSING_DIAMETER"));
        }
        let mode = match layer.unit.as_str() {
            "MM" => "M",   // ODB tools use micrometres for MM
            "INCH" => "I", // ODB tools use mils for INCH
            _ => return Err(error(&hit, "ODB_HOLE_BAD_UNITS")),
        };
        let diameter_m = crate::odb_source::length_symbol(
            stated_diameter,
            mode,
            &layer.source_file,
            tool.source_line,
        )?;
        let plating = match hit.tool_intent {
            DrillIntent::Plated => FabricationPlating::SourceDeclaredPlated,
            DrillIntent::NonPlated => FabricationPlating::NonPlated,
            DrillIntent::Via => FabricationPlating::Unknown,
        };
        let fabrication = FabricationHole {
            shape: FabricationHoleShape::Round { diameter_m },
            plating,
            span,
            operation: FabricationOperation::Drilled,
            evidence: FabricationSourceEvidence {
                format: "ODB++Design-8.1-U4-Drill-P".into(),
                source_sha256: hit.source_sha256,
                source_line: hit.source_line,
                qualified_native_grammar: true,
            },
        };
        fabrication
            .validate(copper_count)
            .map_err(|_| error(&hit, "ODB_HOLE_INVALID_FABRICATION"))?;
        debug_assert!(!fabrication.is_electrical_conductor());
        out.push(OdbSourceHole {
            xy_m: xy_m(&hit)?,
            fabrication,
            hit,
        });
    }
    Ok(out)
}

/// E_019: derive one bounded candidate kind from the already validated
/// manufacturing-hole observations. Unsupported/unknown intent remains in
/// source_holes; it is never promoted by fallback or topology assumptions.
fn source_plated_candidates(
    board: &BoardModel,
    holes: &[OdbSourceHole],
    layers: &[SourceDrillLayer],
) -> Result<Vec<OdbPlatedViaCandidate>, OdbError> {
    let mut out = Vec::new();
    let source = &board.sources[0].id; // validated ODB importer returns one source
    for hole in holes {
        let hit = &hole.hit;
        if hit.tool_intent != DrillIntent::Plated || hit.tool_subtype != "STANDARD" {
            continue;
        }
        let layer = layers.iter().find(|l| l.name == hit.layer_name)
            .ok_or_else(|| error(hit, "ODB_VIA_CANDIDATE_LAYER_MISSING"))?;
        let tool = layer.tools.iter().find(|t| t.number == hit.tool_number)
            .ok_or_else(|| error(hit, "ODB_VIA_CANDIDATE_TOOL_MISSING"))?;
        if tool.intent != DrillIntent::Plated || tool.subtype != "STANDARD" {
            return Err(error(hit, "ODB_VIA_CANDIDATE_TOOL_CONFLICT"));
        }
        // Unknown FINISH_SIZE must stay unknown even when DRILL_SIZE is known.
        if tool.finish_size == "-1" {
            continue;
        }
        let mode = match layer.unit.as_str() {
            "MM" => "M",
            "INCH" => "I",
            _ => return Err(error(hit, "ODB_VIA_CANDIDATE_BAD_UNITS")),
        };
        let finished_diameter_m = crate::odb_source::length_symbol(
            &tool.finish_size, mode, &layer.source_file, tool.source_line,
        )?;
        let tool_diameter_m = tool.drill_size.as_deref().map(|v| {
            crate::odb_source::length_symbol(v, mode, &layer.source_file, tool.source_line)
        }).transpose()?;
        // Bind the exact logical path and SHA for all THREE manufacturing
        // records; hash equality alone is ambiguous for repeated file content.
        for (path, sha) in [
            (hit.source_file.as_str(), hit.source_sha256),
            (hit.tool_source_file.as_str(), hit.tool_source_sha256),
            ("matrix/matrix", hit.matrix_source_sha256),
        ] {
            if source.files.iter().filter(|f| f.logical_path == path && f.sha256 == sha).count() != 1 {
                return Err(error(hit, "ODB_VIA_CANDIDATE_PROVENANCE_MISMATCH"));
            }
        }
        let observed = board.observe_source_via_hole(source, &hole.fabrication)
            .map_err(|_| error(hit, "ODB_VIA_CANDIDATE_SOURCE_OBSERVATION"))?;
        if observed.source_file != hit.source_file
            || observed.source_line != hit.source_line
            || observed.via_evidence != SourceViaEvidenceState::SourceDeclaredPlated
            || observed.stackup_evidence == SourceStackupEvidence::Explicit
                && board.stackup.layers.iter().any(|l| l.material.is_none())
        {
            return Err(error(hit, "ODB_VIA_CANDIDATE_EVIDENCE_CONFLICT"));
        }
        out.push(OdbPlatedViaCandidate {
            source_identity: source.clone(),
            candidate_key: format!(
                "odb-hole:{}:feature-{}:tool-{}",
                hit.source_file, hit.feature_index, hit.tool_number,
            ),
            xy_m: hole.xy_m,
            fabrication: hole.fabrication.clone(),
            finished_diameter_m,
            tool_diameter_m,
            source_observation: observed,
            hit: hit.clone(),
        });
    }
    Ok(out)
}

/// Derive qualified source-hole observations and Canonical copper from ONE
/// immutable source file map. Full unsupported inputs fail transactionally.
pub fn import_unpacked_with_holes(
    files: &OdbFileMap,
    step: &str,
    with_eda: bool,
) -> Result<OdbSourceImport, OdbError> {
    let board = if with_eda {
        import_unpacked_with_eda(files, step)?
    } else {
        import_unpacked(files, step)?
    };
    let layers = audit_unpacked_drill_tools(files, step)?;
    let hits = audit_unpacked_drill_hits(files, step)?;
    let holes = source_holes(&board, &layers, hits)?;
    let via_candidates = source_plated_candidates(&board, &holes, &layers)?;
    // DO NOT synthesize Canonical Drill/Via without complete padstack/process
    // evidence: neither source position nor net implies plated continuity.
    debug_assert!(board.drills.is_empty() && board.vias.is_empty());
    debug_assert!(board.connectivity.edges.is_empty());
    Ok(OdbSourceImport {
        board,
        source_holes: holes,
        via_candidates,
    })
}

/// Directory, USTAR, ZIP 0/8, or single GZIP+USTAR path, safely snapped once
/// through the same validated immutable source-file-map import route.
pub fn import_path_with_holes(
    path: &Path,
    step: &str,
    with_eda: bool,
) -> Result<OdbSourceImport, OdbError> {
    let files = crate::odb_source::read_source_path_file_map(path)?;
    import_unpacked_with_holes(&files, step, with_eda)
}

/// In-memory uncompressed USTAR input, without extraction or shell execution.
pub fn import_tar_with_holes(
    archive: &[u8],
    step: &str,
    with_eda: bool,
) -> Result<OdbSourceImport, OdbError> {
    let files = crate::odb_tar::read_tar(archive)?;
    import_unpacked_with_holes(&files, step, with_eda)
}
