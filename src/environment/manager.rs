use std::{
    fs::{self, File},
    path::Path,
};

#[cfg(windows)]
use junction::create as symlink;
#[cfg(unix)]
use std::os::unix::fs::symlink;

use walkdir::WalkDir;

use crate::{
    config::BelleConfig,
    environment::{Environment, error::EnvironmentError},
    error::{AppError, IoError, IoErrorContext},
    util::create_parent_dirs,
};

fn overwrite_link(original: &Path, link: &Path) -> Result<(), IoError> {
    // Create a temporary symlink and overwrite to avoid `AlreadyExists` errors
    let temp_link = link.with_added_extension("tmp");
    symlink(original, &temp_link).report_save("active environment symlink", &temp_link)?;
    fs::rename(temp_link, link).report_save("active environment symlink", link)?;

    Ok(())
}

pub fn switch_env(name: &str) -> Result<(), AppError> {
    let active_env_link = BelleConfig::get_active_env_link();
    let switching_env = Environment::env_dir_for_name(name);

    if !switching_env.is_dir() {
        return Err(EnvironmentError::DoesNotExist { name: name.to_string() }.into());
    }

    overwrite_link(&switching_env, &active_env_link)?;

    Ok(())
}

pub fn set_env_none() -> Result<(), IoError> {
    let active_env_link = BelleConfig::get_active_env_link();
    let none_env = BelleConfig::get_none_env();

    // If the null environment hasn't been set up then do this now
    if !none_env.is_dir() {
        let none_env_root = none_env.join("ROOT");
        create_parent_dirs(&none_env_root).report_save("none environment root directories", &none_env_root)?;
        File::create(&none_env_root).report_save("none environment root", &none_env_root)?;
    };

    overwrite_link(&none_env, &active_env_link)?;

    Ok(())
}

pub fn iter_envs() -> impl Iterator<Item = String> {
    let env_dir = BelleConfig::get_env_dir();

    WalkDir::new(env_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .map(|env_dir| env_dir.file_name().to_string_lossy().to_string())
}
