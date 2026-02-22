use std::path::PathBuf;

use crate::config::types::ConfigData;

impl ConfigData {
    /// Get folder for metadata
    pub fn get_meta_dir(&self) -> PathBuf {
        return self.home.join("meta");
    }

    /// Get folder for manifest
    pub fn get_manifest_dir(&self) -> PathBuf {
        return self.home.join("manifest");
    }

    /// Get folder for theories
    pub fn get_theory_dir(&self) -> PathBuf {
        return self.home.join("theory");
    }
}
