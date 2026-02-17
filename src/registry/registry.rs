use std::ffi::OsStr;

use pubgrub::SemanticVersion;
use walkdir::WalkDir;

use crate::{config::BelleConfig, registry::PackageIdentifier};

pub fn list_packages(version: &SemanticVersion) -> Vec<PackageIdentifier> {
    let target_file = format!("{}.toml", version.to_string());
    let target_os_file = OsStr::new(&target_file);

    let config = BelleConfig::global();
    let meta_root = config.root_dir.join("meta");

    return WalkDir::new(meta_root)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.file_name().eq(target_os_file))
        .filter_map(|file| {
            file.path()
                .parent()
                .and_then(|parent_path| parent_path.file_name())
                .map(|parent_name| parent_name.to_string_lossy().to_string())
        })
        .map(|package_name| PackageIdentifier {
            name: package_name,
            version: version.clone(),
        })
        .collect();
}
