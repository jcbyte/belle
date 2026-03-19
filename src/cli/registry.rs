use std::{collections::HashSet, fs};

use console::style;
use pubgrub::SemanticVersion;

use crate::{
    config::BelleConfig,
    environment::{Environment, manager::iter_envs},
    error::{AppError, IoErrorContext},
    registry::{
        self, AliasPackage, Package, PackageIdentifier, PackageSource, RegisteredPackage,
        error::RegistryNotExistContext, iter_installed_packages, iter_packages,
    },
    util::get_isabelle_name,
};

/// Remove all theories from disk
pub fn clean_theories() -> Result<(), AppError> {
    let thy_dir = BelleConfig::read_config(|c| c.get_theory_dir());
    if !thy_dir.is_dir() {
        println!("No theories found in cache");
        return Ok(());
    }

    fs::remove_dir_all(&thy_dir).report_delete("packages source", &thy_dir)?;
    println!("Cleaned {} theories from cache.", style("all").bold());

    Ok(())
}

/// Remove all metadata from disk
pub fn clean_metadata() -> Result<(), AppError> {
    let manifest_dir = BelleConfig::read_config(|c| c.get_manifest_dir());

    if !manifest_dir.is_dir() {
        println!("No metadata found in cache");
        return Ok(());
    }

    fs::remove_dir_all(&manifest_dir).report_delete("packages manifests", &manifest_dir)?;
    println!("Cleaned metadata for {} theories.", style("all").bold());

    Ok(())
}

/// List versions of a package in our local metadata
pub fn list_versions(name: &str) -> Result<(), AppError> {
    let versions = registry::get_package_versions(name);

    if versions.is_empty() {
        println!("No versions of package {} installed", name)
    } else {
        let mut installed_count = 0;

        println!("Version listing for {}:", style(name).cyan());
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
            style(name).cyan(),
            style(format!("({} installed)", installed_count)).dim(),
        );
    }

    Ok(())
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

    println!("{:<10} {}", style("Date:").bold(), meta.date);
    if !meta.topics.is_empty() {
        println!("{:<10} {}", style("Topics:").bold(), meta.topics.join(", "));
    }
    println!("{:<10} {}", style("License:").bold(), meta.licence);
    let source_str = match &meta.source {
        PackageSource::Afp(repo) => format!(
            "{} {}{}{}",
            repo.name,
            style("[").dim(),
            style(repo.get_version()).green(),
            style("]").dim()
        ),
        PackageSource::Remote { url } => format!("Remote: {}", url),
        PackageSource::Local { path } => format!("Local: {}", path.to_string_lossy()),
        _ => String::new(),
    };
    println!("{:<10} {}", style("Source:").bold(), source_str);

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

    println!("{}", style("Isabelle Versions:").bold());
    for isabelle_version in &meta.isabelles {
        println!(
            "- {:<6} {}{}{}",
            style(get_isabelle_name(isabelle_version)),
            style("[").dim(),
            style(isabelle_version).green(),
            style("]").dim()
        )
    }

    println!();

    if !meta.dependencies.is_empty() {
        println!("{}", style("Dependencies:").bold());

        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

        let mut dependencies = Vec::new();
        let mut isabelle_dependencies = Vec::new();

        for (name, ver) in &meta.dependencies {
            if isabelle_packages.contains(name) {
                isabelle_dependencies.push(name.clone());
            } else {
                dependencies.push((name.clone(), *ver));
            }
        }

        for (name, version) in dependencies {
            println!(
                "- {} {}{}{}",
                style(name),
                style("[").dim(),
                style(version).dim(),
                style("]").dim()
            )
        }

        for name in isabelle_dependencies {
            println!("- {}", style(name).dim().italic(),)
        }
    }

    println!();

    if !meta.provides.is_empty() {
        println!("{}", style("Provides Packages:").bold());

        for alias in &meta.provides {
            println!(
                "- {} {}{}{}",
                style(alias),
                style("[").dim(),
                style(meta.version).dim(),
                style("]").dim()
            )
        }
    }

    println!();

    if !meta.extra.is_empty() {
        println!("{}", style("Extra Information:").bold());

        for extra in &meta.extra {
            println!("{:<10} {}", style(format!("{}:", extra.0)).dim(), extra.1);
        }
    }
}

/// Display metadata for a specific package on the console, if a version is not given then the latest will be shown
pub fn print_package_meta(name: String, version: Option<SemanticVersion>) -> Result<(), AppError> {
    let package = match version {
        Some(v) => PackageIdentifier::new(name, v),
        None => {
            let versions = registry::get_package_versions(&name);
            versions
                .into_iter()
                .max_by_key(|package_id| package_id.version)
                .report_no_package_versions(name)?
        }
    };

    let package_meta = package.get_package_manifest()?.report_package_nonexistent(package)?;
    match package_meta {
        RegisteredPackage::Package(meta) => print_meta(&meta, None),
        RegisteredPackage::Alias(alias) => {
            let resolved_package = alias
                .alias
                .get_resolved_package_manifest()?
                .report_package_nonexistent(alias.alias.clone())?;
            print_meta(&resolved_package, Some(&alias));
        }
    };

    Ok(())
}

fn highlight_match(text: &str, query: &str) -> String {
    if let Some(start) = text.to_lowercase().find(&query.to_lowercase()) {
        let end = start + query.len();

        let prefix = &text[..start];
        let matched = &text[start..end];
        let suffix = &text[end..];

        // Wrap the matched part in a different color/style
        format!("{}{}{}", prefix, style(matched).cyan(), suffix)
    } else {
        text.to_string()
    }
}

pub fn search_registry(search: String) {
    let mut results = Vec::new();

    for package in iter_packages() {
        if package.to_lowercase().contains(&search.to_lowercase()) {
            results.push(package);
        }
    }

    if results.is_empty() {
        println!("Found {} Results for '{}'.", style("0").bold(), style(search).cyan());

        return;
    }

    // Print list of results
    println!("Search results for '{}':", style(&search).cyan());

    for package in &results {
        println!("{} {}", style("-").dim(), highlight_match(package, &search));
    }
    println!("Found {} Results.", style(results.len()).bold());
}

pub fn purge_packages() -> Result<(), AppError> {
    let mut used_packages: HashSet<PackageIdentifier> = HashSet::new();

    for env_name in iter_envs() {
        let env = Environment::get(&env_name)?.expect("Environment listed, but could not be gotten");
        for (name, version) in env.lock {
            used_packages.insert(PackageIdentifier::new(name, version));
        }
    }

    let mut removed = 0;
    for installed_package in iter_installed_packages() {
        if !used_packages.contains(&installed_package) {
            installed_package.remove()?;
            removed += 1;
        }
    }

    println!("Removed {} PAckages.", style(removed).bold());

    Ok(())
}
