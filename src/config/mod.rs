use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::Context;
use serde::Deserialize;

pub static CONFIG: OnceLock<BelleConfig> = OnceLock::new();

#[derive(Deserialize, Debug)]
pub struct BelleConfig {
    pub root_dir: PathBuf,
}

impl BelleConfig {
    /// Initialise config, this must only be called once
    pub fn init() -> anyhow::Result<()> {
        // Load config file from location at environment variable `BELLE_CONFIG` or check the executables location if that is not set
        let config_path = env::var("BELLE_CONFIG").unwrap_or(String::from("./belle_config.toml"));
        let config_file = Path::new(&config_path);

        // Use default values if the config is not found
        let parsed: BelleConfig = if config_file.is_file() {
            let content = fs::read_to_string(config_file)
                .with_context(|| format!("Failed to read config file at '{}'", config_path))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config file at '{}'", config_path))?
        } else {
            BelleConfig::default()
        };

        CONFIG.set(parsed).expect("Config has already been initialised");
        return Ok(());
    }

    /// Return the global configuration, this must be called after `init`
    pub fn global() -> &'static Self {
        // This will panic if `init()` was not called, so we must ensure this was done in our entry point
        return CONFIG.get().expect("Config was not initialised");
    }

    /// Defaults for config
    fn default() -> Self {
        // todo handle linux differently
        // Default root directory under the user's application data
        let base_path = env::var("APPDATA").map(PathBuf::from).unwrap();

        return Self {
            root_dir: base_path.join("belle"),
        };
    }
}

// todo writing to config
