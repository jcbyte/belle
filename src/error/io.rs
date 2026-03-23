use hinted::Hint;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Hint)]
pub enum IoError {
    #[error("could not save {name} to '{path}'")]
    #[hint("check write permissions, and that the disk is not full")]
    Save {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not remove {name} at '{path}'")]
    #[hint("ensure this file is not in use by another process, and that you have delete permissions")]
    Delete {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not read {name} at '{path}'")]
    #[hint("verify the file exists, and you have read permissions")]
    Read {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("path '{path}' could not be interpreted")]
    #[hint("ensure the path uses valid utf-8 characters")]
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
