//! F_008: typed, exact Excellon drill *observations* preserved in SQLite v1
//! required extension. Never interpret these as plated drills, spans or vias.
use crate::{invalid, Result};
use boardsi_model::{BoardModel, SourceFormat};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const EXT_NAME: &str = "org.boardsi.source.gerber-excellon-drill";
pub const EXT_VERSION: i64 = 1;
const MAGIC: &[u8; 8] = b"BDRL0001";
const MAX_HOLES: usize = 128;
const HEADER: usize = 8 + 4 + 32;
const RECORD: usize = 36;
const CHECKSUM: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrillObservation {
    pub centre_nm: [i64; 2],
    pub diameter_nm: i64,
    pub tool_id: u32,
    pub source_offset: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GerberDrillEvidence {
    pub drill_sha256: [u8; 32],
    pub holes: Vec<DrillObservation>,
}
impl GerberDrillEvidence {
    pub fn check(&self, board: &BoardModel) -> Result<()> {
        if self.holes.is_empty()
            || self.holes.len() > MAX_HOLES
            || board.sources.len() != 2
            || board.stackup.layers.len() != 1
            || board.pads.is_empty()
            || board.pads.len() > 128
            || board.pads.len() + self.holes.len() > 128
            || board.kind != boardsi_model::BoardKind::Pcb
            || !board.materials.is_empty()
            || !board.nets.is_empty()
            || !board.components.is_empty()
            || !board.pins.is_empty()
            || !board.outlines.is_empty()
            || !board.traces.is_empty()
            || !board.arcs.is_empty()
            || !board.areas.is_empty()
            || !board.drills.is_empty()
            || !board.vias.is_empty()
            || !board.connectivity.nodes.is_empty()
            || !board.connectivity.edges.is_empty()
        {
            return Err(invalid("F008 unsupported source-only board inventory"));
        }
        let gerber = board
            .sources
            .iter()
            .find(|s| matches!(&s.id.format, SourceFormat::Gerber))
            .ok_or_else(|| invalid("F008 missing Gerber source"))?;
        let drill = board
            .sources
            .iter()
            .find(|s| matches!(&s.id.format, SourceFormat::Other(s) if s == "Excellon"))
            .ok_or_else(|| invalid("F008 missing Excellon source"))?;
        if drill.id.files.len() != 1
            || gerber.id.files.len() != 1
            || drill.id.files[0].sha256 != self.drill_sha256
            || !gerber
                .id
                .conditions
                .iter()
                .any(|(k, v)| k == "scope" && v == "f008-gerber-x2-flash-excellon-source-only")
            || !drill
                .id
                .conditions
                .iter()
                .any(|(k, v)| k == "scope" && v == "f008-gerber-x2-flash-excellon-source-only")
            || board.stackup.layers[0].meta.id.source.as_ref() != gerber.id.as_ref()
            || board
                .pads
                .iter()
                .any(|p| p.meta.id.source.as_ref() != gerber.id.as_ref() || p.net.is_some())
            || board
                .padstacks
                .iter()
                .any(|p| p.meta.id.source.as_ref() != gerber.id.as_ref())
            || board
                .sources
                .iter()
                .any(|s| s.capability.net != boardsi_model::CapabilityState::Missing)
        {
            return Err(invalid("F008 source/geometry provenance"));
        }
        // Source-qualified positive D03 circular flashes cannot silently
        // become an AntiPad, a rectangle, or a different Copper layer in a
        // modified native file. Such a change is not this required extension.
        use boardsi_model::{LayerKind, PadRole, PadShape};
        let layer = &board.stackup.layers[0];
        if layer.kind != LayerKind::Copper
            || !matches!(&layer.z_m, boardsi_model::Property::Missing)
            || !matches!(&layer.thickness_m, boardsi_model::Property::Missing)
            || layer.material.is_some()
            || board.padstacks.is_empty()
            || board.padstacks.iter().any(|stack| {
                stack.entries.len() != 1
                    || stack.entries[0].layer != layer.meta.id
                    || stack.entries[0].role != PadRole::Conductive
                    || !matches!(
                        &stack.entries[0].shape,
                        PadShape::Circle { diameter_m }
                            if diameter_m.numerator() > 0
                                && diameter_m.numerator() <= diameter_m.denominator()
                    )
                    || !board.pads.iter().any(|pad| pad.padstack == stack.meta.id)
            })
        {
            return Err(invalid("F008 unsupported positive flash geometry"));
        }
        // F_008 stores raw integer-nm pad ticks. A valid but transformed
        // CoordinateFrame would otherwise make Qt flash/drill overlay incorrect.
        use boardsi_model::{Orientation, QuarterTurn, Rational};
        let frame = board
            .frames
            .first()
            .ok_or_else(|| invalid("F008 missing frame"))?;
        let nm = Rational::new(1, 1_000_000_000).map_err(|_| invalid("F008 unit definition"))?;
        if board.frames.len() != 1
            || frame.meta.id.source.as_ref() != gerber.id.as_ref()
            || frame.metres_per_tick != nm
            || frame.origin_m != [Rational::integer(0); 2]
            || !matches!(
                &frame.orientation,
                Orientation::Orthogonal {
                    turn: QuarterTurn::Zero,
                    mirror_x: false
                }
            )
            || board.pads.iter().any(|p| {
                p.position
                    .ticks
                    .iter()
                    .any(|v| v.unsigned_abs() > 1_000_000_000_000)
                    || p.position.frame != frame.meta.id
                    || !matches!(
                        &p.orientation,
                        Orientation::Orthogonal {
                            turn: QuarterTurn::Zero,
                            mirror_x: false
                        }
                    )
            })
        {
            return Err(invalid("F008 unsupported coordinate frame/transform"));
        }
        let mut tool_diameters = BTreeMap::new();
        let mut previous = None;
        for h in &self.holes {
            if h.diameter_nm <= 0
                || h.diameter_nm > 1_000_000_000
                || h.tool_id == 0
                || h.source_offset == 0
                || h.centre_nm
                    .iter()
                    .any(|x| x.unsigned_abs() > 1_000_000_000_000)
                || previous.is_some_and(|p| p >= h.source_offset)
            {
                return Err(invalid("F008 invalid drill observation"));
            }
            previous = Some(h.source_offset);
            if let Some(d) = tool_diameters.insert(h.tool_id, h.diameter_nm) {
                if d != h.diameter_nm {
                    return Err(invalid("F008 tool has two diameters"));
                }
            }
        }
        Ok(())
    }
    pub fn encode(&self, board: &BoardModel) -> Result<Vec<u8>> {
        self.check(board)?;
        let mut bytes = Vec::with_capacity(HEADER + self.holes.len() * RECORD + CHECKSUM);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&(self.holes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.drill_sha256);
        for h in &self.holes {
            bytes.extend_from_slice(&h.centre_nm[0].to_le_bytes());
            bytes.extend_from_slice(&h.centre_nm[1].to_le_bytes());
            bytes.extend_from_slice(&h.diameter_nm.to_le_bytes());
            bytes.extend_from_slice(&h.tool_id.to_le_bytes());
            bytes.extend_from_slice(&h.source_offset.to_le_bytes());
        }
        let hash = Sha256::digest(&bytes);
        bytes.extend_from_slice(&hash);
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8], board: &BoardModel) -> Result<Self> {
        if bytes.len() < HEADER + RECORD + CHECKSUM || bytes.get(..8) != Some(MAGIC.as_slice()) {
            return Err(invalid("F008 extension header"));
        }
        let count = u32::from_le_bytes(bytes[8..12].try_into().expect("slice")) as usize;
        if count == 0
            || count > MAX_HOLES
            || bytes.len() != HEADER + count * RECORD + CHECKSUM
            || Sha256::digest(&bytes[..bytes.len() - CHECKSUM]).as_slice()
                != &bytes[bytes.len() - CHECKSUM..]
        {
            return Err(invalid("F008 extension length/hash"));
        }
        let mut drill_sha256 = [0u8; 32];
        drill_sha256.copy_from_slice(&bytes[12..44]);
        let mut holes = Vec::with_capacity(count);
        for i in 0..count {
            let b = &bytes[HEADER + i * RECORD..HEADER + (i + 1) * RECORD];
            holes.push(DrillObservation {
                centre_nm: [
                    i64::from_le_bytes(b[0..8].try_into().expect("slice")),
                    i64::from_le_bytes(b[8..16].try_into().expect("slice")),
                ],
                diameter_nm: i64::from_le_bytes(b[16..24].try_into().expect("slice")),
                tool_id: u32::from_le_bytes(b[24..28].try_into().expect("slice")),
                source_offset: u64::from_le_bytes(b[28..36].try_into().expect("slice")),
            });
        }
        let value = Self {
            drill_sha256,
            holes,
        };
        value.check(board)?;
        Ok(value)
    }
}
