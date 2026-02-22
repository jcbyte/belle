use std::fs;

use anyhow::Context;
use console::style;
use pubgrub::SemanticVersion;

use crate::{
    config::BelleConfig,
    registry::{self, Package, PackageIdentifier},
};

/// Remove theories from disk, if version is not given then remove all
pub fn clean_theories(version: Option<SemanticVersion>) -> anyhow::Result<()> {
    // todo if folder doesn't exist it will fail

    let thy_dir = BelleConfig::read_config(|c| c.get_theory_dir());

    match version {
        // If no version is given, this means all
        None => {
            fs::remove_dir_all(thy_dir).context("Failed to remove theory cache")?;
            println!("Cleaned {} theories from cache.", style("all").bold());
        }
        Some(version) => {
            let mut count = 0;

            // Find all theory folders for the requested version and remove them
            for theory in registry::iter_package_files(&thy_dir, &version) {
                fs::remove_dir_all(&theory).with_context(|| {
                    let name = theory
                        .parent()
                        .and_then(|parent| parent.file_name())
                        .map(|theory_name| theory_name.to_string_lossy().into_owned())
                        .unwrap_or(String::from("unknown_package"));

                    format!("Failed to remove package '{}", PackageIdentifier { name, version })
                })?;

                count += 1;
            }

            println!("Cleaned {} theories from cache.", style(count).bold());
        }
    };

    return Ok(());
}

/// Remove metadata from disk, if version is not given then remove all
pub fn clean_metadata(version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let meta_dir = BelleConfig::read_config(|c| c.get_meta_dir());
    let manifest_dir = BelleConfig::read_config(|c| c.get_manifest_dir());

    match version {
        // If no version is given, this means all
        None => {
            fs::remove_dir_all(meta_dir).context("Failed to remove metadata cache")?;
            fs::remove_dir_all(manifest_dir).context("Failed to remove manifest cache")?;

            println!("Cleaned metadata for {} theories.", style("all").bold());
        }
        Some(version) => {
            let mut count = 0;

            // Find all metadata files for the requested version and remove them
            for meta_file in registry::iter_package_files(&meta_dir, &version) {
                fs::remove_file(&meta_file).with_context(|| {
                    let name = meta_file
                        .parent()
                        .and_then(|parent| parent.file_name())
                        .map(|theory_name| theory_name.to_string_lossy().into_owned())
                        .unwrap_or(String::from("unknown_package"));

                    format!("Failed to remove metadata for '{}", PackageIdentifier { name, version })
                })?;

                count += 1;
            }

            // Find all manifest files for the requested version and remove them
            // The count should be equal for manifests
            for manifest_file in registry::iter_package_files(&manifest_dir, &version) {
                fs::remove_file(&manifest_file).with_context(|| {
                    let name = manifest_file
                        .parent()
                        .and_then(|parent| parent.file_name())
                        .map(|theory_name| theory_name.to_string_lossy().into_owned())
                        .unwrap_or(String::from("unknown_package"));

                    format!("Failed to remove manifest for '{}", PackageIdentifier { name, version })
                })?;
            }

            println!("Cleaned metadata for {} theories.", style(count).bold());
        }
    };

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
fn print_meta(meta: &Package) {
    println!();

    let header = format!(
        "{} {} {}{}{}",
        style(&meta.name).cyan().bold(),
        style(&meta.title).bold(),
        style("[").dim(),
        style(meta.version).green(),
        style("]").dim()
    );
    println!("{}", header);
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

    let package_meta = package.get_package_meta()?;
    match package_meta {
        Some(meta) => {
            print_meta(&meta);
        }
        None => anyhow::bail!("Package '{}' does not exist", package),
    };

    return Ok(());
}
