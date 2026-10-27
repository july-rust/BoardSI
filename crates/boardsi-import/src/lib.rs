//! M1 format detection only. No CAD geometry, netlist, stackup or SI result is inferred.
//! The canonical BoardModel stays independent of all import-source signatures.

use std::io::{self, Read};

/// Maximum accepted first-line length. Prevent unbounded reads on hostile files.
pub const MAX_PADS_HEADER_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Signature {
    PadsPcbAscii,
    PadsSchematicAscii,
    Unknown,
}

/// Detect only the first line, never a matching token hidden later in a file.
///
/// A matching header proves a PADS-family *signature*, not a supported revision,
/// a valid PCB, parsed geometry or permission to run the solver.
pub fn probe_pads_header(input: &[u8]) -> Signature {
    let first = input
        .split(|&byte| byte == b'\n')
        .next()
        .unwrap_or_default();
    // A trailing CR is a terminator only when the following LF is visible.
    // Reject bare/boundary CR to avoid accepting truncated overlong first lines.
    let first = if let Some(stripped) = first.strip_suffix(b"\r") {
        if input.get(first.len()) != Some(&b'\n') {
            return Signature::Unknown;
        }
        stripped
    } else {
        first
    };
    // No embedded CR is permitted. CRLF is handled only at the line boundary.
    // In particular, "!PADS-POWERPCB-V9.5!\\rX\\n" is not a valid header.
    if first.contains(&b'\r') || first.len() > MAX_PADS_HEADER_BYTES {
        return Signature::Unknown;
    }
    // The exporter normally uses raw ASCII. Permit UTF-8 BOM without moving the
    // signature search beyond byte zero (after the BOM).
    let first = first.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(first);
    let Some(&delimiter) = first.first() else {
        return Signature::Unknown;
    };
    if delimiter != b'!' && delimiter != b'*' {
        return Signature::Unknown;
    }
    let Some(closing) = first[1..].iter().position(|&byte| byte == delimiter) else {
        return Signature::Unknown;
    };
    let tag = &first[1..closing + 1];
    fn starts_family(tag: &[u8], family: &[u8]) -> bool {
        tag.strip_prefix(family)
            .map(|tail| tail.is_empty() || tail.starts_with(b"-"))
            .unwrap_or(false)
    }
    if starts_family(tag, b"PADS-POWERPCB") || starts_family(tag, b"PADS2000") {
        Signature::PadsPcbAscii
    } else if starts_family(tag, b"PADS-POWERLOGIC") || starts_family(tag, b"PADS-LOGIC") {
        Signature::PadsSchematicAscii
    } else {
        Signature::Unknown
    }
}

