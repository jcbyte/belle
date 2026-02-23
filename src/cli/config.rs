use std::path::PathBuf;

use anyhow::anyhow;
use console::style;

use crate::config::BelleConfig;

fn display_row(key: &str, value: &str) {
    println!("{:<8} {}", style(key).cyan().bold(), value);
}

pub fn print_all_config() {
    BelleConfig::read_config(|c| {
        display_row("home", &c.home.to_string_lossy().to_string());
    });
}

pub fn print_config(key: &str) -> anyhow::Result<()> {
    let value = BelleConfig::read_config(|c| match key {
        "home" => Ok(c.home.to_string_lossy().to_string()),
        _ => Err(anyhow!("Unknown configuration parameter '{}'", key)),
    })?;

    display_row(key, &value);
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

    println!("Updated {} {}", style(key).bold(), style(format!("-> {}", value)).dim());

    return Ok(());
}
