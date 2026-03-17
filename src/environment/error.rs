use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvironmentError {
    #[error("Environment '{name}' already exists.")]
    AlreadyExists { name: String },

    #[error("Environment '{name}' does not exist.")]
    DoesNotExist { name: String },

    #[error("Environment file at '{path}' does not exist.")]
    FileDoesNotExist { path: PathBuf },

    #[error("No lockfile at found at '{path}'.")]
    NoLockFile { path: PathBuf },

    #[error("Package {package} already exists in environment.")]
    PackageAlreadyExists { package: String },

    #[error("Package {package} does not exists in environment.")]
    PackageDoesNotExist { package: String },
}
