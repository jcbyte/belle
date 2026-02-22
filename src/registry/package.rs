use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::{
    config::BelleConfig,
    registry::{Manifest, Package, PackageIdentifier},
};

impl PackageIdentifier {
    /// Get metadata path for the given package
    fn get_meta_path(&self) -> PathBuf {
        // Meta file is located within `$meta_dir/{name}/{version}.toml`
        let meta_dir = BelleConfig::read_config(|c| c.get_meta_dir());
        let meta_file = meta_dir
            .join(self.name.clone())
            .join(self.version.to_string())
            .with_added_extension("toml");

        return meta_file;
    }

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
        // Manifest file is located within `$theory_dir/{name}/{version}.toml`
        let theories_dir = BelleConfig::read_config(|c| c.get_theory_dir());
        let theory_dir = theories_dir.join(self.name.clone()).join(self.version.to_string());

        return theory_dir;
    }
}

impl Package {
    /// Write package metadata and manifest to disk
    pub fn register(&self) -> anyhow::Result<()> {
        let identifier = PackageIdentifier::from(self);

        // Write metadata
        let meta_file = identifier.get_meta_path();
        let meta_toml_string =
            toml::to_string(self).with_context(|| format!("Could not create {} TOML metadata", identifier))?;
        // Recursively create parent directory and parents so that we can write to the file
        if let Some(parent) = meta_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create {} metadata directories on disk", identifier))?;
        }
        fs::write(meta_file, meta_toml_string)
            .with_context(|| format!("Could not write {} TOML metadata to disk", identifier))?;

        // Write manifest
        let manifest_file = identifier.get_manifest_path();
        let manifest = Manifest::from(self);
        let manifest_toml_string =
            toml::to_string(&manifest).with_context(|| format!("Could not create {} TOML manifest", identifier))?;
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

impl PackageIdentifier {
    /// Check that package exists in our metadata store on disk
    pub fn package_exists(&self) -> bool {
        let manifest_file = self.get_manifest_path();
        return manifest_file.is_file();
    }

    /// Retrieve a packages manifest data
    /// Will be `None` if the package does not exist in our metadata store
    pub fn get_package_manifest(&self) -> anyhow::Result<Option<Manifest>> {
        let manifest_file = self.get_manifest_path();

        // If the manifest file does not exist then it is not in our store
        if !manifest_file.is_file() {
            return Ok(None);
        }

        let manifest_toml_string = fs::read_to_string(manifest_file)
            .with_context(|| format!("Failed to read manifest file for {} package", self))?;
        let manifest: Manifest = toml::from_str(&manifest_toml_string)
            .with_context(|| format!("Failed to parse TOML for {} manifest file", self))?;

        return Ok(Some(manifest));
    }

    /// Retrieve a packages metadata
    /// Will be `None` if the package does not exist in our metadata store
    pub fn get_package_meta(&self) -> anyhow::Result<Option<Package>> {
        let metadata_file = self.get_meta_path();

        // If the manifest file does not exist then it is not in our store
        if !metadata_file.is_file() {
            return Ok(None);
        }

        let meta_toml_string = fs::read_to_string(metadata_file)
            .with_context(|| format!("Failed to read metadata file for {} package", self))?;
        let meta: Package = toml::from_str(&meta_toml_string)
            .with_context(|| format!("Failed to parse TOML for {} metadata file", self))?;

        return Ok(Some(meta));
    }

    /// Get if this package has been downloaded already
    pub fn exists_locally(&self) -> bool {
        let theory_dir = self.get_theory_location();
        return theory_dir.is_dir();
    }

    /// todo
    pub fn get_package() {
        todo!()
        // should check locally, if not we need to download
        // what does this return where is it called form?
    }
}
