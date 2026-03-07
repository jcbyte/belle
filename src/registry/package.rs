use std::{
    fs,
    io::{self, Cursor},
    path::PathBuf,
};

use anyhow::Context;
use zip::ZipArchive;

use crate::{
    config::BelleConfig,
    fetch::BelleClient,
    registry::{AliasPackage, Package, PackageIdentifier, PackageSource, RegisteredPackage},
};

#[cfg(windows)]
use junction::create as symlink;
#[cfg(unix)]
use std::os::unix::fs::symlink;

pub trait RegistrablePackage: Into<RegisteredPackage> {
    fn get_identifier(&self) -> PackageIdentifier;

    fn register(self) -> anyhow::Result<()> {
        let identifier = self.get_identifier();

        let registerable_package: RegisteredPackage = self.into();

        // Write metadata manifest
        let manifest_file = identifier.get_manifest_path();
        let manifest_toml_string = toml::to_string(&registerable_package)
            .with_context(|| format!("Could not create {} TOML manifest", identifier))?;
        // Recursively create parent directory and parents so that we can write to the file
        if let Some(parent) = manifest_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create {} manifest directories on disk", identifier))?;
        }
        fs::write(manifest_file, manifest_toml_string)
            .with_context(|| format!("Could not write {} TOML manifest to disk", identifier))?;

        return Ok(());
    }
}

impl RegistrablePackage for Package {
    fn get_identifier(&self) -> PackageIdentifier {
        PackageIdentifier::from(self)
    }
}

impl RegistrablePackage for AliasPackage {
    fn get_identifier(&self) -> PackageIdentifier {
        PackageIdentifier::from(self)
    }
}

impl Package {
    pub async fn get_package(&self, client: &BelleClient) -> anyhow::Result<()> {
        let package_location = PackageIdentifier::from(self).get_theory_location();

        match &self.source {
            PackageSource::Afp(..) | PackageSource::Remote { .. } => {
                let bytes = match &self.source {
                    PackageSource::Afp(repo) => client.get_afp_package(&self.name, repo).await?,
                    PackageSource::Remote { url } => client.get_remote_package(url.clone()).await?,
                    _ => unreachable!(),
                };

                let reader = Cursor::new(bytes);
                let mut archive = ZipArchive::new(reader)?;

                // Find the inner folder that has the `ROOT` file
                let mut prefix = PathBuf::new();
                for i in 0..archive.len() {
                    let file = archive.by_index(i)?;

                    if file.name().ends_with("ROOT") {
                        if let Some(parent) = PathBuf::from(file.name()).parent() {
                            prefix = parent.to_path_buf();
                        }
                        break;
                    }
                }

                // Extract contents of the archive from the prefixed location
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let filename = file.enclosed_name().ok_or(anyhow::anyhow!("Invalid file path in archive"))?;

                    if let Ok(stripped_path) = filename.strip_prefix(&prefix) {
                        let file_src = package_location.join(stripped_path);

                        if file.is_dir() {
                            fs::create_dir_all(&file_src)?;
                        } else {
                            if let Some(parent) = file_src.parent() {
                                fs::create_dir_all(parent)?;
                            };
                            let mut out_file = fs::File::create(&file_src)?;
                            io::copy(&mut file, &mut out_file)?;
                        }
                    }
                }
            }
            // Create a symlink from packages directory to given directory
            PackageSource::Local { path } => {
                // Create a temporary symlink and overwrite to avoid `AlreadyExists` errors
                let temp_link = package_location.with_added_extension("tmp");

                symlink(path, &temp_link).context("Failed to create junction/symlink for active environment")?;
                fs::rename(temp_link, package_location)
                    .context("Failed to overwrite existing junction/symlink for the active environment")?;
            }
            PackageSource::Default => anyhow::bail!("Source is not given for this package"),
        };

        return Ok(());
    }
}

impl PackageIdentifier {
    /// Get manifest path for the given package
    fn get_manifest_path(&self) -> PathBuf {
        // Manifest file is located within `$manifest_dir/{name}/{version}.toml`
        let manifest_dir = BelleConfig::read_config(|c| c.get_manifest_dir());
        let manifest_file = manifest_dir
            .join(self.name.clone())
            .join(self.version.to_string())
            .with_added_extension("toml");

        return manifest_file;
    }

    /// Get theory location
    pub fn get_theory_location(&self) -> PathBuf {
        // Theories are is located within `$theory_dir/{name}/{version}.toml`
        let theories_dir = BelleConfig::read_config(|c| c.get_theory_dir());
        let theory_dir = theories_dir.join(self.name.clone()).join(self.version.to_string());

        return theory_dir;
    }

    /// Check that package exists in our metadata store on disk
    pub fn package_exists(&self) -> bool {
        let manifest_file = self.get_manifest_path();
        return manifest_file.is_file();
    }

    /// Retrieve a packages manifest data, it may return an alias or the value (to automatically resolve this use `get_resolved_package_manifest`)
    /// Will be `None` if the package does not exist in our metadata store
    pub fn get_package_manifest(&self) -> anyhow::Result<Option<RegisteredPackage>> {
        let manifest_file = self.get_manifest_path();

        // If the manifest file does not exist then it is not in our store
        if !manifest_file.is_file() {
            return Ok(None);
        }

        let manifest_toml_string = fs::read_to_string(manifest_file)
            .with_context(|| format!("Failed to read manifest file for {} package", self))?;
        let package: RegisteredPackage = toml::from_str(&manifest_toml_string)
            .with_context(|| format!("Failed to parse TOML for {} manifest file", self))?;

        return Ok(Some(package));
    }

    /// Retrieve a packages manifest resolving all aliases data
    /// Will be `None` if the package does not exist in our metadata store
    pub fn get_resolved_package_manifest(&self) -> anyhow::Result<Option<Package>> {
        let package = self.get_package_manifest()?;

        if let Some(registered_package) = package {
            return match registered_package {
                RegisteredPackage::Package(package) => Ok(Some(package)),
                RegisteredPackage::Alias(alias) => alias.alias.get_resolved_package_manifest(),
            };
        }

        return Ok(None);
    }

    /// Get if this package has been downloaded already
    pub fn exists_locally(&self) -> bool {
        let theory_dir = self.get_theory_location();
        return theory_dir.is_dir();
    }

    /// Remove the package source files from disk
    pub fn remove(&self) -> anyhow::Result<()> {
        let theory_dir = self.get_theory_location();

        if theory_dir.is_dir() {
            fs::remove_dir_all(theory_dir)?;
        }

        return Ok(());
    }
}
