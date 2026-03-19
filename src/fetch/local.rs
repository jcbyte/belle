use std::{fs, path::Path};

use crate::{
    error::{AppError, IoErrorContext, ParseErrorContext},
    fetch::{PACKAGE_FILE, error::FetchError, types::ReturnedPackages},
    registry::{AliasPackage, Package, PackageIdentifier},
};

pub fn get_local_package_meta(path: &Path) -> Result<ReturnedPackages, AppError> {
    let pkg_file = path.join(PACKAGE_FILE);

    if !pkg_file.is_file() {
        return Err(FetchError::NoLocalManifest { path: pkg_file }.into());
    }

    let package_content = fs::read_to_string(&pkg_file).report_read("package manifest", &pkg_file)?;
    let mut package = toml::from_str::<Package>(&package_content).report_data("package manifest")?;

    // Set the package source to the local directory
    package.source = crate::registry::PackageSource::Local {
        path: path.canonicalize().report_read("package root", &path)?,
    };

    // Extract aliases to return them separately
    let aliases: Vec<AliasPackage> = package
        .provides
        .iter()
        .map(|provided| AliasPackage {
            name: provided.clone(),
            version: package.version,
            alias: PackageIdentifier::from(&package),
        })
        .collect();

    Ok(ReturnedPackages { package, aliases })
}
