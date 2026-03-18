use std::{
    env, fs,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

use crate::{
    config::types::{ConfigData, default_home_dir},
    error::{AppError, IoErrorContext, ParseErrorContext},
    util::create_parent_dirs,
};

#[derive(Debug)]
pub struct BelleConfig {
    data: ConfigData,
    config_file: PathBuf,
}

/// Global config instance
static CONFIG_INSTANCE: OnceLock<RwLock<BelleConfig>> = OnceLock::new();

impl BelleConfig {
    /// Load config from disk, or use default
    fn load() -> Result<Self, AppError> {
        let config_path = if cfg!(debug_assertions) {
            // Use a local version of the config if we are running in dev
            PathBuf::from("belle_config.toml")
        } else {
            // Load config file from location at environment variable `BELLE_CONFIG` or use the home directory if that is not set
            env::var("BELLE_CONFIG")
                .map(PathBuf::from)
                .unwrap_or_else(|_| default_home_dir().join("config.toml"))
        };

        let parsed_config = if config_path.is_file() {
            let content = fs::read_to_string(&config_path).report_read("config file", &config_path)?;
            toml::from_str(&content).report_file("config file", &config_path)?
        } else {
            // Use default values if the config is not found
            ConfigData::default()
        };

        let config = BelleConfig {
            data: parsed_config,
            config_file: config_path.clone(),
        };

        // Save the config back to disk to place defaults on disk
        create_parent_dirs(&config_path).report_save("config file directories", &config_path)?;
        config.save()?;

        Ok(config)
    }

    /// Save config back to disk
    fn save(&self) -> Result<(), AppError> {
        let content = toml::to_string(&self.data).report_file("config file", &self.config_file)?;
        fs::write(&self.config_file, content).report_save("config file", &self.config_file)?;

        Ok(())
    }

    /// Initialise the config (should be called once)
    pub fn init() -> Result<(), AppError> {
        let mgr = BelleConfig::load()?;
        CONFIG_INSTANCE.set(RwLock::new(mgr)).expect("Config is already initialised");
        Ok(())
    }

    // Global accessors
    pub fn read_config<R>(f: impl FnOnce(&ConfigData) -> R) -> R {
        let lock = CONFIG_INSTANCE.get().expect("Config is not initialised").read().unwrap();
        f(&lock.data)
    }

    pub fn write_config<R>(f: impl FnOnce(&mut ConfigData) -> R) -> R {
        let mut lock = CONFIG_INSTANCE.get().expect("Config is not initialised").write().unwrap();
        let res = f(&mut lock.data);
        // Auto-save on write
        let _ = lock.save();
        res
    }
}
