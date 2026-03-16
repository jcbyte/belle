use anyhow::Context;
use std::{
    env, fs,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

use crate::config::types::{ConfigData, default_home_dir};

#[derive(Debug)]
pub struct BelleConfig {
    data: ConfigData,
    config_file: PathBuf,
}

/// Global config instance
static CONFIG_INSTANCE: OnceLock<RwLock<BelleConfig>> = OnceLock::new();

impl BelleConfig {
    /// Load config from disk, or use default
    fn load() -> anyhow::Result<Self> {
        let config_path = if cfg!(debug_assertions) {
            // Use a local version of the config if we are running in dev
            PathBuf::from("belle_config.toml")
        } else {
            // Load config file from location at environment variable `BELLE_CONFIG` or use the home directory if that is not set
            env::var("BELLE_CONFIG")
                .map(|path| PathBuf::from(path))
                .unwrap_or_else(|_| default_home_dir().join("config.toml"))
        };

        let parsed_config = if config_path.is_file() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file at '{}'", config_path.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config file at '{}'", config_path.display()))?
        } else {
            // Use default values if the config is not found
            ConfigData::default()
        };

        let config = BelleConfig {
            data: parsed_config,
            config_file: config_path.clone(),
        };

        // Save the config back to disk to place defaults on disk
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create directories to contain config file on disk"))?;
        }
        config.save()?;

        Ok(config)
    }

    /// Save config back to disk
    fn save(&self) -> anyhow::Result<()> {
        let content = toml::to_string(&self.data)?;
        fs::write(&self.config_file, content)?;
        Ok(())
    }

    /// Initialise the config (should be called once)
    pub fn init() -> anyhow::Result<()> {
        let mgr = BelleConfig::load()?;
        CONFIG_INSTANCE
            .set(RwLock::new(mgr))
            .map_err(|_| anyhow::anyhow!("Init failed"))?;

        Ok(())
    }

    // Global accessors
    pub fn read_config<R>(f: impl FnOnce(&ConfigData) -> R) -> R {
        let lock = CONFIG_INSTANCE.get().expect("Not init").read().unwrap();
        f(&lock.data)
    }

    pub fn write_config<R>(f: impl FnOnce(&mut ConfigData) -> R) -> R {
        let mut lock = CONFIG_INSTANCE.get().expect("Not init").write().unwrap();
        let res = f(&mut lock.data);
        // Auto-save on write
        let _ = lock.save();
        res
    }
}
