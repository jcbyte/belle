use thiserror::Error;

mod global;

pub use global::*;

use crate::{
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
    // #[error("{msg}")]
    // Custom {
    //     msg: String,
    //     #[source]
    //     source: Box<dyn std::error::Error + Send + Sync>,
    // },
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
}

// pub trait CustomErrorContext<T> {
//     fn report_custom(self, msg: impl Into<String>) -> Result<T, AppError>;
// }

// impl<T, E> CustomErrorContext<T> for Result<T, E>
// where
//     E: Error + Send + Sync + 'static,
// {
//     fn report_custom(self, msg: impl Into<String>) -> Result<T, AppError> {
//         self.map_err(|e| AppError::Custom {
//             msg: msg.into(),
//             source: Box::new(e),
//         })
//     }
// }
