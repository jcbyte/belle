use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use pubgrub::SemanticVersion;

use crate::{
    config::BelleConfig,
    environment::{Environment, PackageListing, PackageType, error::EnvironmentError, types::VersionReq},
    error::{AppError, IoErrorContext, IoPathErrorContext, ParseErrorContext},
    isabelle::IsabellePathContext,
    registry::PackageIdentifier,
    resolver::{BelleDependencyProvider, ISABELLE_PACKAGE},
    util::create_parent_dirs,
};

impl Environment {
    /// Create a new environment with the given name
    pub fn new(name: String, isabelle_version: VersionReq) -> Result<Self, EnvironmentError> {
        let env_dir = Self::env_dir_for_name(&name);

        if env_dir.is_dir() {
            return Err(EnvironmentError::AlreadyExists { name });
        }

        let env = Environment {
            name,
            isabelle: isabelle_version,
            packages: HashMap::new(),
            lock: HashMap::new(),
        };

        Ok(env)
    }

    /// Get the active environment, if any
    pub fn active() -> Result<Option<Self>, AppError> {
        let active_env = BelleConfig::get_active_env_link();
        let env_file = Self::join_env_file(active_env);

        if !env_file.is_file() {
            return Ok(None);
        };

        Ok(Some(Self::load(&env_file)?))
    }

    pub fn get(name: &str) -> Result<Option<Self>, AppError> {
        let env_file = Self::env_file_for_name(name);

        if !env_file.is_file() {
            return Ok(None);
        };

        Ok(Some(Self::load(&env_file)?))
    }

    /// Get the environment in the freeze file, if any
    pub fn frozen() -> Result<Option<Self>, AppError> {
        let freeze_file = Self::get_freeze_file();

        if !freeze_file.is_file() {
            return Ok(None);
        }

        Ok(Some(Self::load(&freeze_file)?))
    }

    pub fn env_dir_for_name(name: &str) -> PathBuf {
        BelleConfig::get_env_dir().join(name)
    }

    pub fn join_env_file(env_dir: PathBuf) -> PathBuf {
        env_dir.join("env.toml")
    }

    pub fn env_file_for_name(name: &str) -> PathBuf {
        Self::join_env_file(Self::env_dir_for_name(name))
    }

    fn get_env_dir(&self) -> PathBuf {
        Self::env_dir_for_name(&self.name)
    }

    fn get_env_file(&self) -> PathBuf {
        Self::join_env_file(self.get_env_dir())
    }

    fn get_roots_file(&self) -> PathBuf {
        self.get_env_dir().join("ROOTS")
    }

    fn load(env_file: &Path) -> Result<Self, AppError> {
        if !env_file.is_file() {
            return Err(EnvironmentError::FileDoesNotExist {
                path: env_file.to_path_buf(),
            }
            .into());
        }

        let content = fs::read_to_string(env_file).report_read("environment file", env_file)?;
        let parsed_env = toml::from_str(&content).report_file("environment file", env_file)?;

        Ok(parsed_env)
    }

    pub fn save(&self) -> Result<(), AppError> {
        let env_file = self.get_env_file();

        create_parent_dirs(&env_file).report_save(format!("{} environment directories", self.name), &env_file)?;
        let content = toml::to_string(self).report_file("environment", &env_file)?;
        fs::write(&env_file, content).report_save(format!("{} environment directories", self.name), &env_file)?;

        Ok(())
    }

    pub fn get_freeze_file() -> PathBuf {
        PathBuf::from(".").join("belle.toml")
    }

    pub fn freeze(&self) -> Result<(), AppError> {
        let freeze_file = Self::get_freeze_file();

        let content = toml::to_string(self).report_file("environment", &freeze_file)?;
        fs::write(&freeze_file, content).report_save("environment lockfile", &freeze_file)?;

        Ok(())
    }

    /// Sync the contents of the freeze file into this environment
    pub fn sync(&mut self) -> Result<(), AppError> {
        let frozen_env = Self::frozen()?.ok_or(EnvironmentError::NoLockFile {
            path: Self::get_freeze_file(),
        })?;

        // Set the active packages to the ones from freeze file and save it back
        self.packages = frozen_env.packages;
        self.lock = frozen_env.lock;

        Ok(())
    }

    pub fn add_package(&mut self, name: String, version: VersionReq) -> Result<(), EnvironmentError> {
        if self.packages.contains_key(&name) {
            Err(EnvironmentError::PackageAlreadyExists { package: name.clone() })?;
        }

        self.packages.insert(name, version);
        Ok(())
    }

    pub fn remove_package(&mut self, name: &str) -> Result<(), EnvironmentError> {
        if !self.packages.contains_key(name) {
            Err(EnvironmentError::PackageDoesNotExist {
                package: name.to_string(),
            })?;
        }

        self.packages.remove(name);
        Ok(())
    }

    pub fn resolve_lock(&mut self) -> Result<(), AppError> {
        let resolved_packages = BelleDependencyProvider::resolve(self.isabelle.clone(), self.packages.clone())?;
        self.lock = resolved_packages;

        Ok(())
    }

    pub fn iter_packages(&self) -> impl Iterator<Item = PackageListing> {
        self.lock.iter().map(|(name, version)| match self.packages.get(name) {
            None => PackageListing {
                name: name.clone(),
                version: *version,
                kind: PackageType::Transitive,
            },
            Some(direct_version) => PackageListing {
                name: name.clone(),
                version: *version,
                kind: PackageType::Direct {
                    given_version: !direct_version.is_any(),
                },
            },
        })
    }

    /// Get packages installed by the user, filtering isabelle's built in ones.
    pub fn iter_user_packages(&self) -> impl Iterator<Item = (&String, &SemanticVersion)> {
        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

        self.lock
            .iter()
            // Remove isabelle packages
            .filter(move |(name, _version)| !name.eq(&ISABELLE_PACKAGE) && !isabelle_packages.contains(name))
    }

    pub fn migrate_isabelle(&mut self, version: VersionReq, unpin_existing: bool) {
        self.isabelle = version;

        if unpin_existing {
            self.packages = self.packages.keys().map(|name| (name.clone(), VersionReq::Any)).collect();
        }
    }

    pub fn create_roots_file(&self) -> Result<(), AppError> {
        let packages_src = self
            .iter_user_packages()
            .map(|(name, version)| PackageIdentifier::new(name, *version))
            .map(|p| p.get_package_location());

        let roots_file_path = self.get_roots_file();
        let roots_file = File::create(&roots_file_path).report_save("root file", &roots_file_path)?;
        let mut writer = BufWriter::new(roots_file);

        for package_src in packages_src {
            let package_root_str = package_src.to_isabelle_path().report_path(&package_src)?;
            writeln!(writer, "{}", package_root_str).report_save("root file", &roots_file_path)?;
        }

        writer.flush().report_save("root file", &roots_file_path)?;

        Ok(())
    }
}
