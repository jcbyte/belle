use thiserror::Error;

mod global;

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

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Custom(#[from] CustomError),

    #[error(transparent)]
    Io(#[from] IoError),

    #[error(transparent)]
    Parse(#[from] ParserError),

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
    Fetch(#[from] FetchError),

    #[error(transparent)]
    Metadata(#[from] MetadataError),

    #[error(transparent)]
    RootParser(#[from] RootParserError),

    #[error(transparent)]
    Cli(#[from] CliError),
}

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("{msg}")]
    WithSource {
        msg: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("{msg}")]
    WithoutSource { msg: String },
}

pub trait CustomErrorContext<T> {
    fn report_custom(self, msg: impl Into<String>) -> Result<T, CustomError>;
}

impl<T> CustomErrorContext<T> for Option<T> {
    fn report_custom(self, msg: impl Into<String>) -> Result<T, CustomError> {
        self.ok_or_else(|| CustomError::WithoutSource { msg: msg.into() })
    }
}

impl<T, E> CustomErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn report_custom(self, msg: impl Into<String>) -> Result<T, CustomError> {
        self.map_err(|e| CustomError::WithSource {
            msg: msg.into(),
            source: Box::new(e),
        })
    }
}
