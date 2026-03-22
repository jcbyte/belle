use std::{borrow::Cow, collections::HashSet, fmt::Display, fs};

use console::{Color, style};

use crate::{
    cli::core::{DisplayVersion, pluralise, print_blank_ln, print_ln, print_skipped_ln, print_success_ln},
    config::BelleConfig,
    environment::{Environment, VersionReq, manager::iter_envs},
    error::{AppError, IoErrorContext},
    registry::{
        self, AliasPackage, Package, PackageIdentifier, PackageSource, RegisteredPackage,
        error::RegistryNotExistContext, iter_installed_packages, iter_packages,
    },
    util::{get_isabelle_name, strip_isabelle_name},
};

/// Remove all packages from disk
pub fn clean_packages() -> Result<(), AppError> {
    let package_dir = BelleConfig::get_package_dir();
    if !package_dir.is_dir() {
        print_skipped_ln("package cache is already empty");
        return Ok(());
    }

    let count = std::fs::read_dir(&package_dir)
        .report_read("packages source", &package_dir)?
        .count();
    fs::remove_dir_all(&package_dir).report_delete("packages source", &package_dir)?;

    print_success_ln(
        "Cleaned",
        format_args!("{} {}", style(count).bold(), pluralise(count, "package", "packages")),
    );

    Ok(())
}

/// Remove all metadata from disk
pub fn clean_metadata() -> Result<(), AppError> {
    let manifest_dir = BelleConfig::get_manifest_dir();

    if !manifest_dir.is_dir() {
        print_skipped_ln("metadata is already empty");
        return Ok(());
    }

    let count = std::fs::read_dir(&manifest_dir)
        .report_read("packages manifests", &manifest_dir)?
        .count();
    fs::remove_dir_all(&manifest_dir).report_delete("packages manifests", &manifest_dir)?;

    // Pluralise does not make sense for this line
    print_success_ln("Cleaned", format_args!("{} packages metadata", style(count).bold()));

    Ok(())
}

/// List versions of a package in our local metadata
pub fn list_versions(name: &str) -> Result<(), AppError> {
    let versions = registry::get_package_versions(name);

    if versions.is_empty() {
        print_skipped_ln("line");
        println!("No versions of package {} installed", name)
    } else {
        let mut installed_count = 0;

        for version in &versions {
            let line = format!("{}", DisplayVersion::Explicit(&version.version));
            if version.exists_locally() {
                print_ln("Installed", Color::Cyan, line);
                installed_count += 1;
            } else {
                print_blank_ln(line);
            }
        }

        print_success_ln(
            "Listed",
            format_args!(
                "{} {} for {} ({} installed).",
                style(versions.len()).bold(),
                pluralise(versions.len(), "version", "versions"),
                style(name).cyan().bright(),
                style(installed_count).bold(),
            ),
        );
    }

    Ok(())
}

/// Prints nicely formatted metadata for a package to the console
fn print_meta(meta: &Package, alias: Option<&AliasPackage>) {
    if let Some(alias) = alias {
        println!(
            "{} {} {}",
            style("Alias").dim().bold(),
            &alias.name,
            DisplayVersion::Implicit(&alias.version),
        )
    }

    let header = format!(
        "{} {} {}",
        style(&meta.name).cyan().bold(),
        style(&meta.title).bold(),
        DisplayVersion::Explicit(&meta.version)
    );
    println!("{}", header);

    println!("{}", style("─".repeat(console::measure_text_width(&header))).dim());

    // Try to parse HTML and display nicely
    let formatted_abstract = html2text::from_read(meta.r#abstract.as_bytes(), 80)
        .map(Cow::Owned)
        // Fallback to just displaying raw text
        .unwrap_or(Cow::Borrowed(&meta.r#abstract));
    println!("{}", formatted_abstract);

    if let Some(note) = &meta.note {
        println!("{} {}", style("Note:").yellow().bold(), note);
    }

    println!();

    fn print_heading<T: Display>(heading: T) {
        println!("{}", style(format!("{}:", heading)).bold());
    }

    fn print_attribute<K: Display, V: Display>(key: K, value: V) {
        println!("{:<8} {}", style(key).bold(), value);
    }

    print_attribute("Date", meta.date);
    if !meta.topics.is_empty() {
        print_attribute("Topics", meta.topics.join(", "));
    }
    print_attribute("License", &meta.licence);

    let source_str = match &meta.source {
        PackageSource::Afp(repo) => format!(
            "AFP {} {}",
            strip_isabelle_name(&repo.name),
            DisplayVersion::Implicit(repo.get_version())
        ),
        PackageSource::Remote { url } => format!("Remote: {}", url),
        PackageSource::Local { path } => format!("Local: {}", path.display()),
        _ => "Unknown Source".to_string(),
    };
    print_attribute("Source", source_str);

    println!();

    if !meta.authors.is_empty() {
        print_heading("Authors");
        for author in &meta.authors {
            print!(" {} {}", style("•").dim(), author.name);
            if let Some(email) = &author.email {
                print!(" {}", style(format!("<{}>", email)).dim());
            }
            if let Some(orcid) = &author.orcid {
                print!(" {}", style(format!("(ORCID:{})", orcid)).dim());
            }
            if let Some(homepages) = &author.homepages {
                for homepage in homepages {
                    print!(" {}", style(homepage).dim().underlined());
                }
            }
            println!()
        }
    }

    println!();

    print_heading("Isabelle Versions");
    for isabelle_version in &meta.isabelles {
        println!(
            " {} {:<6} {}",
            style("•").dim(),
            style(get_isabelle_name(isabelle_version)),
            DisplayVersion::Explicit(isabelle_version),
        )
    }

    println!();

    if !meta.dependencies.is_empty() {
        print_heading("Dependencies");

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
            println!(" {} {} {}", style("•").dim(), name, DisplayVersion::Explicit(&version))
        }

        for name in isabelle_dependencies {
            println!(" {} {}", style("•").dim(), style(name).dim().italic(),)
        }
    }

    println!();

    if !meta.provides.is_empty() {
        print_heading("Provides Packages");

        for alias in &meta.provides {
            println!(
                " {} {} {}",
                style("•").dim(),
                style(alias),
                DisplayVersion::Explicit(&meta.version),
            )
        }
    }

    println!();

    if !meta.extra.is_empty() {
        print_heading("Extra Information");

        for (key, value) in &meta.extra {
            println!(" {} {:<10} {}", style("•").dim(), style(key), value);
        }
    }
}

/// Display metadata for a specific package on the console, if a version is not given then the latest will be shown
pub fn print_package_meta(name: String, version: VersionReq) -> Result<(), AppError> {
    let package = match version {
        VersionReq::Given(v) => PackageIdentifier::new(name, v),
        VersionReq::Any => {
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

    // Print list of results
    for package in &results {
        print_blank_ln(format_args!("{}", highlight_match(package, &search)));
    }
    print_success_ln(
        "Found",
        format_args!(
            "{} {} for '{}'",
            style(results.len()).bold(),
            pluralise(results.len(), "result", "results"),
            style(search).cyan().bright()
        ),
    );
}

pub fn purge_packages() -> Result<(), AppError> {
    let mut used_packages: HashSet<PackageIdentifier> = HashSet::new();

    for env_name in iter_envs() {
        let env = Environment::get(&env_name)?.expect("Environment listed, but failed to be got");
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

    print_success_ln(
        "Cleaned",
        format_args!(
            "{} {}",
            style(removed).bold(),
            pluralise(removed, "package", "packages")
        ),
    );

    Ok(())
}
