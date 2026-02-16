use std::fs;

use anyhow::{Context, anyhow};
use pubgrub::SemanticVersion;

use super::Package;
use crate::{config, registry::Manifest};

impl From<&Package> for Manifest {
    fn from(value: &Package) -> Self {
        return Self {
            name: value.name.clone(),
            version: value.version.clone(),
            dependencies: value.dependencies.clone(),
        };
    }
}

impl Package {
    pub fn register(&self) -> anyhow::Result<()> {
        let config = config::BelleConfig::global();

        let meta_dir = config.root_dir.join("meta");
        let meta_file = meta_dir.join(&self.name).join(self.version.to_string());

        let meta_toml_string = toml::to_string(self)
            .with_context(|| format!("Could not create {}@{} TOML metadata", self.name, self.version))?;
        fs::write(meta_file, meta_toml_string);

        let manifest_dir = config.root_dir.join("manifest");
        let manifest_file = manifest_dir.join(&self.name).join(self.version.to_string());

        let manifest = Manifest::from(self);
        let manifest_toml_string = toml::to_string(&manifest)
            .with_context(|| format!("Could not create {}@{} TOML manifest", self.name, self.version))?;
        fs::write(manifest_file, manifest_toml_string);

        return Ok(());
    }

    pub fn get_package_meta(name: &String, version: &SemanticVersion) -> anyhow::Result<Option<Manifest>> {
        let config = config::BelleConfig::global();

        let meta_dir = config.root_dir.join("meta");
        let meta_file = meta_dir.join(name).join(version.to_string());

        if !meta_file.is_file() {
            return Ok(None);
        }
        let manifest_toml_string = fs::read_to_string(meta_file).with_context(|| {
            format!(
                "Failed to read metadata file for {}@{} package",
                name,
                version.to_string()
            )
        })?;

        let manifest: Manifest = toml::from_str(&manifest_toml_string).with_context(|| {
            format!(
                "Failed to parse TOML for {}@{} metadata file",
                name,
                version.to_string()
            )
        })?;

        return Ok(Some(manifest));
    }

    pub fn get_package() {
        // should check locally, if not we need to download
        // what does this return where is it called form?
    }
}
