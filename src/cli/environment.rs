use std::fs;

use console::style;

use crate::environment::{Environment, manager};

pub fn switch_env(name: &String) -> anyhow::Result<()> {
    manager::switch_env(name)?;
    return Ok(());
}

pub fn create_env(name: String) -> anyhow::Result<()> {
    Environment::new(name)?;
    return Ok(());
}

pub fn list_envs() -> anyhow::Result<()> {
    let envs = manager::get_envs();
    let active_env = manager::get_active_env()?;

    for env in envs {
        if active_env.as_deref().eq(&Some(env.as_str())) {
            print!("*");
        } else {
            print!(" ");
        }
        println!(" {:<9}", style(&env),);
    }

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
