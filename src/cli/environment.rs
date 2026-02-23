use std::fs;

use console::style;

use crate::environment::{Environment, manager};

pub fn switch_env(name: &String) -> anyhow::Result<()> {
    manager::switch_env(name)?;
    println!("Switched to environment {}", style(name).cyan().bold());
    return Ok(());
}

pub fn create_env(name: String) -> anyhow::Result<()> {
    Environment::new(name.clone())?;
    println!("Created new environment: {}", style(name).cyan().bold());
    return Ok(());
}

pub fn list_envs() -> anyhow::Result<()> {
    let envs = manager::get_envs();
    let active_env = manager::get_active_env()?;

    for env in envs {
        let env_line = if active_env.as_deref() == Some(env.as_str()) {
            style(format!("* {}", &env)).cyan().bold()
        } else {
            style(format!("  {}", &env))
        };
        println!("{}", env_line);
    }

    return Ok(());
}

pub fn remove_env(name: &String) -> anyhow::Result<()> {
    let env_dir = Environment::env_dir_for_name(name);

    if !env_dir.is_dir() {
        anyhow::bail!("Environment '{}' cannot be found", name);
    }

    fs::remove_dir_all(env_dir)?;
    println!("Removed environment: {}", style(name).cyan().bold());

    return Ok(());
}
