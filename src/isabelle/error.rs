use pubgrub::SemanticVersion;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IsabelleError {
    #[error("Isabelle failed to execute '{command}'.")]
    CommandFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Isabelle command '{command}' produced invalid UTF-8 output.")]
    InvalidCommandOutput {
        command: String,
        #[source]
        source: std::string::FromUtf8Error,
    },

    #[error("Could not find a linked Isabelle matching version '{version}'")]
    VersionNotLinked { version: SemanticVersion },
}

pub trait IsabelleCommandFailedContext<T> {
    fn report_failed_command(self, command: impl Into<String>) -> Result<T, IsabelleError>;
}

impl<T> IsabelleCommandFailedContext<T> for std::io::Result<T> {
    fn report_failed_command(self, command: impl Into<String>) -> Result<T, IsabelleError> {
        self.map_err(|e| IsabelleError::CommandFailed {
            command: command.into(),
            source: e,
        })
    }
}

pub trait IsabelleInvalidOutputContext<T> {
    fn report_invalid_output(self, command: impl Into<String>) -> Result<T, IsabelleError>;
}

impl<T> IsabelleInvalidOutputContext<T> for Result<T, std::string::FromUtf8Error> {
    fn report_invalid_output(self, command: impl Into<String>) -> Result<T, IsabelleError> {
        self.map_err(|e| IsabelleError::InvalidCommandOutput {
            command: command.into(),
            source: e,
        })
    }
}

pub trait IsabelleVersionLinkedContext<T> {
    fn report_not_linked(self, version: impl Into<SemanticVersion>) -> Result<T, IsabelleError>;
}

impl<T> IsabelleVersionLinkedContext<T> for Option<T> {
    fn report_not_linked(self, version: impl Into<SemanticVersion>) -> Result<T, IsabelleError> {
        self.ok_or_else(|| IsabelleError::VersionNotLinked {
            version: version.into(),
        })
    }
}
