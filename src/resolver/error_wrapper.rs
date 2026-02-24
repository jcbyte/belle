use std::fmt;

/// Wrapper for `anyhow::Error` which implements `std::error::Error` for pubgrub
#[derive(Debug)]
pub struct SolverError(anyhow::Error);

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<anyhow::Error> for SolverError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

impl std::error::Error for SolverError {}
