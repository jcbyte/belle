use std::fs;

#[cfg(windows)]
use junction::create as symlink;
#[cfg(unix)]
use std::os::unix::fs::symlink;

use walkdir::WalkDir;

use crate::{
    config::BelleConfig,
    environment::{Environment, error::EnvironmentError},
    error::{AppError, IoErrorContext},
};

pub fn switch_env(name: &str) -> Result<(), AppError> {
    let active_env_link = BelleConfig::read_config(|c| c.get_active_env_link());
    let active_env = Environment::env_dir_for_name(name);

    if !active_env.is_dir() {
        return Err(EnvironmentError::DoesNotExist { name: name.to_string() }.into());
    }

    // Create a temporary symlink and overwrite to avoid `AlreadyExists` errors
    let temp_link = active_env_link.with_added_extension("tmp");
    symlink(active_env, &temp_link).report_save("active environment symlink", &temp_link)?;
    fs::rename(temp_link, &active_env_link).report_save("active environment symlink", &active_env_link)?;

    Ok(())
}

pub fn iter_envs() -> impl Iterator<Item = String> {
    let env_dir = BelleConfig::read_config(|c| c.get_env_dir());

    WalkDir::new(env_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .map(|env_dir| env_dir.file_name().to_string_lossy().to_string())
}
