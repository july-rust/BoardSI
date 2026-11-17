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
        if self.holes.is_empty() || self.holes.len() > MAX_HOLES
            || board.sources.len() != 2 || board.stackup.layers.len() != 1
            || board.pads.is_empty() || board.pads.len() > 128
            || !board.traces.is_empty() || !board.arcs.is_empty()
            || !board.areas.is_empty() || !board.drills.is_empty()
            || !board.vias.is_empty() || !board.connectivity.nodes.is_empty()
            || !board.connectivity.edges.is_empty()
        {
            return Err(invalid("F008 unsupported source-only board inventory"));
        }
        let gerber = board.sources.iter().find(|s| matches!(s.id.format, SourceFormat::Gerber))
            .ok_or_else(|| invalid("F008 missing Gerber source"))?;
        let drill = board.sources.iter().find(|s| matches!(&s.id.format, SourceFormat::Other(s) if s == "Excellon"))
            .ok_or_else(|| invalid("F008 missing Excellon source"))?;
        if drill.id.files.len()!=1 || gerber.id.files.len()!=1
            || drill.id.files[0].sha256 != self.drill_sha256
            || !gerber.id.conditions.iter().any(|(k,v)| k=="scope" && v=="f008-gerber-x2-flash-excellon-source-only")
            || !drill.id.conditions.iter().any(|(k,v)| k=="scope" && v=="f008-gerber-x2-flash-excellon-source-only")
            || board.stackup.layers[0].meta.id.source.as_ref() != gerber.id.as_ref()
            || board.pads.iter().any(|p| p.meta.id.source.as_ref()!=gerber.id.as_ref() || p.net.is_some())
            || board.padstacks.iter().any(|p| p.meta.id.source.as_ref()!=gerber.id.as_ref())
            || board.sources.iter().any(|s| s.capability.net != boardsi_model::CapabilityState::Missing)
        {
            return Err(invalid("F008 source/geometry provenance"));
        }
        let mut tool_diameters = BTreeMap::new();
        let mut previous = None;
        for h in &self.holes {
            if h.diameter_nm <= 0 || h.diameter_nm > 1_000_000_000
                || h.tool_id==0 || h.source_offset==0
                || h.centre_nm.iter().any(|x| x.unsigned_abs()>1_000_000_000_000)
                || previous.is_some_and(|p| p >= h.source_offset)
            {
                return Err(invalid("F008 invalid drill observation"));
            }
            previous = Some(h.source_offset);
            if let Some(d)=tool_diameters.insert(h.tool_id,h.diameter_nm) {
                if d!=h.diameter_nm {return Err(invalid("F008 tool has two diameters"));}
            }
        }
        Ok(())
    }
    pub fn encode(&self, board: &BoardModel) -> Result<Vec<u8>> {
        self.check(board)?;
        let mut bytes = Vec::with_capacity(HEADER + self.holes.len()*RECORD + CHECKSUM);
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
        if bytes.len() < HEADER + RECORD + CHECKSUM || bytes.get(..8)!=Some(MAGIC.as_slice()) {
            return Err(invalid("F008 extension header"));
        }
        let count = u32::from_le_bytes(bytes[8..12].try_into().expect("slice")) as usize;
        if count==0 || count>MAX_HOLES ||
            bytes.len() != HEADER + count*RECORD + CHECKSUM ||
            Sha256::digest(&bytes[..bytes.len()-CHECKSUM]).as_slice() != &bytes[bytes.len()-CHECKSUM..] {
            return Err(invalid("F008 extension length/hash"));
        }
        let mut drill_sha256 = [0u8;32];
        drill_sha256.copy_from_slice(&bytes[12..44]);
        let mut holes=Vec::with_capacity(count);
        for i in 0..count {
            let b=&bytes[HEADER + i*RECORD .. HEADER + (i+1)*RECORD];
            holes.push(DrillObservation {
                centre_nm:[
                    i64::from_le_bytes(b[0..8].try_into().expect("slice")),
                    i64::from_le_bytes(b[8..16].try_into().expect("slice"))
                ],
                diameter_nm:i64::from_le_bytes(b[16..24].try_into().expect("slice")),
                tool_id:u32::from_le_bytes(b[24..28].try_into().expect("slice")),
                source_offset:u64::from_le_bytes(b[28..36].try_into().expect("slice")),
            });
        }
        let value=Self{drill_sha256,holes};
        value.check(board)?;
        Ok(value)
    }
}
