use std::path::PathBuf;

use anyhow::anyhow;

use crate::config::BelleConfig;

pub fn print_all_config() -> anyhow::Result<()> {
    todo!(); // todo implement
}

pub fn print_config(key: &str) -> anyhow::Result<()> {
    let value = BelleConfig::read_config(|c| match key {
        "home" => Ok(c.home.to_string_lossy().to_string()),
        _ => Err(anyhow!("Unknown configuration parameter '{}'", key)),
    })?;

    println!("{} is {}", key, value); // todo formatting
    return Ok(());
}

pub fn set_config(key: &str, value: &String) -> anyhow::Result<()> {
    BelleConfig::write_config(|c| match key {
        "home" => {
            c.home = PathBuf::from(value);
            Ok(())
        }
        _ => Err(anyhow!("Unknown configuration parameter '{}'", key)),
    })?;

    // todo response

    return Ok(());
}
