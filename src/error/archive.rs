use thiserror::Error;
use zip::result::ZipError;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("could not read archive {name}")]
    Read {
        name: String,
        #[source]
        source: ZipError,
    },

    #[error("could not read entry {index} in archive {name}")]
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
