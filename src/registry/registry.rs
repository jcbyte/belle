use std::str::FromStr;

use anyhow::Context;
use pubgrub::SemanticVersion;
use walkdir::WalkDir;

use crate::{config::BelleConfig, registry::PackageIdentifier};

/// Scan for all installed packages
pub fn iter_installed_packages() -> impl Iterator<Item = PackageIdentifier> {
    let packages_dir = BelleConfig::read_config(|c| c.get_theory_dir());

    return WalkDir::new(packages_dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|entry| entry.ok())
        // If name and version cant be extracted remove them from results
        .filter_map(|entry| {
            // Extract the last two components: [..., "name", "version"]
            let mut p = entry.path().components().rev();
            let version_str = p.next()?.as_os_str().to_string_lossy().to_string();
            let name = p.next()?.as_os_str().to_string_lossy().to_string();

            let version = SemanticVersion::from_str(&version_str).ok()?;

            Some(PackageIdentifier { name, version })
        });
}

/// Scan for all packages in registry
pub fn iter_packages() -> impl Iterator<Item = String> {
    let manifest_dir = BelleConfig::read_config(|c| c.get_manifest_dir());

    WalkDir::new(manifest_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
}

/// Scan for all versions for a specific package
pub fn get_package_versions(name: &String) -> anyhow::Result<Vec<PackageIdentifier>> {
    let package_manifests = BelleConfig::read_config(|c| c.get_manifest_dir()).join(name);

    let versions: Result<Vec<PackageIdentifier>, _> = WalkDir::new(package_manifests)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.path().file_stem().map(|filename| filename.to_string_lossy().to_string()))
        .map(|version_str| {
            SemanticVersion::from_str(&version_str)
                .map(|version| PackageIdentifier {
                    name: name.clone(),
                    version,
                })
                .with_context(|| format!("Could not parse version '{}' for package {}", version_str, name))
        })
        .collect();

    return Ok(versions?);
}
