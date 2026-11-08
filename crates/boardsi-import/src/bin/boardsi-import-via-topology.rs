//! Opt-in PADS V9.4 MILS source-backed via topology diagnostics.
//! This is a separate wire format from Qt Artwork V1-V14; no electrical/SI claim.
use boardsi_import::route_preview::{into_canonical, parse_generalized, MAX_FILE_BYTES};
use boardsi_import::trace_attachment::{source_candidates_generalized, Target};
use boardsi_import::via_topology::NormalizedVia;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::{env, fs};

fn run() -> Result<Vec<u8>, &'static str> {
    let args: Vec<_> = env::args_os().collect();
    let with_contacts = args.len() == 3 && args[2] == "--candidate-contacts";
    if args.len() != 2 && !with_contacts {
        return Err("USAGE: boardsi-import-via-topology FILE.asc [--candidate-contacts]");
    }
    let path = &args[1];
    let metadata = fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES as u64 {
        return Err("INVALID_FILE_SIZE");
    }
    let bytes = fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
    let parsed = parse_generalized(&bytes)?;
    if parsed.skipped != 0 {
        return Err("D056_UNSUPPORTED_SOURCE_ROUTE_INVENTORY");
    }
    let canonical = into_canonical(&bytes, &parsed)?;
    if canonical.validate().has_errors() || canonical.vias.len() != parsed.vias.len() {
        return Err("D056_CANONICAL_VIA_MISMATCH");
    }
    let mut out = String::new();
    writeln!(
        out,
        "BOARDSI_SOURCE_VIA_TOPOLOGY_V1|V9.4-MILS|{}|{}|GEOMETRY_ONLY|SI_NOT_RUN",
        parsed.max_layer,
        parsed.vias.len()
    )
    .map_err(|_| "D056_WIRE_ERROR")?;
    for (index, via) in parsed.vias.iter().enumerate() {
        let normalized = if let Some(topology) = &via.topology {
            topology.clone()
        } else {
            NormalizedVia::pads_v94_through(
                parsed.max_layer,
                via.drill_ticks,
                via.diameter_ticks,
                via.inner_pad.then_some(via.diameter_ticks),
                via.diameter_ticks,
            )?
        };
        normalized.validate()?;
        writeln!(
            out,
            "V|{}|{}|{}|{}|{}|{}|{}|{}|{:?}|{}|{}|{}",
            index,
            via.name,
            via.net,
            via.source_line,
            via.point[0],
            via.point[1],
            normalized.first_layer,
            normalized.last_layer,
            normalized.kind,
            normalized.drill_ticks,
            normalized.plated,
            normalized.pads.len()
        )
        .map_err(|_| "D056_WIRE_ERROR")?;
        for pad in &normalized.pads {
            writeln!(out, "P|{}|{}|R|{}", index, pad.layer, pad.diameter_ticks)
                .map_err(|_| "D056_WIRE_ERROR")?;
        }
    }
    if with_contacts {
        for candidate in source_candidates_generalized(&bytes)? {
            if let Target::Via(index) = candidate.target {
                writeln!(
                    out,
                    "C|{}|{}|{}|{:?}|SOURCE_LOCAL_CANDIDATE_ONLY",
                    candidate.trace_index,
                    candidate.trace_source_line,
                    index,
                    candidate.net_relation
                )
                .map_err(|_| "D056_WIRE_ERROR")?;
            }
        }
    }
    writeln!(out, "END|{}|SOURCE_PROVENANCE_ONLY|GLOBAL_CONNECTIVITY_NOT_VERIFIED", parsed.vias.len())
        .map_err(|_| "D056_WIRE_ERROR")?;
    if out.len() > 1_000_000 {
        return Err("D056_OUTPUT_LIMIT");
    }
    Ok(out.into_bytes())
}

fn main() {
    match run() {
        Ok(out) => {
            if std::io::stdout().write_all(&out).is_err() {
                eprintln!("D056_STDOUT_FAILED");
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
