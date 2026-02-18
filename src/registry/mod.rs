use std::{collections::HashMap, fmt, fs};

use anyhow::Context;
use console::style;
use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

use crate::{config::BelleConfig, registry::registry::get_package_versions};

pub mod package;
pub mod registry;

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageAuthor {
    pub name: String,
    pub email: Option<String>,
    pub homepages: Option<Vec<String>>,
    pub orcid: Option<String>,
}

/// All package metadata stored on disk
#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: SemanticVersion,
    pub title: String,
    pub date: toml::value::Date,
    pub r#abstract: String,
    pub licence: String,
    pub topics: Vec<String>,
    pub note: Option<String>,

    pub authors: Vec<PackageAuthor>,
    pub contributors: Vec<PackageAuthor>,

    pub dependencies: HashMap<String, SemanticVersion>,

    pub extra: toml::Table,
}

/// Subset of `Package` stored in disk for quick dependency resolution
#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    name: String,
    version: SemanticVersion,
    dependencies: HashMap<String, SemanticVersion>,
}

/// Key for packages
#[derive(Clone)]
pub struct PackageIdentifier {
    pub name: String,
    pub version: SemanticVersion,
}

impl From<&Package> for Manifest {
    fn from(package: &Package) -> Self {
        return Self {
            name: package.name.clone(),
            version: package.version.clone(),
            dependencies: package.dependencies.clone(),
        };
    }
}

impl From<&Package> for PackageIdentifier {
    fn from(package: &Package) -> Self {
        return Self {
            name: package.name.clone(),
            version: package.version.clone(),
        };
    }
}

impl From<&Manifest> for PackageIdentifier {
    fn from(manifest: &Manifest) -> Self {
        return Self {
            name: manifest.name.clone(),
            version: manifest.version.clone(),
        };
    }
}

impl fmt::Display for PackageIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

pub fn clean_theories(version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let config = BelleConfig::global();
    let thy_dir = config.get_theory_dir();

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

pub fn clean_metadata(version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let config = BelleConfig::global();
    let meta_dir = config.get_manifest_dir();
    let manifest_dir = config.get_manifest_dir();

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

pub fn list_versions(name: String) -> anyhow::Result<()> {
    let versions = get_package_versions(&name)?;

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

pub fn print_meta(name: String, version: Option<SemanticVersion>) -> anyhow::Result<()> {
    let package = match version {
        Some(v) => PackageIdentifier { name, version: v },
        None => {
            let versions = get_package_versions(&name)?;
            versions
                .iter()
                .max_by_key(|pi| pi.version)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No versions of '{}' can be found", name))?
        }
    };

