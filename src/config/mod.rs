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
    pub fn init() -> anyhow::Result<()> {
        let config_path = env::var("BELLE_CONFIG").unwrap_or(String::from("./belle_config.toml"));
        let config_file = Path::new(&config_path);

        let parsed: BelleConfig = if config_file.is_file() {
            let content = fs::read_to_string(config_file)
                .with_context(|| format!("Failed to read config file at '{}'", config_path))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML config file at '{}'", config_path))?
        } else {
            BelleConfig::default()
        };

        CONFIG.set(parsed);
        return Ok(());
    }

    pub fn global() -> &'static Self {
        return CONFIG.get().expect("Config was not initialised");
    }

    fn default() -> Self {
        // ? how do I handle linux differently
        let base_path = env::var("APPDATA").map(PathBuf::from).unwrap();

        return Self {
            root_dir: base_path.join("belle"),
        };
    }
}
