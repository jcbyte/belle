use std::fs;

use crate::environment::Environment;

pub fn create_env(name: String) -> anyhow::Result<()> {
    Environment::new(name)?;
    return Ok(());
}

pub fn remove_env(name: &String) -> anyhow::Result<()> {
    let env_dir = Environment::env_dir_for_name(name);

    if env_dir.is_dir() {
        fs::remove_dir_all(env_dir)?;
    } else {
        return Err(anyhow::anyhow!("Environment '{}' cannot be found", name));
    }

    return Ok(());
}
