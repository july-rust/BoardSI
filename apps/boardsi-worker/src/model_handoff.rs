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

/// Model borrow must remain live through the Worker client's destruction.
///
/// ```compile_fail
/// use boardsi_model::{BoardKind, BoardModel};
/// use boardsi_ipc::Snapshot;
/// use boardsi_worker::model_handoff::ModelBoundSyntheticClient;
/// let mut board = BoardModel::empty(BoardKind::Pcb);
/// let active = ModelBoundSyntheticClient::spawn(
///     std::path::Path::new("worker"), "test", &board,
///     Snapshot { points: 1, delay_ms: 0 }
/// ).unwrap();
/// active.state(); // even if this is the last explicit use
/// board.nets.clear(); // forbidden until active client is dropped
/// ```
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
        // The synthetic Worker emits DC .. (points-1)*1 MHz, inclusive.
        // Structural validation alone does not check required layer materials,
        // assumed property provenance or their valid frequency bands.
        for frequency_hz in [0.0, f64::from(snapshot.points - 1) * 1e6] {
            let readiness = model.validate_materials_at(frequency_hz);
            if readiness.has_errors()
                || readiness.has_missing()
                || readiness
                    .issues
                    .iter()
                    .any(|issue| issue.severity == Severity::Assumption)
            {
                return Err(invalid("MaterialReadinessFailed"));
            }
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

impl Drop for ModelBoundSyntheticClient<'_> {
    fn drop(&mut self) {
        // The referenced model must remain borrowed until the Worker client
        // is destroyed, not merely until the last explicit call to state().
        // A real Drop impl ensures this bound is enforced by drop checking.
        let _model_lifetime = self.model;
    }
}
