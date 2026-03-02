use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::{
    config::BelleConfig,
    registry::{AliasPackage, Package, PackageIdentifier, RegisteredPackage},
};

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
        // Just call your existing logic here
        PackageIdentifier::from(self)
    }
}

impl RegistrablePackage for AliasPackage {
    fn get_identifier(&self) -> PackageIdentifier {
        // Just call your existing logic here
        PackageIdentifier::from(self)
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
    fn get_theory_location(&self) -> PathBuf {
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

    /// Retrieve a packages manifest data
    /// Will be `None` if the package does not exist in our metadata store
    pub fn get_package_manifest(&self) -> anyhow::Result<Option<Package>> {
        let manifest_file = self.get_manifest_path();

        // If the manifest file does not exist then it is not in our store
        if !manifest_file.is_file() {
            return Ok(None);
        }

        let manifest_toml_string = fs::read_to_string(manifest_file)
            .with_context(|| format!("Failed to read manifest file for {} package", self))?;
        let package: RegisteredPackage = toml::from_str(&manifest_toml_string)
            .with_context(|| format!("Failed to parse TOML for {} manifest file", self))?;

        return match package {
            RegisteredPackage::Package(package) => Ok(Some(package)),
            RegisteredPackage::Alias(alias) => {
                println!("getting {} for {}", alias.alias, alias.name);
                alias.alias.get_package_manifest()
            } // todo should i indicate that we've got an alias
        };
    }

    /// Get if this package has been downloaded already
    pub fn exists_locally(&self) -> bool {
        let theory_dir = self.get_theory_location();
        return theory_dir.is_dir();
    }
}
