use std::fs;

use console::style;
use pubgrub::SemanticVersion;

use crate::{
    config::BelleConfig,
    environment::{Environment, PackageType, manager},
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};

fn get_env_name(name: Option<&String>) -> anyhow::Result<(String, bool)> {
    let name = match name {
        Some(n) => (n.clone(), false),
        None => {
            let frozen_env = Environment::frozen()?
                .ok_or_else(|| anyhow::anyhow!("No name given, and no belle file found in workspace"))?;
            (frozen_env.name.clone(), true)
        }
    };

    return Ok(name);
}

pub fn switch_env(name: Option<String>) -> anyhow::Result<()> {
    let (name, _using_frozen) = get_env_name(name.as_ref())?;

    manager::switch_env(&name)?;
    println!("Switched to environment {}", style(name).cyan().bold());
    return Ok(());
}

pub fn create_env(name: Option<String>, new: bool, isabelle: Option<SemanticVersion>) -> anyhow::Result<()> {
    let (env_name, using_frozen) = get_env_name(name.as_ref())?;

    if using_frozen && !new && isabelle.is_some() {
        anyhow::bail!("Isabelle version cannot be given when creating from an existing belle file");
    }

    let mut new_env = Environment::new(env_name.clone(), isabelle.into())?;
    println!("Created new environment: {}", style(env_name).cyan().bold());

    // If created from a belle file, we want to sync this into the environment
    if using_frozen && new {
        new_env.sync()?;
    }

    return Ok(());
}

pub fn list_envs() -> anyhow::Result<()> {
    let active_env = manager::get_active_env()?;

    for env in manager::iter_envs() {
        let env_line = if active_env.as_deref() == Some(env.as_str()) {
            format!(
                "{} {:<9} {}",
                style("*").cyan().bold(),
                style(&env).cyan().bold(),
                style("[active]").dim()
            )
        } else {
            format!("  {:<9}", &env)
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

pub fn freeze_env() -> anyhow::Result<()> {
    let active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.freeze()?;

    return Ok(());
}

pub fn sync_env() -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No selected environment"))?;
    active_env.sync()?;

    return Ok(());
}

pub fn list_packages(all: bool) -> anyhow::Result<()> {
    let active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;

    let packages = active_env.get_packages()?;
    let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

    // Partition these
    let mut isabelle_listing = None;
    let mut dependencies = Vec::new();
    let mut transitive_dependencies = Vec::new();
    let mut isabelle_dependencies = Vec::new();

    for dependency in packages {
        match dependency.kind {
            PackageType::Direct { .. } => dependencies.push(dependency),
            PackageType::Transitive => {
                if dependency.name.eq(ISABELLE_PACKAGE) {
                    isabelle_listing = Some(dependency);
                } else if isabelle_packages.contains(&dependency.name) {
                    isabelle_dependencies.push(dependency);
                } else {
                    transitive_dependencies.push(dependency);
                }
            }
        }
    }

    let isabelle_version = isabelle_listing
        .ok_or(anyhow::anyhow!("Isabelle version could not be found"))?
        .version;

    println!("Environment: {}", style(active_env.name).cyan());

    println!(
        "{} {} {}{}{}",
        style("* Isabelle:").bold(),
        style(get_isabelle_name(&isabelle_version)).cyan().bold(),
        style("[").dim(),
        style(isabelle_version.to_string()).green(),
        style("]").dim(),
    );

    for package in dependencies {
        let version = style(package.version.to_string());
        let styled_version = match package.kind {
            PackageType::Direct { given_version: true } => version.green(),
            _ => version.dim(),
        };

        println!(
            "- {} {}{}{}",
            style(package.name),
            style("[").dim(),
            styled_version,
            style("]").dim()
        )
    }

    if all {
        for package in transitive_dependencies {
            println!(
                "- {} {}{}{}",
                style(package.name).dim(),
                style("[").dim(),
                style(package.version).dim(),
                style("]").dim()
            )
        }

        for package in isabelle_dependencies {
            println!(
                "- {} {}{}{}",
                style(package.name).dim().italic(),
                style("[").dim(),
                style(package.version).dim(),
                style("]").dim()
            )
        }
    }

    return Ok(());
}

pub fn migrate_isabelle(version: Option<SemanticVersion>, unpin_existing: bool) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;

    active_env.migrate_isabelle(version.into(), unpin_existing)?;

    return Ok(());
}
