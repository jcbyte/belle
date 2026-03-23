use thiserror::Error;

mod custom;
mod global;

pub use custom::*;
pub use global::*;

use crate::{
    cli::error::CliError,
    environment::error::EnvironmentError,
    fetch::{
        afp_metadata::error::{MetadataError, RootParserError},
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
    Io(#[from] IoError),

    #[error(transparent)]
    Parse(Box<ParserError>),

    #[error(transparent)]
    Archive(#[from] ArchiveError),

    #[error(transparent)]
    Environment(#[from] EnvironmentError),

    #[error(transparent)]
    Resolver(#[from] ResolverError),

    #[error(transparent)]
    Isabelle(#[from] IsabelleError),

    #[error(transparent)]
    Registry(#[from] RegistryError),

    #[error(transparent)]
    Fetch(Box<FetchError>),

    #[error(transparent)]
    Metadata(#[from] MetadataError),

    #[error(transparent)]
    RootParser(#[from] RootParserError),

    #[error(transparent)]
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
