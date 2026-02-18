use std::path::PathBuf;

use pubgrub::SemanticVersion;
use walkdir::WalkDir;

/// Scan for `$root/{name}/{version}` toml files or folders
pub fn scour_package_files(root_path: &PathBuf, version: &SemanticVersion) -> impl Iterator<Item = PathBuf> {
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
