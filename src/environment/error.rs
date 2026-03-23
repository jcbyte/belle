use std::path::PathBuf;

use hinted::Hint;
use thiserror::Error;

#[derive(Error, Debug, Hint)]
pub enum EnvironmentError {
    #[error("environment '{name}' already exists")]
    #[hint("use a different name, or remove it with `belle env remove <name>` first")]
    AlreadyExists { name: String },

    #[error("environment '{name}' does not exist")]
    #[hint("check the name, or create it with `belle env create <name>`")]
    DoesNotExist { name: String },

    #[error("no lockfile found at '{path}'")]
    #[hint("lockfile can be generated with `belle env freeze`")]
    NoLockFile { path: PathBuf },
}
