//! D_001 opt-in source-bound SMD Pad footprint / Trace endpoint witnesses.
//! A strict geometric candidate is NOT a manufactured contact, P009 or solver input.
use crate::{part_preview, route_preview};
use boardsi_model::BoardModel;
use std::fmt::Write;

const MAX_WITNESSES: usize = 10_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PadEndpointWitness {
    pub trace_index: usize,
    pub endpoint: usize,
    pub pad_index: usize,
    pub layer: u32,
    pub point: [i64; 2],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PadEndpointAudit {
    pub witnesses: Vec<PadEndpointWitness>,
    pub ambiguous_owners: usize,
    pub cross_net: usize,
    pub unproven_net: usize,
}

/// Exact source-tick *strict* inside predicate; tangency is never accepted.
/// It proves nonzero local geometric overlap with positive-width Trace only,
/// not manufactured pad contact or a whole-board electrical connection.
fn strictly_inside(point: [i64; 2], pad: &part_preview::ComponentPad) -> bool {
    if pad.width_ticks <= 0 || pad.height_ticks <= 0 {
        return false;
    }
    let dx = i128::from(point[0]) - i128::from(pad.point[0]);
    let dy = i128::from(point[1]) - i128::from(pad.point[1]);
    let w = i128::from(pad.width_ticks);
    let h = i128::from(pad.height_ticks);
    if pad.square || pad.rectangular {
        2 * dx.abs() < w && 2 * dy.abs() < h
    } else {
        // A round pad is supported only when its parsed width/height agree.
        w == h && 4 * (dx * dx + dy * dy) < w * w
    }
}

pub fn audit(source_bytes: &[u8]) -> Result<PadEndpointAudit, &'static str> {
    if source_bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D001_SOURCE_SIZE");
    }
    let source_route = route_preview::parse(source_bytes)?;
    let pads = part_preview::parse(source_bytes, source_route.max_layer);
    let mut board: BoardModel = route_preview::into_canonical(source_bytes, &source_route)?;
    part_preview::augment_source_bound(&mut board, &pads, source_bytes)?;
    if board.sources.len() != 1
        || board.sources[0].id.files.len() != 1
        || board.traces.len() != source_route.segments.len()
        || board.pads.len() != pads.pads.len()
        || board.validate().has_errors()
    {
        return Err("D001_SOURCE_CANONICAL_MISMATCH");
    }
    let mut report = PadEndpointAudit::default();
    for (trace_index, (seg, trace)) in source_route.segments.iter().zip(&board.traces).enumerate() {
        if seg.width_ticks <= 0 || trace.centreline.len() != 2 {
            continue;
        }
        for (endpoint, point) in [seg.a, seg.b].iter().enumerate() {
            let overlapping: Vec<usize> = pads
                .pads
                .iter()
                .enumerate()
                .filter(|(_, pad)| pad.layer == seg.layer && strictly_inside(*point, pad))
                .map(|(index, _)| index)
                .collect();
            if overlapping.is_empty() {
                continue;
            }
            // Multiple overlapping pads cannot be assigned one electrical owner.
            if overlapping.len() != 1 {
                report.ambiguous_owners += 1;
                continue;
            }
            let pad_index = overlapping[0];
            let canonical_pad = &board.pads[pad_index];
            let Some(pad_net) = &canonical_pad.net else {
                report.unproven_net += 1;
                continue;
            };
            if trace.net.as_ref() != Some(pad_net) {
                report.cross_net += 1;
                continue;
            }
            if report.witnesses.len() >= MAX_WITNESSES {
                return Err("D001_WITNESS_LIMIT");
            }
            report.witnesses.push(PadEndpointWitness {
                trace_index,
                endpoint,
                pad_index,
                layer: seg.layer,
                point: *point,
            });
        }
    }
    Ok(report)
}

