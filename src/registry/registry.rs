use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use pubgrub::SemanticVersion;
use walkdir::WalkDir;

use crate::{config::BelleConfig, registry::PackageIdentifier};

/// Scan for `$root/{name}/{version}` toml files or folders
pub fn iter_package_files(root_path: &PathBuf, version: &SemanticVersion) -> impl Iterator<Item = PathBuf> {
    let file_target = format!("{}.toml", version.to_string());
    let dir_target = version.to_string();

    return WalkDir::new(root_path)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(move |entry| {
            let name = entry.file_name().to_string_lossy().to_string();

            (entry.file_type().is_file() && name.eq(&file_target)) || //,
            (entry.file_type().is_dir() && name.eq(&dir_target))
        })
        .map(|file| file.path().to_path_buf());
}

pub fn get_package_versions(name: &String) -> anyhow::Result<Vec<PackageIdentifier>> {
    let package_manifests = BelleConfig::get_manifest_dir().join(name);

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
