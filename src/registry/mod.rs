use std::{collections::HashMap, fmt};

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

/// All package metadata stored on disk
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

/// Subset of `Package` stored in disk for quick dependency resolution
#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    name: String,
    version: SemanticVersion,
    dependencies: HashMap<String, SemanticVersion>,
}

/// Key for packages
pub struct PackageIdentifier {
    pub name: String,
    pub version: SemanticVersion,
}

impl From<&Package> for Manifest {
    fn from(package: &Package) -> Self {
        return Self {
            name: package.name.clone(),
            version: package.version.clone(),
            dependencies: package.dependencies.clone(),
        };
    }
}

impl From<&Package> for PackageIdentifier {
    fn from(package: &Package) -> Self {
        return Self {
            name: package.name.clone(),
            version: package.version.clone(),
        };
    }
}

impl From<&Manifest> for PackageIdentifier {
    fn from(manifest: &Manifest) -> Self {
        return Self {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
        };
    }
}

impl fmt::Display for PackageIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}
