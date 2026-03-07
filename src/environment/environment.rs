use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Context;

use crate::{
    config::BelleConfig,
    environment::{Environment, PackageListing, PackageType, types::VersionReq},
    registry::PackageIdentifier,
    resolver::{BelleDependencyProvider, ISABELLE_PACKAGE},
};

impl Environment {
    /// Create a new environment with the given name
    pub fn new(name: String, isabelle_version: VersionReq) -> anyhow::Result<Self> {
        let env_dir = Self::env_dir_for_name(&name);

        if env_dir.is_dir() {
            anyhow::bail!("Environment '{}' already exists", &name);
        }

        let env = Environment {
            name,
            isabelle: isabelle_version,
            packages: HashMap::new(),
            lock: HashMap::new(),
        };

        return Ok(env);
    }

    /// Get the active environment, if any
    pub fn active() -> anyhow::Result<Option<Self>> {
        let active_env = BelleConfig::read_config(|c| c.get_active_env_link());
        let env_file = Self::join_env_file(active_env);

        if !env_file.is_file() {
            return Ok(None);
        };

        return Ok(Some(Self::load(env_file)?));
    }

    pub fn get(name: String) -> anyhow::Result<Option<Self>> {
        let env_file = Self::env_file_for_name(&name);

        if !env_file.is_file() {
            return Ok(None);
        };

        return Ok(Some(Self::load(env_file)?));
    }

    /// Get the environment in the freeze file, if any
    pub fn frozen() -> anyhow::Result<Option<Self>> {
        let freeze_file = Self::get_freeze_file();

        if !freeze_file.is_file() {
            return Ok(None);
        }

        return Ok(Some(Self::load(freeze_file)?));
    }

    pub(crate) fn env_dir_for_name(name: &String) -> PathBuf {
        return BelleConfig::read_config(|c| c.get_env_dir()).join(name);
    }

    pub(crate) fn join_env_file(env_dir: PathBuf) -> PathBuf {
        return env_dir.join("env.toml");
    }

    pub(crate) fn env_file_for_name(name: &String) -> PathBuf {
        return Self::join_env_file(Self::env_dir_for_name(name));
    }

    fn get_env_dir(&self) -> PathBuf {
        return Self::env_dir_for_name(&self.name);
    }

    fn get_env_file(&self) -> PathBuf {
        return Self::join_env_file(self.get_env_dir());
    }

    fn load(env_file: PathBuf) -> anyhow::Result<Self> {
        let parsed_env = if env_file.is_file() {
            let content = fs::read_to_string(&env_file)
                .with_context(|| format!("Failed to read environment file at '{}'", env_file.display()))?;
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML environment file at '{}'", env_file.display()))?
        } else {
            anyhow::bail!("Environment file '{}' does not exist", env_file.display());
        };

        return Ok(parsed_env);
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let env_file = self.get_env_file();

        // Recursively create parent directory and parents so that we can write to the file
        if let Some(parent) = env_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Could not create {} environment directories on disk", &self.name))?;
        }

        let content =
            toml::to_string(self).with_context(|| format!("Failed to parse TOML for environment '{}'", &self.name))?;
        fs::write(env_file, content).with_context(|| format!("Failed to save environment '{}'", &self.name))?;

        return Ok(());
    }

    fn get_freeze_file() -> PathBuf {
        return PathBuf::from(".").join("belle.toml");
    }

    pub fn freeze(&self) -> anyhow::Result<()> {
        let freeze_file = Self::get_freeze_file();

        let content =
            toml::to_string(self).with_context(|| format!("Failed to parse TOML for environment '{}'", &self.name))?;
        fs::write(freeze_file, content)
            .with_context(|| format!("Failed to write to freeze file for '{}'", &self.name))?;

        return Ok(());
    }

    /// Sync the contents of the freeze file into this environment
    pub fn sync(&mut self) -> anyhow::Result<()> {
        let frozen_env = Self::frozen()?.ok_or(anyhow::anyhow!("No belle file found in workspace"))?;

        // Set the active packages to the ones from freeze file and save it back
        self.packages = frozen_env.packages;
        self.lock = frozen_env.lock;

        return Ok(());
    }

    pub fn add_package(&mut self, name: String, version: VersionReq) -> anyhow::Result<()> {
        if self.packages.contains_key(&name) {
            anyhow::bail!("Package '{}' is already installed in this environment", &name);
        }

        self.packages.insert(name, version);

        return Ok(());
    }

    pub fn remove_package(&mut self, name: &String) -> anyhow::Result<()> {
        self.packages.remove(name);

        return Ok(());
    }

    pub fn resolve_lock(&mut self) -> anyhow::Result<()> {
        let resolved_packages = BelleDependencyProvider::resolve(self.isabelle.clone(), self.packages.clone())?;
        self.lock = resolved_packages;

        return Ok(());
    }

    pub fn get_packages(&self) -> anyhow::Result<Vec<PackageListing>> {
        let packages = self
            .lock
            .iter()
            .map(|(name, version)| match self.packages.get(name) {
                None => {
                    return Ok(PackageListing {
                        name: name.clone(),
                        version: version.clone(),
                        kind: PackageType::Transitive,
                    });
                }
                Some(direct_version) => {
                    return Ok(PackageListing {
                        name: name.clone(),
                        version: version.clone(),
                        kind: PackageType::Direct {
                            given_version: !direct_version.is_any(),
                        },
                    });
                }
            })
            .collect();

        return packages;
    }

    pub fn migrate_isabelle(&mut self, version: VersionReq, unpin_existing: bool) -> anyhow::Result<()> {
        self.isabelle = version;

        if unpin_existing {
            self.packages = self
                .packages
                .iter()
                .map(|(name, _version)| (name.clone(), VersionReq::Any))
                .collect()
        }

        return Ok(());
    }

    pub fn get_missing_packages(&self) -> Vec<PackageIdentifier> {
        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

        return self
            .lock
            .iter()
            // Remove isabelle packages
            .filter(|(name, _version)| !name.eq(&ISABELLE_PACKAGE) && !isabelle_packages.contains(name))
            .map(|(name, version)| PackageIdentifier {
                name: name.clone(),
                version: version.clone(),
            })
            // Filter to only retrieve missing packages
            .filter(|p| !p.exists_locally())
            .collect();
    }
}
