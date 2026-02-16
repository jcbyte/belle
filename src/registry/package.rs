use std::{fs, path::PathBuf};

use anyhow::{Context, anyhow};
use pubgrub::SemanticVersion;

use super::Package;
use crate::{
    config,
    registry::{Manifest, PackageIdentifier},
};

impl PackageIdentifier {
    fn get_meta_path(&self) -> PathBuf {
        let config = config::BelleConfig::global();
        let meta_dir = config.root_dir.join("meta");
        let meta_file = meta_dir
            .join(self.name.clone())
            .join(self.version.to_string())
            .with_added_extension("toml");
        return meta_file;
    }

    fn get_manifest_path(&self) -> PathBuf {
        let config = config::BelleConfig::global();
        let manifest_dir = config.root_dir.join("manifest");
        let manifest_file = manifest_dir
            .join(self.name.clone())
            .join(self.version.to_string())
            .with_added_extension("toml");
        return manifest_file;
    }
}

impl Package {
    pub fn register(&self) -> anyhow::Result<()> {
        let identifier = PackageIdentifier::from(self);

        let meta_file = identifier.get_meta_path();
        let meta_toml_string =
            toml::to_string(self).with_context(|| format!("Could not create {} TOML metadata", identifier))?;
        if let Some(parent) = meta_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create {} metadata directories on disk", identifier))?;
        }
        fs::write(meta_file, meta_toml_string)
            .with_context(|| format!("Could not write {} TOML metadata to disk", identifier))?;

        let manifest_file = identifier.get_manifest_path();
        let manifest = Manifest::from(self);
        let manifest_toml_string =
            toml::to_string(&manifest).with_context(|| format!("Could not create {} TOML manifest", identifier))?;
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
    pub fn package_exists(&self) -> bool {
        let manifest_file = self.get_manifest_path();
        return manifest_file.is_file();
    }

    pub fn get_package_manifest(&self) -> anyhow::Result<Option<Manifest>> {
        let manifest_file = self.get_manifest_path();

        if !manifest_file.is_file() {
            return Ok(None);
        }

        let manifest_toml_string = fs::read_to_string(manifest_file)
            .with_context(|| format!("Failed to read manifest file for {} package", self))?;

        let manifest: Manifest = toml::from_str(&manifest_toml_string)
            .with_context(|| format!("Failed to parse TOML for {} manifest file", self))?;

        return Ok(Some(manifest));
    }

    pub fn get_package() {
        // should check locally, if not we need to download
        // what does this return where is it called form?
    }
}
