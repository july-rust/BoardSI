//! BoardSI native .bsi v1; all load paths are read-only and fail-closed.
mod bsig;
mod encoding;
mod read;
mod shared;
mod storage_validation;
mod store;
mod write;
pub use bsig::{Field, GeometryRecord};
pub use store::{load, save, LoadedBoard};

#[derive(Debug)]
pub enum NativeError {
    NotNative,
    FutureSchema,
    UnsupportedRequired,
    Invalid(&'static str),
    Io(std::io::Error),
    Sql(rusqlite::Error),
}
pub type Result<T> = std::result::Result<T, NativeError>;
impl NativeError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotNative => "BSI_NOT_NATIVE",
            Self::FutureSchema => "BSI_FUTURE_SCHEMA",
            Self::UnsupportedRequired => "BSI_UNSUPPORTED_REQUIRED",
            _ => "BSI_INVALID",
        }
    }
}
impl std::fmt::Display for NativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.code(), self)
    }
}
impl std::error::Error for NativeError {}
impl From<std::io::Error> for NativeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for NativeError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}
fn invalid(msg: &'static str) -> NativeError {
    NativeError::Invalid(msg)
}

/// Native storage retains exact data even if a solver f64 transform is not ready.
fn storage_valid(board: &boardsi_model::BoardModel) -> bool {
    use boardsi_model::{ModelError, Severity};
    let report = board.validate();
    let projection_incomplete = report.issues.iter().any(|issue| {
        issue.severity == Severity::Error
            && matches!(issue.error, ModelError::UnsupportedTransform | ModelError::PrecisionLoss)
    });
    // Contact evidence cannot be independently validated if frame projection is pending.
    // Reject that narrower case rather than accepting an unverified electrical graph.
    if projection_incomplete
        && (!board.connectivity.nodes.is_empty() || !board.connectivity.edges.is_empty())
    {
        return false;
    }
    storage_validation::valid(board)
        && !report.issues.iter().any(|issue| {
            issue.severity == Severity::Error
                && !matches!(
                    issue.error,
                    ModelError::UnsupportedTransform | ModelError::PrecisionLoss
                )
        })
}
fn sorted_equal<T: PartialEq>(
    a: &[T],
    b: &[T],
    key: impl Fn(&T) -> &boardsi_model::ObjectId,
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut left: Vec<&T> = a.iter().collect();
    let mut right: Vec<&T> = b.iter().collect();
    left.sort_by(|x, y| key(x).cmp(key(y)));
    right.sort_by(|x, y| key(x).cmp(key(y)));
    left == right
}
fn same_semantics(a: &boardsi_model::BoardModel, b: &boardsi_model::BoardModel) -> bool {
    if a.kind != b.kind {
        return false;
    }
    let mut asrc = a.sources.iter().collect::<Vec<_>>();
    let mut bsrc = b.sources.iter().collect::<Vec<_>>();
    asrc.sort_by(|a, b| a.id.cmp(&b.id));
    bsrc.sort_by(|a, b| a.id.cmp(&b.id));
    asrc == bsrc
        && sorted_equal(&a.frames, &b.frames, |x| &x.meta.id)
        && sorted_equal(&a.stackup.layers, &b.stackup.layers, |x| &x.meta.id)
        && sorted_equal(&a.materials, &b.materials, |x| &x.meta.id)
        && sorted_equal(&a.nets, &b.nets, |x| &x.meta.id)
        && sorted_equal(&a.components, &b.components, |x| &x.meta.id)
        && sorted_equal(&a.pins, &b.pins, |x| &x.meta.id)
        && sorted_equal(&a.traces, &b.traces, |x| &x.meta.id)
        && sorted_equal(&a.arcs, &b.arcs, |x| &x.meta.id)
        && sorted_equal(&a.areas, &b.areas, |x| &x.meta.id)
        && sorted_equal(&a.padstacks, &b.padstacks, |x| &x.meta.id)
        && sorted_equal(&a.pads, &b.pads, |x| &x.meta.id)
        && sorted_equal(&a.drills, &b.drills, |x| &x.meta.id)
        && sorted_equal(&a.vias, &b.vias, |x| &x.meta.id)
        && sorted_equal(&a.outlines, &b.outlines, |x| &x.meta.id)
        && sorted_equal(&a.connectivity.nodes, &b.connectivity.nodes, |x| &x.meta.id)
        && sorted_equal(&a.connectivity.edges, &b.connectivity.edges, |x| &x.meta.id)
}
