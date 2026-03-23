use std::path::PathBuf;

use hinted::Hint;
use thiserror::Error;
use url::Url;

use crate::fetch::AfpRepo;

#[derive(Error, Debug, Hint)]
pub enum FetchError {
    #[error("http client could not be initialised")]
    ClientInit {
        #[source]
        source: reqwest::Error,
    },

    #[error("failed to send request for {name} at {url}")]
    #[hint("check your internet connection or proxy settings")]
    Fetch {
        name: String,
        url: Url,
        #[source]
        source: reqwest::Error,
    },

    #[error("failed to read {name} from {url}")]
    #[hint("the connection may have been closed prematurely")]
    ReadFetched {
        name: String,
        url: Url,
        #[source]
        source: reqwest::Error,
    },

    #[error("invalid repository url '{url}'")]
    #[hint("ensure the url is well-formed")]
    InvalidRepositoryURL { url: Url },

    #[error("repository at {repo} is not supported")]
    #[hint("currently only GitHub repositories are supported")]
    RepositoryNotSupported { repo: String },

    #[error("{name} not found at {url}")]
    #[hint("verify the url is correct, and that the resource is public")]
    NotFound { name: String, url: Url },

    #[error("failed to construct url for {name}")]
    InvalidUrlCreated {
        name: String,
        #[source]
        source: url::ParseError,
    },

    #[error("{repo} is a legacy afp repository")]
    #[hint("legacy repositories cannot be sourced automatically")]
    LegacyAfp { repo: AfpRepo },

    #[error("no package manifest found at '{path}'")]
    #[hint("ensure a 'belle-pkg.toml' exists within the package directory")]
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
