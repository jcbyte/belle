use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::PathBuf,
};

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::fetch::AFPRepo;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "_type")]
pub enum RegisteredPackage {
    Package(Package),
    Alias(AliasPackage),
}

/// Theory author information
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageAuthor {
    pub name: String,
    pub email: Option<String>,
    pub homepages: Option<Vec<String>>,
    pub orcid: Option<String>,
}

/// Theory source information
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(tag = "type")]
pub enum PackageSource {
    Afp(AFPRepo),
    Remote {
        url: Url,
    },
    Local {
        path: PathBuf,
    },

    #[default]
    Default,
}

// todo 1 support remote repos

/// All package metadata
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

    pub provides: Vec<String>,
    pub dependencies: HashMap<String, SemanticVersion>,
    pub isabelles: HashSet<SemanticVersion>,

    #[serde(default)]
    pub source: PackageSource,

    pub extra: toml::Table,
}

/// Package identifier for lookup and passing
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PackageIdentifier {
    pub name: String,
    pub version: SemanticVersion,
}

impl From<&Package> for PackageIdentifier {
    fn from(package: &Package) -> Self {
        return Self {
            name: package.name.clone(),
            version: package.version.clone(),
        };
    }
}

impl fmt::Display for PackageIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// A package which is an alias for another package
#[derive(Serialize, Deserialize, Debug)]
pub struct AliasPackage {
    pub name: String,
    pub version: SemanticVersion,
    pub alias: PackageIdentifier,
}

impl From<&AliasPackage> for PackageIdentifier {
    fn from(alias: &AliasPackage) -> Self {
        return Self {
            name: alias.name.clone(),
            version: alias.version.clone(),
        };
    }
}

impl From<Package> for RegisteredPackage {
    fn from(package: Package) -> Self {
        return Self::Package(package);
    }
}

impl From<AliasPackage> for RegisteredPackage {
    fn from(alias: AliasPackage) -> Self {
        return Self::Alias(alias);
    }
}
