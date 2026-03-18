use thiserror::Error;

mod global;

pub use global::*;

use crate::{
    environment::error::EnvironmentError, isabelle::error::IsabelleError, registry::error::RegistryError,
    resolver::error::ResolverError,
};

#[derive(Error, Debug)]
pub enum AppError {
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
}
