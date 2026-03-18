use std::path::PathBuf;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("Could not save {name} at '{path}'.")]
    Save {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not remove {name} at '{path}'.")]
    Delete {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Could not read {name} at '{path}'.")]
    Read {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Path '{path}' could not be interpreted.")]
    Path { path: PathBuf },
}

pub trait IoErrorContext<T> {
    fn report_save(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError>;
    fn report_delete(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError>;
    fn report_read(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError>;
}

impl<T> IoErrorContext<T> for std::io::Result<T> {
    fn report_save(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.map_err(|e| IoError::Save {
            name: name.into(),
            path: path.into(),
            source: e,
        })
    }

    fn report_delete(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.map_err(|e| IoError::Delete {
            name: name.into(),
            path: path.into(),
            source: e,
        })
    }

    fn report_read(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.map_err(|e| IoError::Read {
            name: name.into(),
            path: path.into(),
            source: e,
        })
    }
}

pub trait IoPathErrorContext<T> {
    fn report_path(self, path: impl Into<PathBuf>) -> Result<T, IoError>;
}

impl<T> IoPathErrorContext<T> for Option<T> {
    fn report_path(self, path: impl Into<PathBuf>) -> Result<T, IoError> {
        self.ok_or_else(|| IoError::Path { path: path.into() })
    }
}

#[derive(Error, Debug)]
pub enum ParserError {
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
        name: String,
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Could not serialise {name} for '{path}'")]
    SerFile {
        name: String,
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
}

pub trait ParseErrorContext<T> {
    fn report_data(self, name: &'static str) -> Result<T, ParserError>;
    fn report_file(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, ParserError>;
}

impl<T> ParseErrorContext<T> for Result<T, toml::de::Error> {
    fn report_data(self, name: &'static str) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::DeData { name, source: e })
    }

    fn report_file(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::DeFile {
            name: name.into(),
            path: path.into(),
            source: e,
        })
    }
}

impl<T> ParseErrorContext<T> for Result<T, toml::ser::Error> {
    fn report_data(self, name: &'static str) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::SerData { name, source: e })
    }

    fn report_file(self, name: impl Into<String>, path: impl Into<PathBuf>) -> Result<T, ParserError> {
        self.map_err(|e| ParserError::SerFile {
            name: name.into(),
            path: path.into(),
            source: e,
        })
    }
}

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("Could read archive {name}.")]
    Read {
        name: String,
        #[source]
        source: ZipError,
    },

    #[error("Could read archive {name} at index {index}.")]
    BadIndex {
        name: String,
        index: usize,
        #[source]
        source: ZipError,
    },
}

pub trait ArchiveErrorContext<T> {
    fn report_read(self, name: impl Into<String>) -> Result<T, ArchiveError>;
    fn report_index(self, name: impl Into<String>, index: impl Into<usize>) -> Result<T, ArchiveError>;
}

impl<T> ArchiveErrorContext<T> for Result<T, ZipError> {
    fn report_read(self, name: impl Into<String>) -> Result<T, ArchiveError> {
        self.map_err(|e| ArchiveError::Read {
            name: name.into(),
            source: e,
        })
    }

    fn report_index(self, name: impl Into<String>, index: impl Into<usize>) -> Result<T, ArchiveError> {
        self.map_err(|e| ArchiveError::BadIndex {
            name: name.into(),
            index: index.into(),
            source: e,
        })
    }
}
