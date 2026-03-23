use hinted::Hint;
use thiserror::Error;

mod archive;
mod custom;
mod io;
mod parser;

pub use archive::*;
pub use custom::*;
pub use io::*;
pub use parser::*;

use crate::{
    cli::error::CliError,
    environment::error::EnvironmentError,
    fetch::{
        afp_metadata::error::{AfpMetadataError, RootParserError},
        error::FetchError,
    },
    isabelle::error::IsabelleError,
    registry::error::RegistryError,
    resolver::error::ResolverError,
};

#[derive(Error, Debug, Hint)]
pub enum AppError {
    #[error(transparent)]
    Custom(#[from] CustomError),

    #[error(transparent)]
    #[hint(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Parse(Box<ParserError>),

    #[error(transparent)]
    Archive(#[from] ArchiveError),

    #[error(transparent)]
    #[hint(transparent)]
    Environment(#[from] EnvironmentError),

    #[error(transparent)]
    Resolver(#[from] ResolverError),

    #[error(transparent)]
    #[hint(transparent)]
    Isabelle(#[from] IsabelleError),

    #[error(transparent)]
    #[hint(transparent)]
    Registry(#[from] RegistryError),

    #[error(transparent)]
    #[hint(transparent)]
    Fetch(Box<FetchError>),

    #[error(transparent)]
    #[hint(transparent)]
    Metadata(#[from] AfpMetadataError),

    #[error(transparent)]
    RootParser(#[from] RootParserError),

    #[error(transparent)]
    #[hint(transparent)]
    Cli(#[from] CliError),
}

// Custom From implementations to handle boxing

impl From<ParserError> for AppError {
    fn from(err: ParserError) -> Self {
        AppError::Parse(Box::new(err))
    }
}

impl From<FetchError> for AppError {
    fn from(err: FetchError) -> Self {
        AppError::Fetch(Box::new(err))
    }
}
