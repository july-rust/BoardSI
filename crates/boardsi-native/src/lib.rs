//! BoardSI native .bsi v1; all load paths are read-only and fail-closed.
mod bsig;
mod encoding;
mod store;
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
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<rusqlite::Error> for NativeError {
    fn from(e: rusqlite::Error) -> Self { Self::Sql(e) }
}
fn invalid(msg: &'static str) -> NativeError { NativeError::Invalid(msg) }
