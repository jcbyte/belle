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
#[allow(clippy::large_enum_variant)] // As majority of packages are the larger variant
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
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PackageIdentifier {
    pub name: String,
    pub version: SemanticVersion,
}

impl PackageIdentifier {
    pub fn new(name: impl Into<String>, version: SemanticVersion) -> Self {
        PackageIdentifier {
            name: name.into(),
            version,
        }
    }
}

impl From<&Package> for PackageIdentifier {
    fn from(package: &Package) -> Self {
        Self::new(package.name.clone(), package.version)
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
        Self::new(alias.name.clone(), alias.version)
    }
}

impl From<Package> for RegisteredPackage {
    fn from(package: Package) -> Self {
        Self::Package(package)
    }
}

impl From<AliasPackage> for RegisteredPackage {
    fn from(alias: AliasPackage) -> Self {
        Self::Alias(alias)
    }
}
