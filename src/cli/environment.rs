use std::{fs, time::Duration};

use console::style;
use indicatif::ProgressBar;
use tokio::time::sleep;

use crate::{
    cli::{CliLine, DisplayVersion, ProgressBarTheme, environment, error::CliError, pluralise},
    config::BelleConfig,
    environment::{Environment, LOCKFILE_NAME, VersionReq, error::EnvironmentError, manager},
    error::{AppError, CustomErrorContext, IoErrorContext},
    registry::{PackageIdentifier, error::RegistryNotExistContext},
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};

#[derive(PartialEq, Eq)]
pub enum FinalizeStrategy {
    /// Resolve lockfile and fetch
    ResolveAndApply,
    /// Fetch only
    ApplyOnly,
}

/// Apply any changes made to environment files, with logging
pub async fn finalise_env(env: &mut Environment, strategy: FinalizeStrategy) -> Result<(), AppError> {
    // Don't resolve if we want to skip it
    if strategy == FinalizeStrategy::ResolveAndApply {
        let pb = ProgressBar::new_spinner().with_belle_spinner_style();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_prefix(CliLine::style_focus_prefix("Resolving").to_string());
        pb.set_message("dependencies".to_string());

        // Resolve lockfile dependencies
        env.resolve_lock()?;

        sleep(Duration::from_secs(10)).await;

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
        let pb = ProgressBar::new(missing_packages.len() as u64).with_belle_bar_style();
        pb.set_prefix(CliLine::style_focus_prefix("Fetching").to_string());

        for package in &missing_packages {
            pb.set_message(format!("{}", package.styled()));

            let package_meta = package
                .get_resolved_package_manifest()?
                .report_package_nonexistent(package.clone())?;

            package_meta.get_package().await?;

            pb.inc(1);
        }

        pb.finish_and_clear();

        CliLine::new()
            .prefix("Fetched")
            .line(format!(
                "{} new {}",
                style(missing_packages.len()).bold(),
                pluralise(missing_packages.len(), "package", "packages")
            ))
            .as_success()
            .print();
    }

    // Save environment back to file once this has completed, if any errors occur we will not reach this state
    // Hence environment will not be saved in a broken state.
    env.save()?;

    // Update the ROOTS file to match new environment
    env.create_roots_file()?;

    Ok(())
}

/// Display a warning if there is no linked isabelle matching the active environment
pub fn warn_no_isabelle() -> Result<(), AppError> {
    let active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    if let VersionReq::Given(v) = active_env.get_isabelle_version()
        && !BelleConfig::read_config(|c| c.isabelles.contains_key(&v))
    {
        CliLine::new()
            .line(format!(
                "This environment uses Isabelle {} {}, but that version is not linked",
                get_isabelle_name(&v),
                DisplayVersion::Explicit(&v)
            ))
            .as_warning()
            .print();
    }

    Ok(())
}

pub fn get_isabelle_version<'a>(env: &'a Environment) -> Option<DisplayVersion<'a>> {
    match &env.isabelle {
        VersionReq::Given(v) => Some(DisplayVersion::Explicit(v)),
        VersionReq::Any => env.lock.get(ISABELLE_PACKAGE).map(DisplayVersion::Implicit),
    }
}

pub fn switch_env(name: Option<String>) -> Result<(), AppError> {
    let name = match name {
        Some(n) => n,
        None => {
            let frozen_env = Environment::frozen()?
                .report_custom("No name was provided, and no lockfile was found to infer from")?;
            frozen_env.name
        }
    };

    manager::switch_env(&name)?;

    CliLine::new()
        .prefix("Switched")
        .line(format!("to environment {}", style(name).cyan()))
        .as_success()
        .print();

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub fn create_env(name: String, isabelle: VersionReq) -> Result<(), AppError> {
    let new_env = Environment::new(name.clone(), isabelle)?;

    // Save the new environment
    new_env.save()?;

    // Create empty roots file
    new_env.create_roots_file()?;

    CliLine::new()
        .prefix("Created")
        .line(format!("environment {}", style(&name).cyan().bright()))
        .as_success()
        .print();

    // Switch into the newly created environment, qol
    manager::switch_env(&name)?;

    CliLine::new()
        .prefix("Switched")
        .line(format!("to environment {}", style(&name).cyan().bright()))
        .as_success()
        .print();

    Ok(())
}

pub fn list_envs() -> Result<(), AppError> {
    let active_env = Environment::active()?.map(|active_env| active_env.name);

    let mut env_count = 0;
    for env in manager::iter_envs() {
        if active_env.as_ref() == Some(&env) {
            // If this is the active environment then highlight it
            CliLine::new().prefix("Active").line(env).as_focus()
        } else {
            CliLine::new().line(&env)
        }
        .print();

        env_count += 1;
    }

    CliLine::new()
        .prefix("Total")
        .line(format!(
            "{} {}",
            style(env_count).bold(),
            pluralise(env_count, "environment", "environments")
        ))
        .as_success()
        .print();

    Ok(())
}

pub fn remove_env(name: &str) -> Result<(), AppError> {
    let env_dir = Environment::env_dir_for_name(name);

    if !env_dir.is_dir() {
        return Err(EnvironmentError::DoesNotExist { name: name.to_string() }.into());
    }

    fs::remove_dir_all(&env_dir).report_delete("environment directory", &env_dir)?;

    // If we deleted our active environment then explicitly revert back to the null environment (so isabelle is happy)
    if !Environment::has_active() {
        environment::manager::set_env_none()?;
    }

    CliLine::new()
        .prefix("Removed")
        .line(format!("environment {}", style(&name).cyan().bright()))
        .as_success()
        .print();

    Ok(())
}

pub fn freeze_env() -> Result<(), AppError> {
    let active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;
    active_env.freeze()?;

    CliLine::new()
        .prefix("Frozen")
        .line(format!("to {}", style(LOCKFILE_NAME).cyan().bright()))
        .as_success()
        .print();

    Ok(())
}

pub async fn sync_env() -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.sync()?;
    // Don't resolve as we want to keep the lockfile identical
    finalise_env(&mut active_env, FinalizeStrategy::ApplyOnly).await?;

    CliLine::new()
        .prefix("Synced")
        .line(format!("from {}", style(LOCKFILE_NAME).cyan().bright()))
        .as_success()
        .print();

    // Warn if this new environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub async fn migrate_isabelle(version: VersionReq, unpin_existing: bool) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.migrate_isabelle(version, unpin_existing);
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    let line = match get_isabelle_version(&active_env) {
        Some(v) => format!("to {} {}", style(get_isabelle_name(v.get_version())).cyan().bright(), v),
        None => format!("to {}", style("latest").cyan().bright()),
    };

    CliLine::new().prefix("Migrated").line(line).as_success().print();

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub async fn restore() -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    // Don't resolve as we want to keep environment identical
    finalise_env(&mut active_env, FinalizeStrategy::ApplyOnly).await?;

    CliLine::new().prefix("Restored").line("all packages").as_success().print();

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}
