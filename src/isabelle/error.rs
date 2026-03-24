use std::path::PathBuf;

use hinted::Hint;
use thiserror::Error;

#[derive(Error, Debug, Hint)]
pub enum IsabelleError {
    #[error("no Isabelle installation found at '{path}'")]
    #[hint("check path, or install isabelle from https://isabelle.in.tum.de")]
    NoIsabelle { path: PathBuf },

    #[error("failed to execute 'isabelle {args}'")]
    #[hint("check execute permissions, and ensure that Isabelle is not corrupted")]
    CommandFailed {
        args: String,
        #[source]
        source: std::io::Error,
    },

    #[error("command 'isabelle {args}' produced invalid output")]
    #[hint("this may be due to a locale mismatch or a corrupted isabelle")]
    InvalidCommandOutput {
        args: String,
        #[source]
        source: std::string::FromUtf8Error,
    },
}

pub trait IsabelleCommandFailedContext<T> {
    fn report_failed_isabelle_command(self, args: impl Into<String>) -> Result<T, IsabelleError>;
}

impl<T> IsabelleCommandFailedContext<T> for std::io::Result<T> {
    fn report_failed_isabelle_command(self, args: impl Into<String>) -> Result<T, IsabelleError> {
        self.map_err(|e| IsabelleError::CommandFailed {
            args: args.into(),
            source: e,
        })
    }
}

pub trait IsabelleInvalidOutputContext<T> {
    fn report_invalid_isabelle_command_output(self, args: impl Into<String>) -> Result<T, IsabelleError>;
}

impl<T> IsabelleInvalidOutputContext<T> for Result<T, std::string::FromUtf8Error> {
    fn report_invalid_isabelle_command_output(self, args: impl Into<String>) -> Result<T, IsabelleError> {
        self.map_err(|e| IsabelleError::InvalidCommandOutput {
            args: args.into(),
            source: e,
        })
    }
}