/// Optional separate stdout protocol. No legacy artwork wire or Canonical edges
/// are changed; no candidate is accepted as a physical/electrical PASS.
pub fn encode(source_bytes: &[u8]) -> Result<String, &'static str> {
    let result = audit(source_bytes)?;
    let route = route_preview::parse(source_bytes)?;
    let pads = part_preview::parse(source_bytes, route.max_layer);
    let source = route_preview::into_canonical(source_bytes, &route)?;
    let file = source
        .sources
        .first()
        .ok_or("D001_MISSING_SOURCE")?
        .id
        .files
        .first()
        .ok_or("D001_MISSING_SOURCE_FILE")?;
    let mut sha = String::with_capacity(64);
    for b in &file.sha256 {
        write!(&mut sha, "{b:02x}").map_err(|_| "D001_WIRE_FORMAT")?;
    }
    let mut out = format!(
        "BOARDSI_D001_PAD_ENDPOINT_V1|{sha}|{}|SOURCE_GEOMETRY_ONLY|P009_UNSUPPORTED|SI_NOT_RUN\n",
        result.witnesses.len()
    );
    for w in &result.witnesses {
        let p = &pads.pads[w.pad_index];
        let shape = if p.rectangular {
            "RF"
        } else if p.square {
            "S"
        } else {
            "R"
        };
        writeln!(
            out,
            "W|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            w.trace_index,
            w.endpoint,
            w.pad_index,
            w.layer,
            w.point[0],
            w.point[1],
            p.width_ticks,
            p.height_ticks,
            shape,
            p.reference,
            p.pin
        )
        .map_err(|_| "D001_WIRE_FORMAT")?;
    }
    writeln!(
        out,
        "END|{}|SOURCE_CONTACT_CANDIDATE_ONLY",
        result.witnesses.len()
    )
    .map_err(|_| "D001_WIRE_FORMAT")?;
    if out.len() > 1024 * 1024 {
        return Err("D001_WIRE_LIMIT");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = include_str!("../tests/fixtures/pads_component_simple_v94.asc");
    fn specimen(replacement: &str) -> String {
        SAMPLE.replace("U9.1 U9.2\n100 200 1 10 0\n300 200 1 10 0", replacement)
    }
    #[test]
    fn round_smd_off_centre_is_new_strict_witness() {
        let s = specimen("U1.1 U9.2\n204 300 1 10 0\n300 300 1 10 0");
        let a = audit(s.as_bytes()).unwrap();
        assert_eq!(a.witnesses.len(), 1);
        assert_eq!(a.witnesses[0].point, [20_400, 30_000]);
        let wire = encode(s.as_bytes()).unwrap();
        assert!(wire.contains("|SOURCE_GEOMETRY_ONLY|P009_UNSUPPORTED|SI_NOT_RUN\n"));
        assert!(wire.contains("W|0|0|0|1|20400|30000|4000|4000|R|U1|1\n"));
    }
    #[test]
    fn round_pad_boundary_and_outside_rejected() {
        for x in [220, 221] {
            let s = specimen(&format!("U1.1 U9.2\n{x} 300 1 10 0\n300 300 1 10 0"));
            assert!(audit(s.as_bytes()).unwrap().witnesses.is_empty());
        }
    }
    #[test]
    fn unknown_or_different_net_is_not_promoted() {
        let no_declaration = specimen("U9.1 U9.2\n204 300 1 10 0\n300 300 1 10 0");
        let a = audit(no_declaration.as_bytes()).unwrap();
        assert!(a.witnesses.is_empty());
        assert!(a.unproven_net > 0);
        let different_net = specimen("U1.2 U9.2\n204 300 1 10 0\n300 300 1 10 0");
        assert!(audit(different_net.as_bytes())
            .unwrap()
            .witnesses
            .is_empty());
    }
    #[test]
    fn other_layer_and_nonoverlap_rejected() {
        let different_layer = specimen("U1.1 U9.2\n204 300 2 10 0\n300 300 2 10 0");
        assert!(audit(different_layer.as_bytes())
            .unwrap()
            .witnesses
            .is_empty());
        let far = specimen("U1.1 U9.2\n250 300 1 10 0\n300 300 1 10 0");
        assert!(audit(far.as_bytes()).unwrap().witnesses.is_empty());
    }
    #[test]
    fn deterministic_wire_and_source_digest_change() {
        let a = specimen("U1.1 U9.2\n204 300 1 10 0\n300 300 1 10 0");
        let b = specimen("U1.1 U9.2\n205 300 1 10 0\n300 300 1 10 0");
        assert_eq!(encode(a.as_bytes()).unwrap(), encode(a.as_bytes()).unwrap());
        assert_ne!(encode(a.as_bytes()).unwrap(), encode(b.as_bytes()).unwrap());
    }
}
