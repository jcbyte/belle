use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::{config::BelleConfig, environment::Environment};

impl Environment {
    pub fn new(name: String) -> anyhow::Result<Self> {
        let env = Environment { name, packages: vec![] };
        env.save()?;
        return Ok(env);
    }

    fn env_dir_for_name(name: &String) -> PathBuf {
        return BelleConfig::read_config(|c| c.get_env_dir()).join(name);
    }

    fn env_file_for_name(name: &String) -> PathBuf {
        return Self::env_dir_for_name(name).join("env.toml");
    }

    fn get_env_dir(&self) -> PathBuf {
        return Self::env_dir_for_name(&self.name);
    }

    fn get_env_file(&self) -> PathBuf {
        return Self::env_file_for_name(&self.name);
    }

    pub fn load(name: String) -> anyhow::Result<Self> {
        let env_file = Self::env_file_for_name(&name);

        let parsed_env = if env_file.is_file() {
            let content = fs::read_to_string(&env_file)
                .with_context(|| format!("Failed to read environment file at '{}'", env_file.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML environment file at '{}'", env_file.display()))?
        } else {
            return Err(anyhow::anyhow!("Environments '{}' does not exist", &name));
        };

        return Ok(parsed_env);
    }

    fn save(&self) -> anyhow::Result<()> {
        let env_file = self.get_env_file();

        // Recursively create parent directory and parents so that we can write to the file
        if let Some(parent) = env_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create {} environment directories on disk", &self.name))?;
        }

        let content =
            toml::to_string(self).with_context(|| format!("Failed to parse TOML for environment '{}'", &self.name))?;
        fs::write(env_file, content).with_context(|| format!("Failed to save environment '{}'", &self.name))?;

        return Ok(());
    }

    pub fn is_active(&self) -> bool {
        todo!()
    }

    pub fn switch(&self) {
        todo!();
    }

    pub fn add_package(&mut self) {
        todo!();
    }

    pub fn to_roots(&self) {
        todo!();
    }
}
