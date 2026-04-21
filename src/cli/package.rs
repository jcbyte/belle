use std::cmp::max;

use console::style;

use crate::{
    cli::{
        core::{CliLine, DisplayVersion, pluralise},
        environment::{FinalizeStrategy, finalise_env, get_isabelle_version, warn_no_isabelle},
        error::CliError,
    },
    config::BelleConfig,
    environment::{Environment, PackageType, VersionReq},
    error::AppError,
    registry::PackageIdentifier,
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};

pub async fn add_package(name: String, version: VersionReq) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    // Assume no-op for specific-specific or any-any, as updates should be performed differently
    if active_env
        .packages
        .get(&name)
        .map(|existing_version| *existing_version == version)
        .unwrap_or(false)
    {
        CliLine::new()
            .line(format!(
                "package {} already exists in environment",
                match version {
                    VersionReq::Any => style(name).cyan().to_string(),
                    VersionReq::Given(v) => PackageIdentifier::new(name, v).styled(),
                }
            ))
            .with_skipped()
            .print();
        CliLine::new().line("use `belle update` to update packages").with_note().print();
        return Ok(());
    };

    active_env.add_package(name.clone(), version)?;
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    let package_listing = active_env
        .get_package_listing(&name)
        .expect("Package just added, now cannot be found");
    let package_version = match package_listing.kind {
        PackageType::ImplicitDirect => DisplayVersion::Implicit(&package_listing.version),
        PackageType::ExplicitDirect => DisplayVersion::Explicit(&package_listing.version),
        _ => unreachable!(),
    };

    CliLine::new()
        .prefix("Added")
        .line(format!("package {} {}", style(name).cyan(), package_version))
        .with_success()
        .print();

    warn_no_isabelle()?;

    Ok(())
}

pub async fn remove_package(name: &str) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    if !active_env.packages.contains_key(name) {
        CliLine::new()
            .line(format!(
                "package '{}' is not in this environment; nothing to remove",
                style(name).cyan()
            ))
            .with_skipped()
            .print();
        CliLine::new()
            .line("use `belle list` to see packages in active environment")
            .with_note()
            .print();
        return Ok(());
    };

    active_env.remove_package(name)?;
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    CliLine::new()
        .prefix("Removed")
        .line(format!("package {}", style(name).cyan()))
        .with_success()
        .print();

    warn_no_isabelle()?;

    Ok(())
}

pub fn list_packages(all: bool) -> Result<(), AppError> {
    let active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

    // Partition packages into these
    let mut dependencies = Vec::new();
    let mut transitive_dependencies = Vec::new();
    let mut isabelle_dependencies = Vec::new();

    let mut largest_dependency_name: usize = 0;
    let mut largest_dependency_name_all: usize = 0;

    for dependency in active_env.iter_packages() {
        match dependency.kind {
            PackageType::ImplicitDirect | PackageType::ExplicitDirect => {
                largest_dependency_name = max(largest_dependency_name, dependency.name.len());
                dependencies.push(dependency)
            }
            // Ignore the main Isabelle package, as we get this separately
            PackageType::Transitive if dependency.name != ISABELLE_PACKAGE => {
                largest_dependency_name_all = max(largest_dependency_name_all, dependency.name.len());

                if isabelle_packages.contains(&dependency.name) {
                    isabelle_dependencies.push(dependency);
                } else {
                    transitive_dependencies.push(dependency);
                }
            }
            _ => {}
        }
    }

    // Calculate the largest dependency name for version column padding
    largest_dependency_name = max(
        if all {
            max(largest_dependency_name, largest_dependency_name_all)
        } else {
            largest_dependency_name
        },
        "Isabelle".len(),
    );

    let formatted_isabelle_str = match get_isabelle_version(&active_env) {
        Some(v) => format!(
            "{:padding$} {}",
            style(get_isabelle_name(v.get_version())).cyan(),
            v,
            padding = largest_dependency_name
        ),
        None => "unspecified version".to_string(),
    };
    CliLine::new()
        .prefix("Isabelle")
        .line(formatted_isabelle_str)
        .with_focus()
        .print();

    for package in &dependencies {
        let styled_version = match package.kind {
            PackageType::ExplicitDirect => DisplayVersion::Explicit(&package.version),
            _ => DisplayVersion::Implicit(&package.version),
        };

        CliLine::new()
            .line(format!(
                "{:padding$} {}",
                package.name,
                styled_version,
                padding = largest_dependency_name
            ))
            .print();
    }

    if all {
        for package in &transitive_dependencies {
            CliLine::new()
                .line(format!(
                    "{:padding$} {}",
                    style(&package.name).dim(),
                    DisplayVersion::Implicit(&package.version),
                    padding = largest_dependency_name,
                ))
                .print();
        }

        for package in &isabelle_dependencies {
            CliLine::new()
                .line(format!(
                    "{:padding$} {}",
                    style(&package.name).dim().italic(),
                    DisplayVersion::Implicit(&package.version),
                    padding = largest_dependency_name,
                ))
                .print();
        }
    }

    let line = if !all {
        format!(
            "{} {} in {}",
            style(dependencies.len()).bold(),
            pluralise(dependencies.len(), "package", "packages"),
            style(active_env.name).cyan()
        )
    } else {
        let total_packages = dependencies.len() + transitive_dependencies.len() + isabelle_dependencies.len();
        format!(
            "{} {} ({} direct, {} transitive, {} core) in {}",
            style(total_packages).bold(),
            pluralise(total_packages, "package", "packages"),
            style(dependencies.len()).bold(),
            style(transitive_dependencies.len()).bold(),
            style(isabelle_dependencies.len()).bold(),
            style(active_env.name).cyan()
        )
    };
    CliLine::new().prefix("Listed").line(line).with_success().print();

    Ok(())
}
