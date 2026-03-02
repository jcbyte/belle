use std::fs;

use anyhow::Context;
use console::style;
use pubgrub::SemanticVersion;

use crate::{
    config::BelleConfig,
    registry::{self, AliasPackage, Package, PackageIdentifier, RegisteredPackage},
};

/// Remove all theories from disk
pub fn clean_theories() -> anyhow::Result<()> {
    let thy_dir = BelleConfig::read_config(|c| c.get_theory_dir());
    if !thy_dir.is_dir() {
        println!("No theories found in cache");
        return Ok(());
    }

    fs::remove_dir_all(thy_dir).context("Failed to remove theory cache")?;
    println!("Cleaned {} theories from cache.", style("all").bold());

    return Ok(());
}

/// Remove all metadata from disk
pub fn clean_metadata() -> anyhow::Result<()> {
    let manifest_dir = BelleConfig::read_config(|c| c.get_manifest_dir());

    if !manifest_dir.is_dir() {
        println!("No metadata found in cache");
        return Ok(());
    }

    fs::remove_dir_all(manifest_dir).context("Failed to remove manifest cache")?;
    println!("Cleaned metadata for {} theories.", style("all").bold());

    return Ok(());
}

/// List versions of a package in our local metadata
pub fn list_versions(name: String) -> anyhow::Result<()> {
    let versions = registry::get_package_versions(&name)?;

    if versions.is_empty() {
        println!("No versions of package {} installed", name)
    } else {
        let mut installed_count = 0;

        println!("Version listing for {}:", style(&name).cyan());
        for version in &versions {
            print!(" - {:<9}", style(version.version.to_string()).green(),);
            if version.exists_locally() {
                installed_count += 1;
                print!("{}", style(" [installed]").dim());
            }
            println!();
        }
        println!(
            "Found {} versions for {} {}.",
            style(versions.len()).bold(),
            style(&name).cyan(),
            style(format!("({} installed)", installed_count)).dim(),
        );
    }

    return Ok(());
}

/// Prints nicely formatted metadata for a package to the console
fn print_meta(meta: &Package, alias: Option<&AliasPackage>) {
    let header = format!(
        "{} {} {}{}{}",
        style(&meta.name).cyan().bold(),
        style(&meta.title).bold(),
        style("[").dim(),
        style(meta.version).green(),
        style("]").dim()
    );
    println!("{}", header);

    if let Some(alias) = alias {
        println!(
            "{} {}{}{} {}",
            style(&alias.name).cyan().dim(),
            style("[").dim(),
            style(alias.version).green().dim(),
            style("]").dim(),
            style("[Alias]").dim(),
        )
    }

    println!("{}", style("─".repeat(console::measure_text_width(&header))).dim());

    println!("{}", style(&meta.r#abstract).italic());

    if let Some(note) = &meta.note {
        println!("{} {}", style("Note:").yellow().bold(), note);
    }

    println!();

    println!("{:<10} {}", style("Date:").dim(), meta.date);
    if !meta.topics.is_empty() {
        println!("{:<10} {}", style("Topics:").dim(), meta.topics.join(", "));
    }
    println!("{:<10} {}", style("License:").dim(), meta.licence);

    println!();

    if !meta.authors.is_empty() {
        println!("{}", style("Authors:").bold());
        for author in &meta.authors {
            print!(" - {}", author.name);
            if let Some(email) = &author.email {
                print!(" {}", style(format!("<{}>", email)).dim());
            }
            if let Some(orcid) = &author.orcid {
                print!(" {}", style(format!("(ORCID:{})", orcid)).dim());
            }
            println!()
        }
    }

    println!();

    if !meta.dependencies.is_empty() {
        println!("{}", style("Dependencies:").bold());
        for (name, ver) in &meta.dependencies {
            println!(" - {} {}", style(&name).magenta(), style(format!("[{}]", ver)).dim());
        }
    }

    println!();
}

/// Display metadata for a specific package on the console, if a version is not given then the latest will be shown
pub fn print_package_meta(name: String, version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let package = match version {
        Some(v) => PackageIdentifier { name, version: v },
        None => {
            let versions = registry::get_package_versions(&name)?;
            versions
                .iter()
                .max_by_key(|pi| pi.version)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No versions of '{}' can be found", name))?
        }
    };

    let package_meta = package.get_package_manifest()?;

    match package_meta {
        Some(meta) => match meta {
            RegisteredPackage::Package(meta) => print_meta(&meta, None),
            RegisteredPackage::Alias(alias) => {
                let resolved_package = alias
                    .alias
                    .get_resolved_package_manifest()?
                    .expect(format!("Resolved alias '{}' cannot be found", alias.name).as_str());
                print_meta(&resolved_package, Some(&alias));
            }
        },
        None => anyhow::bail!("Package '{}' does not exist", package),
    };

    return Ok(());
}

// todo add isabelle versions to listing and colour dependencies correctly
