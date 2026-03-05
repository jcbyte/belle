use std::{fs, path::PathBuf};

use anyhow::{self, Context};

use crate::{
    fetch::PACKAGE_FILE,
    registry::{AliasPackage, Package, PackageIdentifier},
};

pub fn get_local_package_meta(path: PathBuf) -> anyhow::Result<(Package, Vec<AliasPackage>)> {
    //
    let pkg_file = path.join(PACKAGE_FILE);
    if !pkg_file.is_file() {
        anyhow::bail!("Package manifest could not be found");
    }

    let package_content = fs::read_to_string(pkg_file).context("Could not read package manifest")?;
    let mut package =
        toml::from_str::<Package>(&package_content).context("Failed to parse TOML for package manifest")?;

    package.source = crate::registry::PackageSource::Local {
        path: path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalise path '{}'", path.to_string_lossy().to_string()))?,
    };

    let aliases: Vec<AliasPackage> = package
        .provides
        .iter()
        .map(|provided| AliasPackage {
            name: provided.clone(),
            version: package.version.clone(),
            alias: PackageIdentifier::from(&package),
        })
        .collect();

    return Ok((package, aliases));
}
