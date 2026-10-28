//! C_003: versioned, immutable PROVISIONAL solver-input identity and stale-result contract.
//!
//! Does not serialize/verify the full Canonical BoardModel. Source/model SHA fields
//! are caller claims: no independent source authentication or physical qualification.
//! No SI solver, P009, physical cache reuse, or result accuracy is implied.
use crate::digest::sha256;

pub const SCHEMA: &str = "boardsi-c003-provisional-input-v1";
const CONDITIONS_DOMAIN: &[u8] = b"BoardSI C003 conditions v1\0";
const COMPAT_DOMAIN: &[u8] = b"BoardSI C003 compatibility v1\0";
const SNAPSHOT_DOMAIN: &[u8] = b"BoardSI C003 snapshot v1\0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioDraft {
    /// SHA-256 supplied by the caller, NOT independently authenticated by this contract.
    pub source_sha256: String,
    /// Full canonical model digest is not yet available for arbitrary PADS objects.
    pub canonical_model_sha256: String,
    pub geometry_revision: String,
    pub stackup_revision: String,
    pub material_provenance_revision: String,
    pub signal_net: String,
    pub reference_net: String,
    pub port_load_revision: String,
    pub group_coupling_revision: String,
    pub solver_model_revision: String,
    pub engine_version: String,
    pub backend_precision: String,
    /// Explicit direct sample frequencies in Hz. Sorted, unique, finite integer samples.
    pub frequencies_hz: Vec<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioError {
    MissingInput,
    InvalidDigest,
    OversizedInput,
    InvalidFrequencyGrid,
    SameSignalReference,
    InvalidRun,
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn append_field(dst: &mut Vec<u8>, value: &str) {
    dst.extend_from_slice(&(value.len() as u32).to_be_bytes());
    dst.extend_from_slice(value.as_bytes());
}

fn digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut data = Vec::with_capacity(domain.len() + bytes.len());
    data.extend_from_slice(domain);
    data.extend_from_slice(bytes);
    sha256(&data).iter().map(|b| format!("{b:02x}")).collect()
}

/// Owned, immutable snapshot, NOT an authorization to invoke a physical solver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenScenario {
    validated_inputs: ScenarioDraft,
    conditions_sha256: String,
    compatibility_sha256: String,
    snapshot_sha256: String,
    frequencies_hz: Vec<u64>,
}

impl FrozenScenario {
    /// Immutable view of the exact accepted draft fields; caller can only clone and edit a copy.
    pub fn input(&self) -> &ScenarioDraft {
        &self.validated_inputs
    }

    pub fn conditions_sha256(&self) -> &str {
        &self.conditions_sha256
    }

    pub fn compatibility_sha256(&self) -> &str {
        &self.compatibility_sha256
    }

    pub fn snapshot_sha256(&self) -> &str {
        &self.snapshot_sha256
    }

    pub fn frequencies_hz(&self) -> &[u64] {
        &self.frequencies_hz
    }

    /// Always false until a separately qualified complete physical snapshot is implemented.
    pub fn solver_ready(&self) -> bool {
        false
    }

    /// This contract does not create, validate, or permit physical result cache reuse.
    pub fn cache_reuse_eligible(&self) -> bool {
        false
    }
}

impl ScenarioDraft {
    pub fn freeze(&self) -> Result<FrozenScenario, ScenarioError> {
        if !valid_digest(&self.source_sha256) || !valid_digest(&self.canonical_model_sha256) {
            return Err(ScenarioError::InvalidDigest);
        }
        if self.signal_net == self.reference_net {
            return Err(ScenarioError::SameSignalReference);
        }
        let texts = [
            &self.geometry_revision,
            &self.stackup_revision,
            &self.material_provenance_revision,
            &self.signal_net,
            &self.reference_net,
            &self.port_load_revision,
            &self.group_coupling_revision,
            &self.solver_model_revision,
            &self.engine_version,
            &self.backend_precision,
        ];
        if texts
            .iter()
            .any(|v| v.trim().is_empty() || v.contains('\0'))
        {
            return Err(ScenarioError::MissingInput);
        }
        if texts.iter().any(|v| v.len() > 512) {
            return Err(ScenarioError::OversizedInput);
        }
        if self.frequencies_hz.is_empty()
            || self.frequencies_hz.len() > 4096
            || self.frequencies_hz.iter().any(|&f| f > 30_000_000_000)
            || self.frequencies_hz.windows(2).any(|v| v[0] >= v[1])
        {
            return Err(ScenarioError::InvalidFrequencyGrid);
        }

        // Length-prefix every field so concatenations cannot collide.
        // Conditions include solver and backend compatibility, but exclude sweep.
        let mut condition = Vec::new();
        for value in [
            &self.source_sha256,
            &self.canonical_model_sha256,
            &self.geometry_revision,
            &self.stackup_revision,
            &self.material_provenance_revision,
            &self.signal_net,
            &self.reference_net,
            &self.port_load_revision,
            &self.group_coupling_revision,
            &self.solver_model_revision,
            &self.engine_version,
            &self.backend_precision,
        ] {
            append_field(&mut condition, value);
        }
        let conditions_sha256 = digest(CONDITIONS_DOMAIN, &condition);
        let mut compatibility = Vec::new();
        for value in [
            &self.solver_model_revision,
            &self.engine_version,
            &self.backend_precision,
        ] {
            append_field(&mut compatibility, value);
        }
        let compatibility_sha256 = digest(COMPAT_DOMAIN, &compatibility);
        let mut run = Vec::new();
        append_field(&mut run, SCHEMA);
        append_field(&mut run, &conditions_sha256);
        run.extend_from_slice(&(self.frequencies_hz.len() as u32).to_be_bytes());
        for frequency in &self.frequencies_hz {
            run.extend_from_slice(&frequency.to_be_bytes());
        }
        let snapshot_sha256 = digest(SNAPSHOT_DOMAIN, &run);
        Ok(FrozenScenario {
            validated_inputs: self.clone(),
            conditions_sha256,
            compatibility_sha256,
            snapshot_sha256,
            frequencies_hz: self.frequencies_hz.clone(),
        })
    }
}

