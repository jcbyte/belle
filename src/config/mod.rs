use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigData {
    home: PathBuf,
}

impl Default for ConfigData {
    /// Defaults for config
    fn default() -> Self {
        // Default root directory under the user's application data
        let data_dir = dirs::data_dir().expect("Could not get users data folder");
        let home_dir = data_dir.join("belle");

        return Self { home: home_dir };
    }
}

#[derive(Debug)]
pub struct BelleConfig {
    data: ConfigData,
    config_file: PathBuf,
}

impl BelleConfig {
    fn new() -> anyhow::Result<Self> {
        // Load config file from location at environment variable `BELLE_CONFIG` or check the executables location if that is not set
        let config_path = env::var("BELLE_CONFIG").unwrap_or(String::from("./belle_config.toml"));
        let config_file = Path::new(&config_path);

        let parsed_config = if config_file.is_file() {
            let content = fs::read_to_string(&config_file).with_context(|| {
                format!(
                    "Failed to read config file at '{}'",
                    config_file.to_string_lossy().to_string()
                )
            })?;
            toml::from_str(&content).with_context(|| {
                format!(
                    "Failed to parse TOML config file at '{}'",
                    config_file.to_string_lossy().to_string()
                )
            })?
        } else {
            // Use default values if the config is not found
            ConfigData::default()
        };

        return Ok(BelleConfig {
            data: parsed_config,
            config_file: config_file.to_path_buf(),
        });
    }

    fn save(&self) -> anyhow::Result<()> {
        let content = toml::to_string(&self.data)?;
        fs::write(&self.config_file, content);
        return Ok(());
    }
}

impl BelleConfig {
    fn get_config() -> &'static RwLock<BelleConfig> {
        return CONFIG_INSTANCE.get().expect("Config must be initialised before use");
    }

    fn get_config_read() -> RwLockReadGuard<'static, BelleConfig> {
        return Self::get_config().read().expect("Config is poised");
    }

    fn get_config_write() -> RwLockWriteGuard<'static, BelleConfig> {
        return Self::get_config().write().expect("Config is poised");
    }

    pub fn get_home() -> PathBuf {
        let config = Self::get_config_read();
        return config.data.home.to_path_buf();
    }

    pub fn set_home(new_home: PathBuf) {
        let mut config = Self::get_config_write();
        config.data.home = new_home;
        config.save();
    }

    /// Get folder for metadata
    pub fn get_meta_dir() -> PathBuf {
        let config = Self::get_config_read();
        return config.data.home.join("meta");
    }

    /// Get folder for manifest
    pub fn get_manifest_dir() -> PathBuf {
        let config = Self::get_config_read();
        return config.data.home.join("manifest");
    }

    /// Get folder for theories
    pub fn get_theory_dir() -> PathBuf {
        let config = Self::get_config_read();
        return config.data.home.join("theory");
    }
}

// Global config instance
static CONFIG_INSTANCE: OnceLock<RwLock<BelleConfig>> = OnceLock::new();

pub fn init_config() -> anyhow::Result<()> {
    let config = BelleConfig::new()?;
    CONFIG_INSTANCE
        .set(RwLock::new(config))
        .expect("Config has already been initialised");

    return Ok(());
}
