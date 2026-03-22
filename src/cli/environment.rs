use std::{fs, time::Duration};

use console::style;
use indicatif::ProgressBar;

use crate::{
    cli::{
        core::{DisplayVersion, ProgressBarTheme, print_blank_ln, print_ln, print_success_ln, print_warning_ln},
        environment,
        error::CliError,
    },
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
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Resolving dependency tree".to_string());

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
            pb.set_message(format!("Fetching {}", style(&package).cyan()));

            let package_meta = package
                .get_resolved_package_manifest()?
                .report_package_nonexistent(package.clone())?;

            package_meta.get_package().await?;

            pb.inc(1);
        }

        pb.finish_and_clear();

        print_success_ln(
            "Fetched",
            format_args!("{} new packages", style(missing_packages.len()).bold()),
        );
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
        print_warning_ln(format_args!(
            "This environment uses Isabelle {} [{}], but that version is not linked",
            get_isabelle_name(&v),
            &v
        ));
    }

    Ok(())
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

    print_success_ln("Switched", format_args!("to environment {}", style(name).cyan()));

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

    print_success_ln("Created", format_args!("environment {}", style(&name).cyan().bright()));

    // Switch into the newly created environment, qol
    manager::switch_env(&name)?;

    print_success_ln(
        "Switched",
        format_args!("to environment {}", style(&name).cyan().bright()),
    );

    Ok(())
}

pub fn list_envs() -> Result<(), AppError> {
    let active_env = Environment::active()?.map(|active_env| active_env.name);

    let mut env_count = 0;
    for env in manager::iter_envs() {
        if active_env.as_ref() == Some(&env) {
            // If this is the active environment then highlight it
            print_ln("Active", console::Color::Cyan, env);
        } else {
            print_blank_ln(&env);
        }

        env_count += 1;
    }

    print_success_ln("Total", format_args!("{} environments", style(env_count).bold()));

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

    print_success_ln("Removed", format_args!("environment {}", style(&name).cyan().bright()));

    Ok(())
}

pub fn freeze_env() -> Result<(), AppError> {
    let active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;
    active_env.freeze()?;

    print_success_ln("Frozen", format_args!("to {}", style(LOCKFILE_NAME).cyan().bright()));

    Ok(())
}

pub async fn sync_env() -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.sync()?;
    // Don't resolve as we want to keep the lockfile identical
    finalise_env(&mut active_env, FinalizeStrategy::ApplyOnly).await?;

    print_success_ln("Synced", format_args!("from {}", style(LOCKFILE_NAME).cyan().bright()));

    // Warn if this new environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub async fn migrate_isabelle(version: VersionReq, unpin_existing: bool) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.migrate_isabelle(version, unpin_existing);
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    let new_isabelle_version = match &active_env.isabelle {
        VersionReq::Given(v) => Some(DisplayVersion::Explicit(v)),
        VersionReq::Any => active_env.lock.get(ISABELLE_PACKAGE).map(DisplayVersion::Implicit),
    };

    let line = match new_isabelle_version {
        Some(v) => format!("to {} {}", style(get_isabelle_name(v.get_version())).cyan().bright(), v),
        None => format!("to {}", style("latest").cyan().bright()),
    };

    print_success_ln("Migrated", line);

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}

pub async fn restore() -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    // Don't resolve as we want to keep environment identical
    finalise_env(&mut active_env, FinalizeStrategy::ApplyOnly).await?;

    print_success_ln("Restored", "all packages");

    // Warn if this environment doesn't have a linked isabelle version
    warn_no_isabelle()?;

    Ok(())
}
