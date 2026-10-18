//! M0-only model-gated synthetic Worker handoff.
//!
//! Not a PCB geometry serializer, solver-ready run or physical validation.
//! The IPC v1 Snapshot still contains only synthetic points/delay; it does not
//! cryptographically bind the model or transfer copper/net topology to Worker.
//! Keep the validated model immutably borrowed for the lifetime of this client.
use crate::Client;
use boardsi_ipc::{Message, Snapshot, State};
use boardsi_model::{BoardModel, Severity, SourceFormat};
use std::{io, path::Path};

pub struct ModelBoundSyntheticClient<'a> {
    client: Client,
    model: &'a BoardModel,
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
        if report.has_errors()
            || report.has_missing()
            || report
                .issues
                .iter()
                .any(|issue| issue.severity == Severity::Assumption)
        {
            return Err(invalid("ModelValidationFailed"));
        }
        let client = Client::spawn(path, None, run_id, snapshot)?;
        Ok(Self { client, model })
    }
    /// Read-only reference preserving the validated model lifetime.
    pub fn model(&self) -> &BoardModel {
        self.model
    }

    /// Bounded nonblocking reception; never exposes the owned Client.
    pub fn poll(&mut self) -> io::Result<Vec<Message>> {
        self.client.poll()
    }

    pub fn cancel(&mut self) -> io::Result<()> {
        self.client.cancel()
    }

    pub fn force_kill(&mut self) -> io::Result<()> {
        self.client.force_kill()
    }

    pub fn state(&self) -> State {
        self.client.guard.state
    }

    pub fn result_count(&self) -> usize {
        self.client.guard.results.len()
    }

    pub fn exit_success(&self) -> Option<bool> {
        self.client.exit.as_ref().map(|status| status.success())
    }
}
