use thiserror::Error;

mod global;

pub use global::{IoContext, IoError, IoPathContext, ParseContext, ParseError};

use crate::environment::error::EnvironmentError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Environment(#[from] EnvironmentError),
}
