use std::sync::OnceLock;

use pubgrub::SemanticVersion;
use serde::Deserialize;

use tabled::Tabled;

/// Container holding a repositories name and heptapod id
/// It is deserializable and able to be pretty printed as a table
#[derive(Deserialize, Debug, Clone, Tabled)]
pub struct AFPRepo {
    #[tabled(skip)]
    pub id: i32,
    #[tabled(rename = "Repo Name")]
    pub name: String,

    // Display version on the table, but do not serialise
    #[serde(skip)]
    #[tabled(display("display_version_private", self))]
    pub version: (), // Use () as a placeholder

    // Keep a cache of version number as it may be created multiple times
    #[serde(skip)]
    #[tabled(skip)]
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
}

fn display_version_private(_: &(), repo: &AFPRepo) -> String {
    repo.get_version().to_string()
}
