use pubgrub::SemanticVersion;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IsabelleError {
    #[error("Failed to execute 'isabelle {args}'.")]
    CommandFailed {
        args: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command 'isabelle {args}' produced invalid UTF-8 output.")]
    InvalidCommandOutput {
        args: String,
        #[source]
        source: std::string::FromUtf8Error,
    },

    #[error("There already exists a linked Isabelle matching version '{version}'")]
    AlreadyLinked { version: SemanticVersion },

    #[error("Could not find a linked Isabelle matching version '{version}'")]
    VersionNotLinked { version: SemanticVersion },
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
