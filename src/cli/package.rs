use console::style;

use crate::{
    cli::{
        environment::{FinalizeStrategy, finalise_env},
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

    println!("Added package {}.", style(name).cyan());
    Ok(())
}

pub async fn remove_package(name: &str) -> Result<(), AppError> {
    let mut active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    active_env.remove_package(name)?;
    finalise_env(&mut active_env, FinalizeStrategy::ResolveAndApply).await?;

    println!("Removed package {}.", style(name).cyan());
    Ok(())
}

pub fn list_packages(all: bool) -> Result<(), AppError> {
    let active_env = Environment::active()?.ok_or(CliError::NoActiveEnvironment)?;

    let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

    // Partition packages into these
    let mut isabelle_listing = None;
    let mut dependencies = Vec::new();
    let mut transitive_dependencies = Vec::new();
    let mut isabelle_dependencies = Vec::new();

    for dependency in active_env.iter_packages() {
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

    println!("Environment: {}", style(active_env.name).cyan());

    let formatted_isabelle_str = match isabelle_listing {
        Some(isabelle) => format!(
            "{} {}{}{}",
            style(get_isabelle_name(&isabelle.version)).cyan().bold(),
            style("[").dim(),
            style(&isabelle.version.to_string()).green(),
            style("]").dim(),
        ),
        None => format!("{}", style("Unspecified").dim()),
    };
    println!("{} {}", style("* Isabelle:").bold(), formatted_isabelle_str,);

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

    Ok(())
}
