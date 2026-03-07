use std::{fs, time::Duration};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use pubgrub::SemanticVersion;

use crate::{
    environment::{Environment, manager},
    fetch::BelleClient,
    registry::PackageIdentifier,
};

/// Apply any changes made to environment files, with logging
pub async fn finalise_env(env: &mut Environment) -> anyhow::Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(format!("Resolving dependency list"));

    // Resolve lockfile dependencies
    env.resolve_lock()?;

    pb.finish_and_clear();

    // Fetch all packages we currently do not have
    let client = BelleClient::new()?;
    let missing_packages: Vec<PackageIdentifier> = env
        .iter_user_packages()
        .map(|(name, version)| PackageIdentifier {
            name: name.clone(),
            version: version.clone(),
        })
        // Filter to only retrieve missing packages
        .filter(|p| !p.exists_locally())
        .collect();

    if missing_packages.len() > 0 {
        let pb = ProgressBar::new(missing_packages.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner} [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
                .progress_chars("#>-"),
        );

        for package in &missing_packages {
            pb.set_message(format!("Fetching{}", style(&package).cyan()));

            let package_meta = package.get_resolved_package_manifest()?.ok_or_else(|| {
                anyhow::anyhow!(
                    "Package '{}' from environment cannot be found in local registry",
                    package
                )
            })?;

            package_meta.get_package(&client).await?;

            pb.inc(1);
        }

        pb.finish_with_message(format!(
            "Fetched '{}' new packages",
            style(missing_packages.len()).bold()
        ));
    }

    // Save environment back to file once this has completed, if any errors occur we will not reach this state
    // Hence environment will not be saved in a broken state.
    env.save()?;

    return Ok(());
}

fn get_env_name(name: Option<&String>) -> anyhow::Result<(String, bool)> {
    let name = match name {
        Some(n) => (n.clone(), false),
        None => {
            let frozen_env = Environment::frozen()?
                .ok_or_else(|| anyhow::anyhow!("No name given, and no belle file found in workspace."))?;
            (frozen_env.name.clone(), true)
        }
    };

    return Ok(name);
}

pub fn switch_env(name: Option<String>) -> anyhow::Result<()> {
    let (name, _using_frozen) = get_env_name(name.as_ref())?;

    manager::switch_env(&name)?;

    println!("Switched to environment {}.", style(name).cyan().bold());
    return Ok(());
}

pub async fn create_env(name: Option<String>, new: bool, isabelle: Option<SemanticVersion>) -> anyhow::Result<()> {
    let (env_name, using_frozen) = get_env_name(name.as_ref())?;

    if using_frozen && !new && isabelle.is_some() {
        anyhow::bail!("Isabelle version cannot be given when creating from an existing belle file.");
    }

    let mut new_env = Environment::new(env_name.clone(), isabelle.into())?;

    if using_frozen && new {
        // If created from a belle file, we want to sync this into the environment
        new_env.sync()?;
        finalise_env(&mut new_env).await?;
    } else {
        // Else just save it
        new_env.save()?;
    }

    println!("Created new environment: {}.", style(env_name).cyan().bold());

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
        anyhow::bail!("Environment '{}' cannot be found.", name);
    }

    fs::remove_dir_all(env_dir)?;

    println!("Removed environment: {}.", style(name).cyan().bold());
    return Ok(());
}

pub fn freeze_env() -> anyhow::Result<()> {
    let active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.freeze()?;

    println!("Frozen environments to belle file.");
    return Ok(());
}

pub async fn sync_env() -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No selected environment"))?;

    active_env.sync()?;
    // todo This will also resolve the lockfile again, do we want that?
    finalise_env(&mut active_env).await?;

    println!("Synced environment from belle file.");
    return Ok(());
}

pub async fn migrate_isabelle(version: Option<SemanticVersion>, unpin_existing: bool) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;

    active_env.migrate_isabelle(version.into(), unpin_existing)?;
    finalise_env(&mut active_env).await?;

    // todo display this properly (name and version) and get version if its not given
    println!("Migrated Isabelle to {:?}.", style(version).cyan());
    return Ok(());
}
