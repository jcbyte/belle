use console::style;
use pubgrub::SemanticVersion;

use crate::{
    cli::environment::finalise_env,
    config::BelleConfig,
    environment::{Environment, PackageType},
    resolver::ISABELLE_PACKAGE,
    util::get_isabelle_name,
};

pub async fn add_package(name: String, version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;

    active_env.add_package(name.clone(), version.into())?;
    finalise_env(&mut active_env).await?;

    println!("Added package {}.", style(name).cyan());
    return Ok(());
}

pub async fn remove_package(name: &String) -> anyhow::Result<()> {
    let mut active_env = Environment::active()?.ok_or(anyhow::anyhow!("No environment is selected"))?;

    active_env.remove_package(name)?;
    finalise_env(&mut active_env).await?;

    println!("Removed package {}.", style(name).cyan());
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
