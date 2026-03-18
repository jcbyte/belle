use std::{fs, time::Duration};

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use pubgrub::SemanticVersion;

use crate::{
    environment::{Environment, VersionReq, manager},
    registry::PackageIdentifier,
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};

/// Apply any changes made to environment files, with logging
pub async fn finalise_env(env: &mut Environment, include_resolve: bool) -> anyhow::Result<()> {
    // Don't resolve if we want to skip it
    if include_resolve {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Resolving dependency list".to_string());

        // Resolve lockfile dependencies
        env.resolve_lock()?;

        pb.finish_and_clear();
    }

    // Fetch all packages we currently do not have
    let missing_packages: Vec<PackageIdentifier> = env
        .iter_user_packages()
        .map(|(name, version)| PackageIdentifier::new(name, *version))
        // Filter to only retrieve missing packages
        .filter(|p| !p.exists_locally())
        .collect();

    if !missing_packages.is_empty() {
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

            package_meta.get_package().await?;

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

    // Update the ROOTS file to match new environment
    env.create_roots_file()?;

    Ok(())
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

    Ok(name)
}

pub fn switch_env(name: Option<String>) -> anyhow::Result<()> {
    let (name, _using_frozen) = get_env_name(name.as_ref())?;

    manager::switch_env(&name)?;

    println!("Switched to environment {}.", style(name).cyan().bold());
    Ok(())
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
        // Don't resolve as we want to keep the lockfile identical
        finalise_env(&mut new_env, false).await?;
    } else {
        // Else just save the environment and generate a blank ROOTS file
        new_env.save()?;
        new_env.create_roots_file()?;
    }

    println!("Created new environment: {}.", style(env_name).cyan().bold());

    Ok(())
}

pub fn list_envs() -> anyhow::Result<()> {
    let active_env = Environment::active()?.map(|active_env| active_env.name);

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

    Ok(())
}

pub fn remove_env(name: &String) -> anyhow::Result<()> {
    let env_dir = Environment::env_dir_for_name(name);

    if !env_dir.is_dir() {
        anyhow::bail!("Environment '{}' cannot be found.", name);
    }

    fs::remove_dir_all(env_dir)?;

    println!("Removed environment: {}.", style(name).cyan().bold());
    Ok(())
}

pub fn freeze_env() -> anyhow::Result<()> {
    let active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;
    active_env.freeze()?;

    println!("Frozen environments to belle file.");
    Ok(())
}

pub async fn sync_env() -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No selected environment"))?;

    active_env.sync()?;
    // Don't resolve as we want to keep the lockfile identical
    finalise_env(&mut active_env, false).await?;

    println!("Synced environment from belle file.");
    Ok(())
}

pub async fn migrate_isabelle(version: Option<SemanticVersion>, unpin_existing: bool) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;

    active_env.migrate_isabelle(version.into(), unpin_existing);
    finalise_env(&mut active_env, true).await?;

    let (isabelle_version, given) = match active_env.isabelle {
        VersionReq::Given(version) => (version, true),
        VersionReq::Any => {
            let version = active_env
                .lock
                .get(ISABELLE_PACKAGE)
                .ok_or(anyhow::anyhow!("No Isabelle version is given for the environment"))?;
            (*version, false)
        }
    };

    let mut formatted_version = style(isabelle_version);
    formatted_version = if given {
        formatted_version.green()
    } else {
        formatted_version.dim()
    };

    println!(
        "Migrated Isabelle to {} {}{}{}.",
        style(get_isabelle_name(&isabelle_version)).cyan().bold(),
        style("[").dim(),
        formatted_version,
        style("]").dim()
    );
    Ok(())
}
