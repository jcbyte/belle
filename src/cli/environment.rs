use std::fs;

use clap::error::Result;
use console::style;
use hinted::{Hinted, HintedResultExt};
use indicatif::ProgressBar;

use crate::{
    cli::{CliLine, DisplayVersion, ProgressBarTheme, environment, error::CliError, pluralise},
    config::BelleConfig,
    environment::{Environment, LOCKFILE_NAME, VersionReq, manager},
    error::{AppError, CustomErrorContext, IoErrorContext},
    registry::{PackageIdentifier, error::RegistryNotExistContext},
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};
use std::fmt::Write;

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
        pb.set_belle_prefix("Resolving");
        pb.set_message("dependencies".to_string());

        // Resolve lockfile dependencies
        env.resolve_lock()?;

        pb.finish_and_clear();
    }

    // Fetch all packages we currently do not have
    let mut missing_packages = Vec::new();
    for (name, version) in env.iter_user_packages() {
        let raw_package = PackageIdentifier::new(name, *version);

        // Resolve aliases to not try to re-download an alias
        let resolved = raw_package
            .get_resolved_package_manifest()?
            .report_package_nonexistent(raw_package)?;

        if !PackageIdentifier::from(&resolved).exists_locally() {
            missing_packages.push(resolved);
        }
    }

    if !missing_packages.is_empty() {
        let pb = ProgressBar::new(missing_packages.len() as u64).with_belle_bar_style();
        pb.set_belle_prefix("Fetching");

        for package in &missing_packages {
            pb.set_message(PackageIdentifier::from(package).styled());

            package.get_package().await?;

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
            .with_success()
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
            .with_warning()
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

pub fn switch_env(name: Option<String>) -> Result<(), Hinted<AppError>> {
    let name = match name {
        Some(n) => n,
        None => {
            let frozen_env = Environment::frozen()?
                .report_custom("no environment name was provided, and no lockfile was found to infer from")
                .hint("specify a name with `belle switch <name>`")?;
            frozen_env.name
        }
    };

    if let Some(active_env) = Environment::active()?
        && active_env.name == name
    {
        CliLine::new()
            .line(format!("environment {} is already active", style(name).cyan().bright()))
            .with_skipped()
            .print();
        return Ok(());
    };

    manager::switch_env(&name)?;

    CliLine::new()
        .prefix("Switched")
        .line(format!("to environment {}", style(name).cyan()))
        .with_success()
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
        .with_success()
        .print();

    // Switch into the newly created environment, qol
    manager::switch_env(&name)?;

    CliLine::new()
        .prefix("Switched")
        .line(format!("to environment {}", style(&name).cyan().bright()))
        .with_success()
        .print();

    Ok(())
}

pub fn list_envs() -> Result<(), AppError> {
    let active_env = Environment::active()?.map(|active_env| active_env.name);

    let mut env_count = 0;
    for env in manager::iter_envs() {
        if active_env.as_ref() == Some(&env) {
            // If this is the active environment then highlight it
            CliLine::new().prefix("Active").line(env).with_focus()
        } else {
            CliLine::new().line(env)
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
        .with_success()
        .print();

    Ok(())
}

pub fn remove_env(name: &str) -> Result<(), AppError> {
    let env_dir = Environment::env_dir_for_name(name);

    if !env_dir.is_dir() {
        CliLine::new()
            .line(format!(
                "environment '{}' does not exist; nothing to remove",
                style(name).cyan().bright()
            ))
            .with_skipped()
            .print();
        return Ok(());
    }

    fs::remove_dir_all(&env_dir).report_delete("environment directory", &env_dir)?;

    // If we deleted our active environment then explicitly revert back to the null environment (so that isabelle is happy)
    if !Environment::has_active() {
        environment::manager::set_env_none()?;
    }

    CliLine::new()
        .prefix("Removed")
        .line(format!("environment {}", style(&name).cyan().bright()))
        .with_success()
        .print();

    Ok(())
}

pub fn freeze_env() -> Result<(), AppError> {
    let active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;
    active_env.freeze()?;

    CliLine::new()
        .prefix("Frozen")
        .line(format!("to {}", style(LOCKFILE_NAME).cyan().bright()))
        .with_success()
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
        .with_success()
        .print();

    // Warn if this new environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub async fn migrate_isabelle(version: VersionReq, unpin_existing: bool) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    // Do not assume this is a no-op if a version is not given
    // As there could be a new latest to migrate too
    if let VersionReq::Given(target_version) = &version
        && version == active_env.isabelle
    {
        CliLine::new()
            .line(format!(
                "environment already matches Isabelle {} {}",
                get_isabelle_name(target_version),
                DisplayVersion::Explicit(target_version)
            ))
            .with_skipped()
            .print();
        return Ok(());
    };

    if unpin_existing {
        active_env.unpin_package_versions();
    }
    active_env.migrate_isabelle(version);
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    let line = match get_isabelle_version(&active_env) {
        Some(v) => format!("to {} {}", get_isabelle_name(v.get_version()), v),
        None => "to latest".to_string(),
    };

    CliLine::new().prefix("Migrated").line(line).with_success().print();

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub async fn update(unpin_existing: bool) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    // Record current statistics, to compare later
    let previous_isabelle = active_env.get_isabelle_version();
    let previous_lock = active_env.lock.clone();

    if unpin_existing {
        active_env.unpin_package_versions();
    }
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    // Calculate changed statistics
    let mut updated = 0;
    let mut added = 0;
    let mut up_to_date = 0;

    for (name, version) in &active_env.lock {
        match previous_lock.get(name) {
            Some(prev_version) => {
                if prev_version == version {
                    up_to_date += 1;
                } else {
                    updated += 1;
                }
            }
            // Not seen in previous environment
            None => added += 1,
        }
    }

    let removed = previous_lock.keys().filter(|name| !active_env.lock.contains_key(*name)).count();

    let total_changed = updated + added + removed;

    let mut line = format!(
        "{} packages changed, {} up to date",
        style(total_changed).bold(),
        style(up_to_date).bold()
    );

    let active_isabelle_version = active_env.get_isabelle_version();
    // If isabelle has been updated also display
    if active_isabelle_version != previous_isabelle {
        let version_str = match active_isabelle_version {
            VersionReq::Given(v) => format!("{} {}", get_isabelle_name(&v), DisplayVersion::Implicit(&v)),
            VersionReq::Any => "latest Isabelle".to_string(),
        };
        write!(line, "; migrated Isabelle to {}", version_str).expect("Writing to a String failed");
    }

    CliLine::new().prefix("Updated").line(line).with_success().print();

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub async fn restore() -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    // Don't resolve as we want to keep environment identical
    finalise_env(&mut active_env, FinalizeStrategy::ApplyOnly).await?;

    CliLine::new().prefix("Restored").line("all packages").with_success().print();

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}
