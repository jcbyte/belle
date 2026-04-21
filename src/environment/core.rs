use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use pubgrub::SemanticVersion;

use crate::{
    config::BelleConfig,
    environment::{
        Environment, LOCKFILE_NAME, PackageListing, PackageType, error::EnvironmentError, types::VersionReq,
    },
    error::{AppError, IoErrorContext, IoPathErrorContext, ParseErrorContext},
    isabelle::IsabellePathContext,
    registry::{
        PackageIdentifier, RegisteredPackage,
        error::{RegistryError, RegistryNotExistContext},
        get_package_versions,
    },
    resolver::{BelleDepsProvider, ISABELLE_PACKAGE},
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

    pub fn has_active() -> bool {
        let active_env = BelleConfig::get_active_env_link();
        let env_file = Self::join_env_file(active_env);

        env_file.is_file()
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
        let freeze_file = Self::get_lockfile();

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

    pub fn get_isabelle_version(&self) -> VersionReq {
        match self.lock.get(ISABELLE_PACKAGE) {
            Some(v) => VersionReq::Given(*v),
            None => self.isabelle.clone(),
        }
    }

    fn load(env_file: &Path) -> Result<Self, AppError> {
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

    pub fn get_lockfile() -> PathBuf {
        PathBuf::from(".").join(LOCKFILE_NAME)
    }

    pub fn freeze(&self) -> Result<(), AppError> {
        let lockfile = Self::get_lockfile();

        let content = toml::to_string(self).report_file("environment", &lockfile)?;
        fs::write(&lockfile, content).report_save("environment lockfile", &lockfile)?;

        Ok(())
    }

    /// Sync the contents of the freeze file into this environment
    pub fn sync(&mut self) -> Result<(), AppError> {
        let frozen_env = Self::frozen()?.ok_or(EnvironmentError::NoLockFile {
            path: Self::get_lockfile(),
        })?;

        // Sync versions, including lock from the frozen environment
        self.isabelle = frozen_env.isabelle;
        self.packages = frozen_env.packages;
        self.lock = frozen_env.lock;

        Ok(())
    }

    pub fn add_package(&mut self, name: String, version: VersionReq) -> Result<(), AppError> {
        // Check that this package exists before adding
        match &version {
            VersionReq::Given(v) => {
                let adding = PackageIdentifier::new(&name, v);
                if !adding.package_exists() {
                    return Err(RegistryError::NotExist { package_id: adding }.into());
                }
            }
            VersionReq::Any => {
                if get_package_versions(&name).is_none() {
                    return Err(RegistryError::NameNotExist { package: name }.into());
                }
            }
        };

        self.packages.insert(name, version);
        Ok(())
    }

    pub fn remove_package(&mut self, name: &str) -> Result<(), EnvironmentError> {
        self.packages.remove(name);
        Ok(())
    }

    pub fn resolve_lock(&mut self) -> Result<(), AppError> {
        let resolved_packages: HashMap<String, SemanticVersion> =
            BelleDepsProvider::resolve(self.isabelle.clone(), self.packages.clone())?
                .into_iter()
                .collect();
        self.lock = resolved_packages;

        Ok(())
    }

    pub fn get_package_listing(&self, name: &str) -> Option<PackageListing> {
        self.lock.get(name).map(|&locked_version| match self.packages.get(name) {
            Some(listed_version) => PackageListing {
                name: name.to_string(),
                version: locked_version,
                kind: if listed_version.is_any() {
                    PackageType::ImplicitDirect
                } else {
                    PackageType::ExplicitDirect
                },
            },
            None => PackageListing {
                name: name.to_string(),
                version: locked_version,
                kind: PackageType::Transitive,
            },
        })
    }

    pub fn iter_packages(&self) -> impl Iterator<Item = PackageListing> {
        self.lock.keys().map(|name| {
            self.get_package_listing(name)
                .expect("Package known to exist cannot now be found")
        })
    }

    /// Get packages installed by the user, filtering Isabelle's built in ones.
    pub fn iter_user_packages(&self) -> impl Iterator<Item = (&String, &SemanticVersion)> {
        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());

        self.lock
            .iter()
            // Remove Isabelle packages
            .filter(move |&(name, _version)| name != ISABELLE_PACKAGE && !isabelle_packages.contains(name))
    }

    pub fn unpin_package_versions(&mut self) {
        self.packages = self.packages.keys().map(|name| (name.clone(), VersionReq::Any)).collect();
    }

    pub fn migrate_isabelle(&mut self, version: VersionReq) {
        self.isabelle = version;
    }

    pub fn create_roots_file(&self) -> Result<(), AppError> {
        let roots_file_path = self.get_roots_file();
        let roots_file = File::create(&roots_file_path).report_save("root file", &roots_file_path)?;
        let mut writer = BufWriter::new(roots_file);

        for (name, version) in self.iter_user_packages() {
            let package = PackageIdentifier::new(name, *version);

            // If the package is an alias, then don't add it.
            // The main package would have been a dependency, and this will also be added
            if let RegisteredPackage::Alias(_) =
                package.get_package_manifest()?.report_package_nonexistent(package.clone())?
            {
                continue;
            };

            // Create Isabelle recognised path (os specific implementations)
            let package_src = package.get_package_location();
            let full_package_src =
                dunce::canonicalize(&package_src).report_read("package source directory", &package_src)?;
            let package_root_str = full_package_src.to_isabelle_path().report_path(&package_src)?;

            // Write this packages source directory to root file
            writeln!(writer, "{}", package_root_str).report_save("root file", &roots_file_path)?;
        }

        writer.flush().report_save("root file", &roots_file_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::types::VersionReq;
    use pubgrub::SemanticVersion;

    #[test]
    fn test_getting_isabelle_version() {
        let no_isabelle_env = Environment {
            name: "test".to_string(),
            isabelle: VersionReq::Any,
            packages: HashMap::new(),
            lock: HashMap::new(),
        };

        assert!(no_isabelle_env.get_isabelle_version().is_any());

        let explicit_isabelle_env = Environment {
            name: "test".to_string(),
            isabelle: VersionReq::Given(SemanticVersion::one()),
            packages: HashMap::new(),
            lock: HashMap::new(),
        };

        assert_eq!(
            explicit_isabelle_env.get_isabelle_version(),
            VersionReq::Given(SemanticVersion::one())
        );

        let resolved_isabelle_env = Environment {
            name: "test".to_string(),
            isabelle: VersionReq::Any,
            packages: HashMap::new(),
            lock: HashMap::from([(ISABELLE_PACKAGE.to_string(), SemanticVersion::two())]),
        };

        assert_eq!(
            resolved_isabelle_env.get_isabelle_version(),
            VersionReq::Given(SemanticVersion::two())
        );

        // Prioritise locked Isabelle version
        let differing_isabelle_env = Environment {
            name: "test".to_string(),
            isabelle: VersionReq::Given(SemanticVersion::new(3, 1, 0)),
            packages: HashMap::new(),
            lock: HashMap::from([(ISABELLE_PACKAGE.to_string(), SemanticVersion::new(3, 0, 0))]),
        };

        assert_eq!(
            differing_isabelle_env.get_isabelle_version(),
            VersionReq::Given(SemanticVersion::new(3, 0, 0))
        );
    }

    #[test]
    fn test_unpin_package_versions() {
        let mut env = Environment {
            name: "test".to_string(),
            isabelle: VersionReq::Any,
            packages: HashMap::from([
                ("pkg1".to_string(), VersionReq::Given(SemanticVersion::one())),
                ("pkg2".to_string(), VersionReq::Given(SemanticVersion::two())),
                ("pkg3".to_string(), VersionReq::Any),
            ]),
            lock: HashMap::new(),
        };

        env.unpin_package_versions();

        let pkg1_v = env.packages.get("pkg1");
        assert!(pkg1_v.is_some());
        assert!(pkg1_v.unwrap().is_any());
        let pkg2_v = env.packages.get("pkg2");
        assert!(pkg2_v.is_some());
        assert!(pkg2_v.unwrap().is_any());
        let pkg3_v = env.packages.get("pkg3");
        assert!(pkg3_v.is_some());
        assert!(pkg3_v.unwrap().is_any());
    }

    #[test]
    fn test_get_package_listing() {
        let mut env = Environment {
            name: "test".to_string(),
            isabelle: VersionReq::Any,
            packages: HashMap::new(),
            lock: HashMap::new(),
        };

        // Test Transitive
        env.lock.insert("pkg".to_string(), SemanticVersion::one());

        let listing = env.get_package_listing("pkg");
        assert!(listing.is_some());
        let listing = listing.unwrap();
        assert_eq!(listing.name, "pkg");
        assert_eq!(listing.version, SemanticVersion::one());
        assert_eq!(listing.kind, PackageType::Transitive);

        // Test Direct (implicit)
        env.packages.insert("pkg".to_string(), VersionReq::Any);

        let listing = env.get_package_listing("pkg");
        assert!(listing.is_some());
        let listing = listing.unwrap();
        assert_eq!(listing.name, "pkg");
        assert_eq!(listing.version, SemanticVersion::one());
        assert_eq!(listing.kind, PackageType::ImplicitDirect);

        // Test Direct (explicit)
        env.packages
            .insert("pkg".to_string(), VersionReq::Given(SemanticVersion::one()));

        let listing = env.get_package_listing("pkg");
        assert!(listing.is_some());
        let listing = listing.unwrap();
        assert_eq!(listing.name, "pkg");
        assert_eq!(listing.version, SemanticVersion::one());
        assert_eq!(listing.kind, PackageType::ExplicitDirect);
    }
}
