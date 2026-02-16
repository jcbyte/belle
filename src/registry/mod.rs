use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

pub mod package;

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageAuthor {
    pub name: String,
    pub email: Option<String>,
    pub homepages: Option<Vec<String>>,
    pub orcid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: SemanticVersion,
    pub title: String,
    pub date: toml::value::Date,
    pub r#abstract: String,
    pub licence: String,
    pub topics: Vec<String>,
    pub note: Option<String>,

    pub authors: Vec<PackageAuthor>,
    pub contributors: Vec<PackageAuthor>,

    pub dependencies: HashMap<String, SemanticVersion>,

    pub extra: toml::Table,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    name: String,
    version: SemanticVersion,
    dependencies: HashMap<String, SemanticVersion>,
}
