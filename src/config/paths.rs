use std::{
    env,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::config::BelleConfig;

static HOME_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn get_home_dir() -> &'static Path {
    // Use a cached home directory, as this could be called many times and performs system calls/lookups
    HOME_DIR.get_or_init(|| {
        if cfg!(debug_assertions) {
            // Use a local version of home if we are running in dev
            return PathBuf::from("belle_home");
        }

        // Use a belle home location at environment variable or default to directory under the user's application data
        env::var("BELLE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::data_dir().expect("Could not get users data folder").join("belle"))
    })
}

impl BelleConfig {
    /// Get folder for manifest
    pub fn get_manifest_dir() -> PathBuf {
        get_home_dir().join("mft")
    }

    /// Get folder for theories
    pub fn get_package_dir() -> PathBuf {
        get_home_dir().join("pkg")
    }

    fn get_root_env_dir() -> PathBuf {
        get_home_dir().join("env")
    }

    /// Get folder for environments
    pub fn get_env_dir() -> PathBuf {
        Self::get_root_env_dir().join("envs")
    }

    /// Get folder for environments
    pub fn get_active_env_link() -> PathBuf {
        Self::get_root_env_dir().join("active")
    }
}
