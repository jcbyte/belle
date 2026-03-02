use std::collections::HashMap;

use crate::{fetch::AFPRepo, registry::PackageAuthor};

/// Interpretation of AFP Author metadata
#[derive(Debug, Clone)]
pub struct AuthorMetadata {
    pub name: String,
    pub email: Option<String>,
    pub homepages: Option<Vec<String>>,
    pub orcid: Option<String>,
}

/// Interpretation of AFP Theory metadata
#[derive(Debug, Clone)]
pub struct TheoryMetadata {
    pub title: String,
    pub date: toml::value::Date,
    pub r#abstract: String,
    pub licence_key: String,
    pub topics: Vec<String>,
    pub note: Option<String>,
    pub author_keys: Vec<String>,
    pub contributor_keys: Vec<String>,
    pub extra: toml::Table,
}

/// Interpretation of AFP repo metadata
#[derive(Debug)]
pub struct RepoMetadata {
    pub repo: AFPRepo,
    pub authors: HashMap<String, AuthorMetadata>,
    pub licences: HashMap<String, String>,
    pub theories: HashMap<String, TheoryMetadata>,

    pub seen_aliases: HashMap<String, String>,
}

impl From<AuthorMetadata> for PackageAuthor {
    fn from(meta: AuthorMetadata) -> Self {
        Self {
            name: meta.name,
            email: meta.email,
            homepages: meta.homepages,
            orcid: meta.orcid,
        }
    }
}
