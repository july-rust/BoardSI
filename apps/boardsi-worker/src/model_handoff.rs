//! M0-only model-gated synthetic Worker handoff.
//!
//! Not a PCB geometry serializer, solver-ready run or physical validation.
//! The IPC v1 Snapshot still contains only synthetic points/delay; it does not
//! cryptographically bind the model or transfer copper/net topology to Worker.
//! Keep the validated model immutably borrowed for the lifetime of this client.
use crate::Client;
use boardsi_ipc::Snapshot;
use boardsi_model::{BoardModel, SourceFormat};
use std::{io, path::Path};

pub struct ModelBoundSyntheticClient<'a> {
    pub client: Client,
    pub model: &'a BoardModel,
}

impl<'a> ModelBoundSyntheticClient<'a> {
    /// Reject invalid, empty, non-synthetic or incomplete models BEFORE spawning.
    /// This is a structural/readiness gate, not electromagnetic sign-off.
    pub fn spawn(
        path: &Path,
        run_id: &str,
        model: &'a BoardModel,
        snapshot: Snapshot,
    ) -> io::Result<Self> {
        let invalid = |why: &str| io::Error::new(io::ErrorKind::InvalidInput, why);
        snapshot.validate().map_err(invalid)?;
        if run_id.is_empty() || run_id.len() > 128 {
            return Err(invalid("InvalidRunId"));
        }
        if model.sources.len() != 1
            || model.sources[0].id.format != SourceFormat::Synthetic
            || model.stackup.layers.is_empty()
            || model.nets.is_empty()
        {
            return Err(invalid("UnsupportedOrEmptySyntheticModel"));
        }
        let report = model.validate();
        if report.has_errors() || report.has_missing() {
            return Err(invalid("ModelValidationFailed"));
        }
        let client = Client::spawn(path, None, run_id, snapshot)?;
        Ok(Self { client, model })
    }
}
