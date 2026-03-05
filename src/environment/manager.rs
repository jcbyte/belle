use std::fs;

#[cfg(windows)]
use junction::create as symlink;
#[cfg(unix)]
use std::os::unix::fs::symlink;

use anyhow::Context;
use walkdir::WalkDir;

use crate::{config::BelleConfig, environment::Environment};

pub fn switch_env(name: &String) -> anyhow::Result<()> {
    let active_env_link = BelleConfig::read_config(|c| c.get_active_env_link());
    let active_env = Environment::env_dir_for_name(name);

    if !active_env.is_dir() {
        anyhow::bail!("Environment '{}' cannot be found", name);
    }

    // Create a temporary symlink and overwrite to avoid `AlreadyExists` errors
    let temp_link = active_env_link.with_added_extension("tmp");
    symlink(active_env, &temp_link).context("Failed to create junction/symlink for active environment")?;
    fs::rename(temp_link, active_env_link)
        .context("Failed to overwrite existing junction/symlink for the active environment")?;

    return Ok(());
}

pub fn get_active_env() -> anyhow::Result<Option<String>> {
    let active_env = Environment::active()?;
    return Ok(active_env.map(|env| env.name.clone()));
}

pub fn iter_envs() -> impl Iterator<Item = String> {
    let env_dir = BelleConfig::read_config(|c| c.get_env_dir());

    return WalkDir::new(env_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .map(|env_dir| env_dir.file_name().to_string_lossy().to_string());
}
