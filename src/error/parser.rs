use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("could not parse {name} data")]
    DeData {
        name: String,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("could not format {name} data")]
    SerData {
        name: String,
        #[source]
        source: Box<toml::ser::Error>,
    },

    #[error("Could not parse {name} from '{path}'")]
    DeFile {
        name: String,
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },

    #[error("Could not format {name} for '{path}'")]
    SerFile {
        name: String,
        path: PathBuf,
        #[source]
        source: Box<toml::ser::Error>,
    },
}

pub trait ParseErrorContext<T> {
    fn report_data(self, name: impl Into<String>) -> Result<T, ParserError>;
    fn report_file(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, ParserError>;
}

impl<T> ParseErrorContext<T> for Result<T, toml::de::Error> {
    fn report_data(self, name: impl Into<String>) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::DeData {
            name: name.into(),
            source: e.into(),
        })
    }

    fn report_file(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::DeFile {
            name: name.into(),
            path: path.into(),
            source: e.into(),
        })
    }
}

impl<T> ParseErrorContext<T> for Result<T, toml::ser::Error> {
    fn report_data(self, name: impl Into<String>) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::SerData {
            name: name.into(),
            source: e.into(),
        })
    }

    fn report_file(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::SerFile {
            name: name.into(),
            path: path.into(),
            source: e.into(),
        })
    }
}
