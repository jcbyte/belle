use std::{fs, time::Duration};

use console::style;
use indicatif::ProgressBar;
use pubgrub::SemanticVersion;

use crate::{
    cli::{error::CliError, theming::ProgressBarTheme},
    environment::{Environment, VersionReq, error::EnvironmentError, manager},
    error::{AppError, CustomErrorContext, IoErrorContext},
    registry::{PackageIdentifier, error::RegistryNotExistContext},
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};

/// Apply any changes made to environment files, with logging
pub async fn finalise_env(env: &mut Environment, include_resolve: bool) -> Result<(), AppError> {
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
        let pb = ProgressBar::new(missing_packages.len() as u64).with_belle_style();

        for package in &missing_packages {
            pb.set_message(format!("Fetching{}", style(&package).cyan()));

            let package_meta = package
                .get_resolved_package_manifest()?
                .report_package_nonexistent(package.clone())?;

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

pub fn switch_env(name: Option<String>) -> Result<(), AppError> {
    let name = match name {
        Some(n) => n,
        None => {
            let frozen_env = Environment::frozen()?
                .report_custom("No name was given, and no lockfile is found in workspace to infer from.")?;
            frozen_env.name
        }
    };

    manager::switch_env(&name)?;

    println!("Switched to environment {}.", style(name).cyan().bold());
    Ok(())
}

pub async fn create_env(name: String, isabelle: Option<SemanticVersion>) -> Result<(), AppError> {
    let mut new_env = Environment::new(name.clone(), isabelle.into())?;

    finalise_env(&mut new_env, true).await?;

    println!("Created new environment: {}.", style(&name).cyan().bold());

    // Switch into the newly created environment, qol
    manager::switch_env(&name)?;

    println!("Switched to environment {}.", style(&name).cyan().bold());

    Ok(())
}

pub fn list_envs() -> Result<(), AppError> {
    let active_env = Environment::active()?.map(|active_env| active_env.name);

    let mut env_count = 0;
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

        env_count += 1;
    }

    println!("Found {} Environments.", style(env_count).bold());

    Ok(())
}

pub fn remove_env(name: &str) -> Result<(), AppError> {
    let env_dir = Environment::env_dir_for_name(name);

    if !env_dir.is_dir() {
        return Err(EnvironmentError::DoesNotExist { name: name.to_string() }.into());
    }

    fs::remove_dir_all(&env_dir).report_delete("environment directory", &env_dir)?;

    println!("Removed environment: {}.", style(name).cyan().bold());
    Ok(())
}

pub fn freeze_env() -> Result<(), AppError> {
    let active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;
    active_env.freeze()?;

    println!("Frozen environment to belle file.");
    Ok(())
}

pub async fn sync_env() -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.sync()?;
    // Don't resolve as we want to keep the lockfile identical
    finalise_env(&mut active_env, false).await?;

    println!("Synced environment from belle file.");
    Ok(())
}

pub async fn migrate_isabelle(version: Option<SemanticVersion>, unpin_existing: bool) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.migrate_isabelle(version.into(), unpin_existing);
    finalise_env(&mut active_env, true).await?;

    let (isabelle_version, given) = match active_env.isabelle {
        VersionReq::Given(version) => (version, true),
        VersionReq::Any => {
            let version = active_env
                .lock
                .get(ISABELLE_PACKAGE)
                // todo if no packages this could fail
                .ok_or(EnvironmentError::NoIsabelleVersion)?;
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