/// Read at most 514 bytes: up to 512 header bytes plus CRLF lookahead.
/// Never scan the entire (possibly huge) PCB file. A bare CR is not a terminator.
pub fn probe_pads_reader<R: Read>(mut source: R) -> io::Result<Signature> {
    let mut prefix = Vec::with_capacity(MAX_PADS_HEADER_BYTES + 2);
    source
        .by_ref()
        .take((MAX_PADS_HEADER_BYTES + 2) as u64)
        .read_to_end(&mut prefix)?;
    Ok(probe_pads_header(&prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn accepts_pcb_export_signatures_without_parsing_geometry() {
        for header in [
            "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\r\n*PCB*\n",
            "*PADS-POWERPCB-V9.5*\n*LINES*\n",
            "!PADS2000-V9-MILS!\n",
            "\u{feff}!PADS-POWERPCB-V9.5-MM!\n",
        ] {
            assert_eq!(
                probe_pads_header(header.as_bytes()),
                Signature::PadsPcbAscii
            );
        }
    }

    #[test]
    fn rejects_schematics_as_pcb() {
        for header in ["*PADS-POWERLOGIC-V9.5*\n", "!PADS-LOGIC-V9.0-CP1250!\n"] {
            assert_eq!(
                probe_pads_header(header.as_bytes()),
                Signature::PadsSchematicAscii
            );
        }
    }

    #[test]
    fn rejects_truncation_spoofing_and_extension_only_matches() {
        for invalid in [
            "",
            "*VERSION* 4.0\nSHEET 1\n", // LTspice .asc, not PADS
            "noise !PADS-POWERPCB-V9.4-MILS!\n",
            "unknown\n!PADS-POWERPCB-V9.4-MILS!\n",
            "!PADS-POWERPCBEVIL-V9.4-MILS!\n",
            "!PADS2000EVIL!\n",
            "!PADS-POWERPCB-V9.5!\rX\n",
            "!PADS-POWERPCB-V9.4-MILS!\r*PCB*",
            "!PADS-POWERPCB-V9.5!\r*PCB*\n",
            "!PADS-POWERPCB-V9.4-MILS\n",
            "!PADS-POWERLOGIC",
            "!!PADS-POWERPCB-V9.4-MILS!",
        ] {
            assert_eq!(probe_pads_header(invalid.as_bytes()), Signature::Unknown);
        }
        let oversized = format!("!PADS-POWERPCB-{}!\n", "A".repeat(520));
        assert_eq!(probe_pads_header(oversized.as_bytes()), Signature::Unknown);
    }

    #[test]
    fn bounds_streaming_reads_even_for_huge_data() {
        let mut file = b"!PADS-POWERPCB-V9.4-MILS!\n".to_vec();
        file.extend(std::iter::repeat(b'X').take(2_000_000));
        let mut reader = Cursor::new(file);
        assert_eq!(
            probe_pads_reader(&mut reader).unwrap(),
            Signature::PadsPcbAscii
        );
        assert!(reader.position() <= (MAX_PADS_HEADER_BYTES + 2) as u64);
    }

    #[test]
    fn handles_exact_length_crlf_and_rejects_truncated_boundary_cr() {
        let tag = b"!PADS-POWERPCB-V9.5!";
        let mut header = tag.to_vec();
        header.resize(MAX_PADS_HEADER_BYTES, b' ');
        assert_eq!(header.len(), 512);

        // 512 content bytes + visible CRLF require a 514-byte bounded read.
        for ending in [b"\n".as_slice(), b"\r\n".as_slice(), b"".as_slice()] {
            let mut file = header.clone();
            file.extend_from_slice(ending);
            let mut reader = Cursor::new(file);
            assert_eq!(
                probe_pads_reader(&mut reader).unwrap(),
                Signature::PadsPcbAscii
            );
            assert!(reader.position() <= (MAX_PADS_HEADER_BYTES + 2) as u64);
        }

        // A CR on the boundary is not enough: do not strip CR until LF is seen.
        for ending in [
            b"\rXmore bytes\n".as_slice(),
            b"\r".as_slice(),
            b"X\r\n".as_slice(),
            b"XX\n".as_slice(),
        ] {
            let mut file = header.clone();
            file.extend_from_slice(ending);
            assert_eq!(probe_pads_header(&file), Signature::Unknown);
            assert_eq!(
                probe_pads_reader(Cursor::new(file)).unwrap(),
                Signature::Unknown
            );
        }
    }

    #[test]
    fn rejects_overlong_unterminated_first_line() {
        let mut reader = Cursor::new(vec![b'X'; 20_000]);
        assert_eq!(probe_pads_reader(&mut reader).unwrap(), Signature::Unknown);
    }
}

pub mod circle_cutout_topology;
pub mod connectivity_preview;
pub mod copper_arc_preflight;
pub mod copper_contact;
pub mod copper_preview;
pub mod curved_cutout_topology;
pub mod digest;
pub mod outline_arc;
pub mod outline_preview;
pub mod outline_topology;
pub mod part_preview;
pub mod polygon_topology;
pub mod pour_preview;
pub mod reference_geometry;
pub mod route_preview;

// A_032_2 opt-in local endpoint-evidence wire adapter.
pub mod contact_wire;
