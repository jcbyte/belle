use std::sync::OnceLock;

use pubgrub::SemanticVersion;
use serde::Deserialize;

use crate::util::get_isabelle_version;

/// Container holding a repositories name and heptapod id
#[derive(Deserialize, Debug, Clone)]
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
