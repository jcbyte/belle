use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::PathBuf,
};

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::fetch::AfpRepo;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "_type")]
#[allow(clippy::large_enum_variant)] // As majority of packages are the larger variant
pub enum RegisteredPackage {
    Package(Package),
    Alias(AliasPackage),
}

/// Entry author information
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageAuthor {
    pub name: String,
    pub email: Option<String>,
    pub homepages: Option<Vec<String>>,
    pub orcid: Option<String>,
}

/// Entry source information
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(tag = "type")]
pub enum PackageSource {
    Afp(AfpRepo),
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
    pub fn new(name: impl Into<String>, version: impl Into<SemanticVersion>) -> Self {
        PackageIdentifier {
            name: name.into(),
            version: version.into(),
        }
    }
}

impl From<Package> for PackageIdentifier {
    fn from(package: Package) -> Self {
        Self::new(package.name, package.version)
    }
}

impl From<&Package> for PackageIdentifier {
    fn from(package: &Package) -> Self {
        Self::new(package.name.clone(), package.version)
    }
}

impl fmt::Display for PackageIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)
    }
}

/// A package which is an alias for another package
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AliasPackage {
    pub name: String,
    pub version: SemanticVersion,
    pub alias: PackageIdentifier,
}

impl From<AliasPackage> for PackageIdentifier {
    fn from(alias: AliasPackage) -> Self {
        Self::new(alias.name, alias.version)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use pubgrub::SemanticVersion;

    // Function to create a toml date value for tests
    fn get_mock_date() -> toml::value::Date {
        #[derive(Deserialize)]
        struct TempStruct {
            date: toml::value::Date,
        }

        let temp_data: TempStruct = toml::from_str("date = 2026-02-17").unwrap();
        temp_data.date
    }

    #[test]
    fn test_package_conversion() {
        // Create a minimal package for testing
        let pkg = Package {
            name: "test-pkg".to_string(),
            version: SemanticVersion::two(),
            title: "Title".into(),
            date: get_mock_date(),
            r#abstract: "Abstract".into(),
            licence: "MIT".into(),
            topics: Vec::new(),
            note: None,
            authors: Vec::new(),
            contributors: Vec::new(),
            provides: Vec::new(),
            dependencies: HashMap::new(),
            isabelles: HashSet::new(),
            source: PackageSource::Default,
            extra: toml::Table::new(),
        };

        // Test package identifier conversion
        let ident: PackageIdentifier = (&pkg).into();
        assert_eq!(ident.name, "test-pkg");
        assert_eq!(ident.version, SemanticVersion::two());

        // Test registered package conversion
        let reg: RegisteredPackage = pkg.into();
        assert!(matches!(reg, RegisteredPackage::Package(_)));
    }

    #[test]
    fn test_alias_to_identifier_conversion() {
        let alias = AliasPackage {
            name: "my-alias".into(),
            version: SemanticVersion::zero(),
            alias: PackageIdentifier::new("original", SemanticVersion::one()),
        };

        // Test package identifier conversion
        let ident: PackageIdentifier = alias.clone().into();
        assert_eq!(ident.name, "my-alias");
        assert_eq!(ident.version, SemanticVersion::zero());

        // Test registered package conversion
        let reg: RegisteredPackage = alias.into();
        assert!(matches!(reg, RegisteredPackage::Alias(_)));
    }
}
