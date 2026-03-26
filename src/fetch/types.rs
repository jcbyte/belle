use std::{fmt, sync::OnceLock};

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

use crate::{
    registry::{AliasPackage, Package},
    util::{get_isabelle_name, get_isabelle_version},
};

/// Container holding a repositories name and heptapod id
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AfpRepo {
    pub id: u32,
    pub name: String,

    // Keep a cache of version number as it may be created multiple times
    #[serde(skip)]
    pub version_cache: OnceLock<SemanticVersion>,
}

impl AfpRepo {
    /// Generate version number for afp repo though its name
    pub fn get_version(&self) -> &SemanticVersion {
        self.version_cache.get_or_init(|| get_isabelle_version(&self.name))
    }

    pub fn get_formatted_name(&self) -> String {
        get_isabelle_name(self.get_version())
    }
}

impl fmt::Display for AfpRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct ReturnedPackages {
    pub package: Package,
    pub aliases: Vec<AliasPackage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version_and_caching() {
        let repo = AfpRepo {
            id: 1,
            name: "afp-2024-1".to_string(),
            version_cache: OnceLock::new(),
        };

        // First call initializes
        let v1 = repo.get_version();
        assert_eq!(*v1, SemanticVersion::new(2024, 1, 0));

        // This should receive the cached version
        let v2 = repo.get_version();
        assert_eq!(v1, v2);
    }
}
