use std::sync::OnceLock;

use pubgrub::SemanticVersion;
use serde::Deserialize;

/// Container holding a repositories name and heptapod id
#[derive(Deserialize, Debug, Clone)]
pub struct AFPRepo {
    pub id: i32,
    pub name: String,

    // Keep a cache of version number as it may be created multiple times
    #[serde(skip)]
    pub version_cache: OnceLock<SemanticVersion>,
}

impl AFPRepo {
    /// Generate version number for theories within a repo though its name
    pub fn get_version(&self) -> &SemanticVersion {
        // Use each number seperated with a dash as its SemVer version:
        // > 2019   -> 2019.0.0
        // > 2025-2 -> 2025.2.0
        self.version_cache.get_or_init(|| {
            let name_parts: Vec<u32> = self.name.split('-').filter_map(|s| s.parse::<u32>().ok()).collect();

            let major = name_parts.get(0).unwrap_or(&0);
            let minor = name_parts.get(1).unwrap_or(&0);
            let patch = name_parts.get(2).unwrap_or(&0);

            SemanticVersion::new(*major, *minor, *patch)
        })
    }

    pub fn get_repo_name(version: &SemanticVersion) -> String {
        let ver_string = version.to_string();
        let ver_parts: Vec<&str> = ver_string.split('.').collect();

        let name_parts: Vec<&str> = ver_parts.into_iter().filter(|vp| !vp.eq(&"0")).collect();
        return name_parts.join("-");
    }
}
