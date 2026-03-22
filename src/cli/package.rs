use std::cmp::max;

use console::style;

use crate::{
    cli::{
        core::{DisplayVersion, pluralise, print_blank_ln, print_ln, print_success_ln},
        environment::{FinalizeStrategy, finalise_env, warn_no_isabelle},
        error::CliError,
    },
    config::BelleConfig,
    environment::{Environment, PackageType, VersionReq},
    error::AppError,
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};

pub async fn add_package(name: String, version: VersionReq) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.add_package(name.clone(), version)?;
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    // todo should this contain package version too
    print_success_ln("Added", format_args!("package {}", style(name).cyan().bright()));

    warn_no_isabelle()?;

    Ok(())
}

pub async fn remove_package(name: &str) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.remove_package(name)?;
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    print_success_ln("Removed", format_args!("package {}", style(name).cyan().bright()));

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
            // Ignore the main isabelle package, as we get this separately
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
    largest_dependency_name = if all {
        max(largest_dependency_name, largest_dependency_name_all)
    } else {
        largest_dependency_name
    };

    let isabelle_version = match &active_env.isabelle {
        VersionReq::Given(v) => Some(DisplayVersion::Explicit(v)),
        VersionReq::Any => active_env.lock.get(ISABELLE_PACKAGE).map(DisplayVersion::Implicit),
    };

    let formatted_isabelle_str = match isabelle_version {
        Some(v) => format!("{} {}", style(get_isabelle_name(v.get_version())).cyan().bright(), v),
        None => "unspecified version".to_string(),
    };
    print_ln("Isabelle", console::Color::Cyan, formatted_isabelle_str);

    for package in &dependencies {
        let styled_version = match package.kind {
            PackageType::ExplicitDirect => DisplayVersion::Explicit(&package.version),
            _ => DisplayVersion::Implicit(&package.version),
        };

        print_blank_ln(format_args!(
            "{:padding$} {}",
            package.name,
            styled_version,
            padding = largest_dependency_name
        ));
    }

    if all {
        for package in &transitive_dependencies {
            print_blank_ln(format_args!(
                "{:padding$} {}",
                style(&package.name).dim(),
                DisplayVersion::Implicit(&package.version),
                padding = largest_dependency_name,
            ));
        }

        for package in &isabelle_dependencies {
            print_blank_ln(format_args!(
                "{:padding$} {}",
                style(&package.name).dim().italic(),
                DisplayVersion::Implicit(&package.version),
                padding = largest_dependency_name,
            ));
        }
    }

    let line = if !all {
        format!(
            "{} {} in {}",
            style(dependencies.len()).bold(),
            pluralise(dependencies.len(), "package", "packages"),
            style(active_env.name).cyan().bright()
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
            style(active_env.name).cyan().bright()
        )
    };
    print_success_ln("Listed", line);

    Ok(())
}
