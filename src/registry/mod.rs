use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::Serialize;

pub mod package;

#[derive(Serialize)]
pub struct PackageAuthor {
    name: String,
    email: Option<String>,
    homepages: Option<Vec<String>>,
    orcid: Option<String>,
}

#[derive(Serialize)]
pub struct Package {
    name: String,
    version: SemanticVersion,
    title: String,
    date: toml::value::Datetime,
    r#abstract: String,
    licence: String,
    topics: Vec<String>,
    note: Option<String>,

    authors: Vec<PackageAuthor>,
    contributors: Vec<PackageAuthor>,

    dependencies: HashMap<String, SemanticVersion>,

    extra: toml::Table,
}

#[derive(Serialize)]
pub struct Manifest {
    name: String,
    version: SemanticVersion,
    dependencies: HashMap<String, SemanticVersion>,
}
