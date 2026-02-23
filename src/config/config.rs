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

    fn get_root_env_dir(&self) -> PathBuf {
        return self.home.join("env");
    }

    /// Get folder for environments
    pub fn get_env_dir(&self) -> PathBuf {
        return self.get_root_env_dir().join("envs");
    }

    /// Get folder for environments
    pub fn get_active_env_link(&self) -> PathBuf {
        return self.get_root_env_dir().join("active");
    }
}
