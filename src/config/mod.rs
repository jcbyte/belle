use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigData {
    pub home: PathBuf,
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
    fn load() -> anyhow::Result<Self> {
        // Load config file from location at environment variable `BELLE_CONFIG` or check the executables location if that is not set
        let config_path = env::var("BELLE_CONFIG").unwrap_or(String::from("./belle_config.toml"));
        let config_file = PathBuf::from(&config_path);

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
            config_file,
        });
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let content = toml::to_string(&self.data)?;
        fs::write(&self.config_file, content)?;
        return Ok(());
    }
}

// Global config instance
static CONFIG_INSTANCE: OnceLock<RwLock<BelleConfig>> = OnceLock::new();

impl BelleConfig {
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

impl ConfigData {
    /// Retrieve string value of config setting, for CLI
    pub fn get(&self, key: &str) -> anyhow::Result<String> {
        match key {
            "home" => Ok(self.home.to_string_lossy().to_string()),
            _ => Err(anyhow::anyhow!("Unknown config setting '{}'", key)),
        }
    }

    /// Set string value of key, for CLI
    pub fn set(&mut self, key: &str, value: &String) -> anyhow::Result<()> {
        match key {
            "home" => {
                self.home = PathBuf::from(value);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Unknown config setting '{}'", key)),
        }
    }
}

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
