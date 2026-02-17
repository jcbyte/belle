use std::collections::HashMap;

use serde::Deserialize;

// Schema types to mirror the TOML structure used in AFP metadata

#[derive(Deserialize, Debug)]
pub struct MetaAuthorEmail {
    pub user: Vec<String>,
    pub host: Vec<String>,
}
impl MetaAuthorEmail {
    pub fn to_string(&self) -> String {
        format!("{}@{}", self.user.join("."), self.host.join("."))
    }
}

#[derive(Deserialize, Debug)]
pub struct MetaAuthor {
    pub name: String,
    pub orcid: Option<String>,

    // Default to empty map if the section is missing in TOML
    #[serde(default)]
    pub emails: HashMap<String, MetaAuthorEmail>,

    #[serde(default)]
    pub homepages: HashMap<String, String>,
}

#[derive(serde::Deserialize)]
pub struct MetaLicence {
    pub name: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct MetaTheoryRelated {
    #[serde(default)]
    pub dois: Vec<String>,
    #[serde(default)]
    pub pubs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaTheory {
    pub title: String,
    pub date: toml::value::Date,
    pub topics: Vec<String>,
    pub r#abstract: String,
    pub license: String,
    pub note: Option<String>,

    #[serde(default)]
    pub authors: HashMap<String, toml::Value>,
    #[serde(default)]
    pub contributors: HashMap<String, toml::Value>,
    #[serde(default)]
    pub extra: toml::Table,
    #[serde(default)]
    pub related: MetaTheoryRelated,
}
