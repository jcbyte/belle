use anyhow::Context;
use std::{
    env, fs,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

use crate::config::types::ConfigData;

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
        // Load config file from location at environment variable `BELLE_CONFIG` or check the executables location if that is not set
        let config_path = env::var("BELLE_CONFIG").unwrap_or(String::from("./belle_config.toml"));
        let config_file = PathBuf::from(&config_path);

        let parsed_config = if config_file.is_file() {
            let content = fs::read_to_string(&config_file)
                .with_context(|| format!("Failed to read config file at '{}'", config_file.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config file at '{}'", config_file.display()))?
        } else {
            // Use default values if the config is not found
            ConfigData::default()
        };

        return Ok(BelleConfig {
            data: parsed_config,
            config_file,
        });
    }

    /// Save config back to disk
    fn save(&self) -> anyhow::Result<()> {
        let content = toml::to_string(&self.data)?;
        fs::write(&self.config_file, content)?;
        return Ok(());
    }

    /// Initialise the config (should be called once)
    pub fn init() -> anyhow::Result<()> {
        let mgr = BelleConfig::load()?;
        CONFIG_INSTANCE
            .set(RwLock::new(mgr))
            .map_err(|_| anyhow::anyhow!("Init failed"))?;

        return Ok(());
    }

    // Global accessors
    pub fn read_config<R>(f: impl FnOnce(&ConfigData) -> R) -> R {
        let lock = CONFIG_INSTANCE.get().expect("Not init").read().unwrap();
        return f(&lock.data);
    }

    pub fn write_config<R>(f: impl FnOnce(&mut ConfigData) -> R) -> R {
        let mut lock = CONFIG_INSTANCE.get().expect("Not init").write().unwrap();
        let res = f(&mut lock.data);
        // Auto-save on write
        let _ = lock.save();
        return res;
    }
}
