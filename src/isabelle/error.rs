use std::{path::PathBuf, process::Output};

use hinted::Hint;
use thiserror::Error;

#[derive(Error, Debug, Hint)]
pub enum IsabelleError {
    #[error("no Isabelle installation found at '{path}'")]
    #[hint("check path, or install isabelle from https://isabelle.in.tum.de")]
    NoIsabelle { path: PathBuf },

    #[error("failed to execute 'isabelle {}'", args.join(" "))]
    #[hint("check execute permissions, and ensure that Isabelle is not corrupted")]
    CommandFailed {
        args: Vec<String>,
        #[source]
        source: std::io::Error,
    },

    #[error("'isabelle {}' retuned a non-success error status{}", args.join(" "), output.status.code().map(|c| format!(": {c}")).unwrap_or_default())]
    CommandNotSuccess { args: Vec<String>, output: Output },

    #[error("command 'isabelle {}' produced invalid output", args.join(" "))]
    #[hint("this may be due to a locale mismatch or a corrupted isabelle")]
    InvalidCommandOutput {
        args: Vec<String>,
        #[source]
        source: std::string::FromUtf8Error,
    },
}

pub trait IsabelleCommandFailedContext<T> {
    fn report_failed_isabelle_command<I, S>(self, args: I) -> Result<T, IsabelleError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>;
}

impl<T> IsabelleCommandFailedContext<T> for std::io::Result<T> {
    fn report_failed_isabelle_command<I, S>(self, args: I) -> Result<T, IsabelleError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.map_err(|e| IsabelleError::CommandFailed {
            args: args.into_iter().map(|a| a.into()).collect(),
            source: e,
        })
    }
}

pub trait IsabelleInvalidOutputContext<T> {
    fn report_invalid_isabelle_command_output<I, S>(self, args: I) -> Result<T, IsabelleError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>;
}

impl<T> IsabelleInvalidOutputContext<T> for Result<T, std::string::FromUtf8Error> {
    fn report_invalid_isabelle_command_output<I, S>(self, args: I) -> Result<T, IsabelleError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.map_err(|e| IsabelleError::InvalidCommandOutput {
            args: args.into_iter().map(|a| a.into()).collect(),
            source: e,
        })
    }
}
