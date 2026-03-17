use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Could not save {item_type} at '{path}': {source}")]
    Save {
        item_type: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not remove {item_type} at '{path}': {source}")]
    Delete {
        item_type: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not read {item_type} at '{path}': {source}")]
    Read {
        item_type: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Path '{path}' could not be interpreted.")]
    Path { path: PathBuf },
}

pub trait IoContext<T> {
    fn report_save(self, item_type: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError>;
    fn report_delete(self, item_type: &'static str, path: impl Into<PathBuf>) -> Result<T, IoError>;
    fn report_read(self, item_type: &'static str, path: impl Into<PathBuf>) -> Result<T, IoError>;
}

impl<T> IoContext<T> for std::io::Result<T> {
    fn report_save(self, item_type: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.map_err(|e| IoError::Save {
            item_type: item_type.into(),
            path: path.into(),
            source: e,
        })
    }

    fn report_delete(self, item_type: &'static str, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.map_err(|e| IoError::Delete {
            item_type,
            path: path.into(),
            source: e,
        })
    }

    fn report_read(self, item_type: &'static str, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.map_err(|e| IoError::Read {
            item_type,
            path: path.into(),
            source: e,
        })
    }
}

pub trait IoPathContext<T> {
    fn report_path(self, path: impl Into<PathBuf>) -> Result<T, IoError>;
}

impl<T> IoPathContext<T> for Option<T> {
    fn report_path(self, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.ok_or_else(|| IoError::Path { path: path.into() })
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Could not deserialise data from {name}")]
    DeData {
        name: &'static str,
        #[source]
        source: toml::de::Error,
    },

    #[error("Could not serialise data for {name}")]
    SerData {
        name: &'static str,
        #[source]
        source: toml::ser::Error,
    },

    #[error("Could not deserialise {name} from '{path}'")]
    DeFile {
        name: &'static str,
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Could not serialise {name} for '{path}'")]
    SerFile {
        name: &'static str,
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
}

pub trait ParseContext<T> {
    fn report_data(self, name: &'static str) -> Result<T, ParseError>;
    fn report_file(self, name: &'static str, path: impl Into<PathBuf>) -> Result<T, ParseError>;
}

impl<T> ParseContext<T> for Result<T, toml::de::Error> {
    fn report_data(self, name: &'static str) -> Result<T, ParseError> {
        self.map_err(|e| ParseError::DeData { name, source: e })
    }

    fn report_file(self, name: &'static str, path: impl Into<PathBuf>) -> Result<T, ParseError> {
        self.map_err(|e| ParseError::DeFile {
            name,
            path: path.into(),
            source: e,
        })
    }
}

impl<T> ParseContext<T> for Result<T, toml::ser::Error> {
    fn report_data(self, name: &'static str) -> Result<T, ParseError> {
        self.map_err(|e| ParseError::SerData { name, source: e })
    }

    fn report_file(self, name: &'static str, path: impl Into<PathBuf>) -> Result<T, ParseError> {
        self.map_err(|e| ParseError::SerFile {
            name,
            path: path.into(),
            source: e,
        })
    }
}
