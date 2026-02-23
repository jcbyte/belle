use std::path::PathBuf;

use anyhow::anyhow;
use console::style;

use crate::config::BelleConfig;

/// Display a single key-value pair
fn display_row(key: &str, value: &str) {
    println!("{:<10} {}", style(key).cyan().bold(), value);
}

/// List all configuration parameters
pub fn print_all_config() {
    BelleConfig::read_config(|c| {
        display_row("home", &c.home.to_string_lossy().to_string());
        display_row("afp-group", &c.afp_group);
    });
}

/// List a single configuration parameter
pub fn print_config(key: &str) -> anyhow::Result<()> {
    let value = BelleConfig::read_config(|c| match key {
        "home" => Ok(c.home.to_string_lossy().to_string()),
        "afp-group" => Ok(c.afp_group.to_owned()),
        _ => Err(anyhow!("Unknown configuration parameter '{}'", key)),
    })?;

    display_row(key, &value);
    return Ok(());
}

/// Set a configuration parameter
pub fn set_config(key: &str, value: &String) -> anyhow::Result<()> {
    BelleConfig::write_config(|c| match key {
        "home" => {
            c.home = PathBuf::from(value);
            Ok(())
        }
        "afp-group" => {
            c.afp_group = value.to_owned();
            Ok(())
        }
        _ => Err(anyhow!("Unknown configuration parameter '{}'", key)),
    })?;

    println!("Updated {} {}", style(key).bold(), style(format!("-> {}", value)).dim());

    return Ok(());
}

// todo have hint of what it is, also can this be done with less repetition
