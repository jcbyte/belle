use std::{
    fs,
    io::{self, Cursor},
    path::PathBuf,
};

use zip::ZipArchive;

use crate::{
    config::BelleConfig,
    error::{AppError, ArchiveErrorContext, IoError, IoErrorContext, ParseErrorContext},
    fetch::BelleClient,
    registry::{AliasPackage, Package, PackageIdentifier, PackageSource, RegisteredPackage, error::RegistryError},
    util::create_parent_dirs,
};

#[cfg(windows)]
use junction::create as symlink;
#[cfg(unix)]
use std::os::unix::fs::symlink;

pub trait RegistrablePackage: Into<RegisteredPackage> {
    fn register(self) -> Result<(), AppError>
    where
        for<'a> &'a Self: Into<PackageIdentifier>,
    {
        let identifier: PackageIdentifier = (&self).into();
        let registerable_package: RegisteredPackage = self.into();

        // Write metadata manifest
        let manifest_file = identifier.get_manifest_path();
        let manifest_toml_string =
            toml::to_string(&registerable_package).report_file(format!("{} manifest", identifier), &manifest_file)?;

        create_parent_dirs(&manifest_file)
            .report_save(format!("{} manifest directories", identifier), &manifest_file)?;
        fs::write(&manifest_file, manifest_toml_string)
            .report_save(format!("{} manifest", identifier), &manifest_file)?;

        Ok(())
    }
}

impl RegistrablePackage for Package {}
impl RegistrablePackage for AliasPackage {}

impl Package {
    pub async fn get_package(&self) -> Result<(), AppError> {
        let package_location = PackageIdentifier::from(self).get_theory_location();

        match &self.source {
            PackageSource::Afp(..) | PackageSource::Remote { .. } => {
                let client = BelleClient::get()?;

                let bytes = match &self.source {
                    PackageSource::Afp(repo) => client.get_afp_package(&self.name, repo).await?,
                    PackageSource::Remote { url } => client.get_remote_package(url).await?,
                    _ => unreachable!(),
                };

                let reader = Cursor::new(bytes);
                let mut archive = ZipArchive::new(reader)
                    .report_read(format!("fetched {} package source", PackageIdentifier::from(self)))?;

                // Find the inner folder that has the `ROOT` file
                let mut prefix = PathBuf::new();
                for i in 0..archive.len() {
                    let file = archive
                        .by_index(i)
                        .report_index(format!("fetched {} package source", PackageIdentifier::from(self)), i)?;

                    if file.name().ends_with("ROOT") {
                        if let Some(parent) = PathBuf::from(file.name()).parent() {
                            prefix = parent.to_path_buf();
                        }
                        break;
                    }
                }

                // Extract contents of the archive from the prefixed location
                for i in 0..archive.len() {
                    let mut file = archive
                        .by_index(i)
                        .report_index(format!("{} package source", PackageIdentifier::from(self)), i)?;
                    // If the path is unsafe, skip
                    let Some(filename) = file.enclosed_name() else { continue };

                    if let Ok(stripped_path) = filename.strip_prefix(&prefix) {
                        let file_src = package_location.join(stripped_path);

                        if file.is_dir() {
                            fs::create_dir_all(&file_src).report_save(
                                format!("{} package source directories", PackageIdentifier::from(self)),
                                &file_src,
                            )?;
                        } else {
                            create_parent_dirs(&file_src).report_save(
                                format!("{} package source directories", PackageIdentifier::from(self)),
                                &file_src,
                            )?;
                            let mut out_file = fs::File::create(&file_src)
                                .report_save(format!("{} package source", PackageIdentifier::from(self)), &file_src)?;
                            io::copy(&mut file, &mut out_file)
                                .report_save(format!("{} package source", PackageIdentifier::from(self)), &file_src)?;
                        }
                    }
                }
            }
            // Create a symlink from packages directory to given directory
            PackageSource::Local { path } => {
                // Create a temporary symlink and overwrite to avoid `AlreadyExists` errors
                let temp_link = package_location.with_added_extension("tmp");

                symlink(path, &temp_link).report_save("active environment symlink", &temp_link)?;
                fs::rename(temp_link, &package_location)
                    .report_save("active environment symlink", &package_location)?;
            }
            PackageSource::Default => Err(RegistryError::NoSource {
                package_id: self.into(),
            })?,
        };

        Ok(())
    }
}

impl PackageIdentifier {
    /// Get manifest path for the given package
    fn get_manifest_path(&self) -> PathBuf {
        // Manifest file is located within `$manifest_dir/{name}/{version}.toml`
        let manifest_dir = BelleConfig::read_config(|c| c.get_manifest_dir());

        manifest_dir
            .join(self.name.clone())
            .join(self.version.to_string())
            .with_added_extension("toml")
    }

    /// Get theory location
    pub fn get_theory_location(&self) -> PathBuf {
        // Theories are is located within `$theory_dir/{name}/{version}.toml`
        let theories_dir = BelleConfig::read_config(|c| c.get_theory_dir());

        theories_dir.join(self.name.clone()).join(self.version.to_string())
    }

    /// Check that package exists in our metadata store on disk
    pub fn package_exists(&self) -> bool {
        let manifest_file = self.get_manifest_path();
        manifest_file.is_file()
    }

    /// Retrieve a packages manifest data, it may return an alias or the value (to automatically resolve this use `get_resolved_package_manifest`)
    /// Will be `None` if the package does not exist in our metadata store
    pub fn get_package_manifest(&self) -> Result<Option<RegisteredPackage>, AppError> {
        let manifest_file = self.get_manifest_path();

        // If the manifest file does not exist then it is not in our store
        if !manifest_file.is_file() {
            return Ok(None);
        }

        let manifest_toml_string =
            fs::read_to_string(&manifest_file).report_read(format!("{} manifest", self), &manifest_file)?;
        let package: RegisteredPackage =
            toml::from_str(&manifest_toml_string).report_file(format!("{} manifest", self), &manifest_file)?;

        Ok(Some(package))
    }

    /// Retrieve a packages manifest resolving all aliases data
    /// Will be `None` if the package does not exist in our metadata store
    pub fn get_resolved_package_manifest(&self) -> Result<Option<Package>, AppError> {
        let package = self.get_package_manifest()?;

        if let Some(registered_package) = package {
            return match registered_package {
                RegisteredPackage::Package(package) => Ok(Some(package)),
                RegisteredPackage::Alias(alias) => alias.alias.get_resolved_package_manifest(),
            };
        }

        Ok(None)
    }

    /// Get if this package has been downloaded already
    pub fn exists_locally(&self) -> bool {
        let theory_dir = self.get_theory_location();
        theory_dir.is_dir()
    }

    /// Remove the package source files from disk
    pub fn remove(&self) -> Result<(), IoError> {
        let theory_dir = self.get_theory_location();

        if theory_dir.is_dir() {
            fs::remove_dir_all(&theory_dir).report_delete(format!("{} package source", self), &theory_dir)?;
        }

        Ok(())
    }
}