/// These states describe provenance/freshness only, never numerical correctness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultFreshness {
    NotRun,
    Current,
    OutOfDate,
    Incompatible,
    Rejected,
}

/// Receipt absence is not a successful or failed physics result.
pub fn classify_result(
    receipt: Option<&ResultReceipt>,
    current: &FrozenScenario,
) -> ResultFreshness {
    match receipt {
        Some(receipt) => receipt.freshness(current),
        None => ResultFreshness::NotRun,
    }
}

/// A provisional synthetic result envelope; public fields model untrusted transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResultReceipt {
    pub schema: String,
    pub run_id: String,
    pub snapshot_sha256: String,
    pub conditions_sha256: String,
    pub compatibility_sha256: String,
    pub sequence: u64,
    pub frequency_hz: u64,
    pub synthetic: bool,
}

impl ResultReceipt {
    /// Check against a current draft snapshot; caller must separately guard run/sequence.
    pub fn freshness(&self, current: &FrozenScenario) -> ResultFreshness {
        if self.schema != SCHEMA
            || !self.synthetic
            || self.run_id.is_empty()
            || self.sequence == 0
            || !valid_digest(&self.snapshot_sha256)
            || !valid_digest(&self.conditions_sha256)
            || !valid_digest(&self.compatibility_sha256)
        {
            ResultFreshness::Rejected
        } else if self.compatibility_sha256 != current.compatibility_sha256 {
            ResultFreshness::Incompatible
        } else if self.snapshot_sha256 != current.snapshot_sha256
            || self.conditions_sha256 != current.conditions_sha256
        {
            ResultFreshness::OutOfDate
        } else if !current.frequencies_hz.contains(&self.frequency_hz) {
            ResultFreshness::Rejected
        } else {
            ResultFreshness::Current
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiptError {
    WrongRun,
    WrongSnapshot,
    WrongSchema,
    NonSynthetic,
    InvalidSequence,
    InvalidFrequency,
    InvalidIdentity,
}

/// One-run ordered receipt gate. Rejected messages do not advance its cursor.
pub struct ResultScope {
    run_id: String,
    frozen: FrozenScenario,
    accepted: usize,
}

impl ResultScope {
    pub fn new(run_id: &str, frozen: FrozenScenario) -> Result<Self, ScenarioError> {
        if run_id.is_empty()
            || run_id.len() > 128
            || !run_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_:".contains(&b))
        {
            return Err(ScenarioError::InvalidRun);
        }
        Ok(Self {
            run_id: run_id.to_owned(),
            frozen,
            accepted: 0,
        })
    }

    pub fn accepted(&self) -> usize {
        self.accepted
    }

    pub fn snapshot(&self) -> &FrozenScenario {
        &self.frozen
    }

    pub fn accept(&mut self, receipt: &ResultReceipt) -> Result<(), ReceiptError> {
        if receipt.schema != SCHEMA {
            return Err(ReceiptError::WrongSchema);
        }
        if !receipt.synthetic {
            return Err(ReceiptError::NonSynthetic);
        }
        if receipt.run_id != self.run_id {
            return Err(ReceiptError::WrongRun);
        }
        if !valid_digest(&receipt.snapshot_sha256)
            || !valid_digest(&receipt.conditions_sha256)
            || !valid_digest(&receipt.compatibility_sha256)
        {
            return Err(ReceiptError::InvalidIdentity);
        }
        if receipt.snapshot_sha256 != self.frozen.snapshot_sha256
            || receipt.conditions_sha256 != self.frozen.conditions_sha256
            || receipt.compatibility_sha256 != self.frozen.compatibility_sha256
        {
            return Err(ReceiptError::WrongSnapshot);
        }
        if receipt.sequence != self.accepted as u64 + 1 {
            return Err(ReceiptError::InvalidSequence);
        }
        if self.frozen.frequencies_hz.get(self.accepted) != Some(&receipt.frequency_hz) {
            return Err(ReceiptError::InvalidFrequency);
        }
        self.accepted += 1;
        Ok(())
    }

    /// Test-only generator: cannot be used as a physical Solver receipt.
    pub fn synthetic_receipt(&self) -> Option<ResultReceipt> {
        self.frozen
            .frequencies_hz
            .get(self.accepted)
            .map(|&frequency_hz| ResultReceipt {
                schema: SCHEMA.to_owned(),
                run_id: self.run_id.clone(),
                snapshot_sha256: self.frozen.snapshot_sha256.clone(),
                conditions_sha256: self.frozen.conditions_sha256.clone(),
                compatibility_sha256: self.frozen.compatibility_sha256.clone(),
                sequence: self.accepted as u64 + 1,
                frequency_hz,
                synthetic: true,
            })
    }
}
