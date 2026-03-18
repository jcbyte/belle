use std::path::PathBuf;

use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("HTTP client could not be initialised.")]
    ClientInit {
        #[source]
        source: reqwest::Error,
    },

    #[error("Failed to send request for {name} at {url}")]
    Fetch {
        name: String,
        url: Url,
        #[source]
        source: reqwest::Error,
    },

    #[error("Failed to read {name} fetched from {url}")]
    ReadFetched {
        name: String,
        url: Url,
        #[source]
        source: reqwest::Error,
    },

    #[error("Repository host cannot be identified.")]
    NoRepository,

    #[error("Repository at {repo} is not supported.")]
    RepositoryNotSupported { repo: String },

    #[error("{name} not found at {url}")]
    NotFound { name: String, url: Url },

    #[error("Invalid repository url '{url}'.")]
    InvalidRepositoryURL { url: Url },

    #[error("Failed to create URL for {name}.")]
    InvalidUrlCreated {
        name: String,
        #[source]
        source: url::ParseError,
    },

    #[error("{afp_name} is a legacy AFP repository, the metadata cannot be fetched automatically.")]
    LegacyAfp { afp_name: String },

    #[error("No package manifest found at '{path}'.")]
    NoLocalManifest { path: PathBuf },
}

pub trait FetchErrorContext<T> {
    fn report_failed_init(self) -> Result<T, FetchError>;
    fn report_fetch(self, name: impl Into<String>, url: &Url) -> Result<T, FetchError>;
    fn report_reading_fetched(self, name: impl Into<String>, url: &Url) -> Result<T, FetchError>;
}

impl<T> FetchErrorContext<T> for Result<T, reqwest::Error> {
    fn report_failed_init(self) -> Result<T, FetchError> {
        self.map_err(|e| FetchError::ClientInit { source: e })
    }

    fn report_fetch(self, name: impl Into<String>, url: &Url) -> Result<T, FetchError> {
        self.map_err(|e| FetchError::Fetch {
            name: name.into(),
            url: url.clone(),
            source: e,
        })
    }

    fn report_reading_fetched(self, name: impl Into<String>, url: &Url) -> Result<T, FetchError> {
        self.map_err(|e| FetchError::ReadFetched {
            name: name.into(),
            url: url.clone(),
            source: e,
        })
    }
}

pub trait FetchUrlContext<T> {
    fn report_invalid_url(self, name: impl Into<String>) -> Result<T, FetchError>;
}

impl<T> FetchUrlContext<T> for Result<T, url::ParseError> {
    fn report_invalid_url(self, name: impl Into<String>) -> Result<T, FetchError> {
        self.map_err(|e| FetchError::InvalidUrlCreated {
            name: name.into(),
            source: e,
        })
    }
}
