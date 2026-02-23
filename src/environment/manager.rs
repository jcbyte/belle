use std::fs;

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(windows)]
use junction::create as symlink;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    config::BelleConfig,
    environment::{EnvManager, Environment},
};

pub fn switch_env(name: &String) -> anyhow::Result<()> {
    let active_env_link = BelleConfig::read_config(|c| c.get_active_env_link());
    let active_env = Environment::env_dir_for_name(name);

    if !active_env.is_dir() {
        return Err(anyhow::anyhow!("Environment '{}' cannot be found", name));
    }

    symlink(active_env, active_env_link).context("Failed to create junction/symlink for active environment")?;

    return Ok(());
}

pub fn get_active_env() -> anyhow::Result<Option<String>> {
    let active_env_link = BelleConfig::read_config(|c| c.get_active_env_link());

    // Environment::load(name);

    todo!();

    return Ok(None);
}

pub fn get_envs() -> Vec<String> {
    let env_dir = BelleConfig::read_config(|c| c.get_env_dir());

    return WalkDir::new(env_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .map(|env_dir| env_dir.file_name().to_string_lossy().to_string())
        .collect();
}
