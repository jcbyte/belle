use std::path::PathBuf;

use crate::config::types::ConfigData;

impl ConfigData {
    /// Get folder for manifest
    pub fn get_manifest_dir(&self) -> PathBuf {
        self.home.join("mft")
    }

    /// Get folder for theories
    pub fn get_package_dir(&self) -> PathBuf {
        self.home.join("pkg")
    }

    fn get_root_env_dir(&self) -> PathBuf {
        self.home.join("env")
    }

    /// Get folder for environments
    pub fn get_env_dir(&self) -> PathBuf {
        self.get_root_env_dir().join("envs")
    }

    /// Get folder for environments
    pub fn get_active_env_link(&self) -> PathBuf {
        self.get_root_env_dir().join("active")
    }
}
