use std::{fmt, sync::OnceLock};

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

use crate::{
    registry::{AliasPackage, Package},
    util::get_isabelle_version,
};

/// Container holding a repositories name and heptapod id
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AFPRepo {
    pub id: u32,
    pub name: String,

    // Keep a cache of version number as it may be created multiple times
    #[serde(skip)]
    pub version_cache: OnceLock<SemanticVersion>,
}

impl AFPRepo {
    /// Generate version number for theories within a repo though its name
    pub fn get_version(&self) -> &SemanticVersion {
        self.version_cache.get_or_init(|| get_isabelle_version(&self.name))
    }
}

impl fmt::Display for AFPRepo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct ReturnedPackages {
    pub package: Package,
    pub aliases: Vec<AliasPackage>,
}
