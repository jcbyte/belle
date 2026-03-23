use pubgrub::{Dependencies, DependencyProvider, PackageResolutionStatistics, Ranges, SemanticVersion, resolve};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{HashMap, HashSet},
};

use crate::{
    config::BelleConfig,
    environment::VersionReq,
    error::AppError,
    isabelle::error::IsabelleError,
    registry::{PackageIdentifier, RegisteredPackage, error::RegistryNotExistContext, get_package_versions},
    resolver::{ISABELLE_PACKAGE, error::ResolverContext},
};

type SemVS = Ranges<SemanticVersion>;

pub struct BelleDependencyProvider {
    root_packages: HashMap<String, VersionReq>,

    /// Whether to update the list of valid isabelle versions as we resolve dependencies
    update_isabelle_versions: bool,
    /// List of seen isabelle versions from packages
    isabelle_versions: RefCell<HashSet<SemanticVersion>>,
    /// Cache for package versions
    package_versions: RefCell<HashMap<String, HashSet<SemanticVersion>>>,
}

impl BelleDependencyProvider {
    fn new(isabelle_version: VersionReq, root_packages: HashMap<String, VersionReq>) -> Result<Self, IsabelleError> {
        let isabelle_versions = match isabelle_version {
            // If an isabelle version is given, only allow this to be the available version
            // All packages will eventually reference an isabelle package
            VersionReq::Given(version) => HashSet::from([version]),
            // If any version is given, use all possible seen versions
            // Start with the list of known versions from config, versions packages declare will also be added to this set
            VersionReq::Any => BelleConfig::read_config(|c| c.isabelles.keys().copied().collect()),
        };

        Ok(Self {
            root_packages,
            update_isabelle_versions: isabelle_version.is_any(),
            isabelle_versions: RefCell::new(isabelle_versions),
            package_versions: RefCell::new(HashMap::new()),
        })
    }

    fn get_package_versions(&self, name: &str) -> HashSet<SemanticVersion> {
        if let Some(versions) = self.package_versions.borrow().get(name) {
            return versions.clone();
        }

        let mut cache = self.package_versions.borrow_mut();
        let fetched: HashSet<SemanticVersion> = get_package_versions(name)
            .map(|versions| versions.into_iter().map(|v| v.version).collect())
            // If this package cannot be found then give no versions
            .unwrap_or_default();
        cache.insert(name.to_string(), fetched.clone());

        fetched
    }
}

impl DependencyProvider for BelleDependencyProvider {
    fn choose_version(&self, package: &String, range: &SemVS) -> Result<Option<SemanticVersion>, AppError> {
        // Always use a version of 0.0.0 for the main package
        if package == "." {
            return Ok(Some(SemanticVersion::zero()));
        }

        let versions =
            if package == ISABELLE_PACKAGE || BelleConfig::read_config(|c| c.isabelle_packages.contains(package)) {
                // If this is an isabelle package (the global isabelle package or, a defined one from config) then pick a version from the available isabelle versions
                let isabelle_versions = self.isabelle_versions.borrow();
                isabelle_versions.clone()
            } else {
                // Else pick from the list of the packages versions
                self.get_package_versions(package)
            };

        // Return the highest version of the package that satisfies the range
        let top_valid_version = versions.iter().filter(|v| range.contains(v)).max();

        Ok(top_valid_version.cloned())
    }

    type Priority = Reverse<usize>;
    fn prioritize(
        &self,
        package: &String,
        range: &SemVS,
        _conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        // Prioritise this package the most
        if package == "." {
            return Reverse(0);
        }

        // Process isabelle packages last
        // This ensure that all 3rd party packages have been seen, and all versions of isabelle have been included
        if package == ISABELLE_PACKAGE || BelleConfig::read_config(|c| c.isabelle_packages.contains(package)) {
            return Reverse(usize::MAX);
        }

        // Prioritise packages with fewer compatible versions
        // If versions cannot be got, an empty HashSet is provided => Reverse(0)
        let versions = self.get_package_versions(package);
        let valid_versions_count = versions.iter().filter(|v| range.contains(v)).count();

        // Invert to pick packages with fewest versions
        Reverse(valid_versions_count)
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &SemanticVersion,
    ) -> Result<Dependencies<String, SemVS, Self::M>, AppError> {
        // If the package name is "." this is our root package so its dependencies are as given
        if package == "." {
            let deps: HashMap<String, Ranges<SemanticVersion>, rustc_hash::FxBuildHasher> = self
                .root_packages
                .iter()
                .map(|(name, version)| {
                    (
                        name.clone(),
                        match version {
                            // Only allow the specific version of a package if it is explicitly given
                            VersionReq::Given(v) => SemVS::singleton(v),
                            // Else allow any version
                            // THe individual packages will have tighter requirements causing conflicts, etc
                            VersionReq::Any => SemVS::full(),
                        },
                    )
                })
                .collect();

            return Ok(Dependencies::Available(deps));
        }

        // The main isabelle package has no further dependencies
        if package == ISABELLE_PACKAGE {
            return Ok(Dependencies::Available(HashMap::default()));
        }

        // Isabelle packages have isabelle as a dependency with the same version as themselves
        if BelleConfig::read_config(|c| c.isabelle_packages.contains(package)) {
            let isabelle_dep = FxHashMap::from_iter([(ISABELLE_PACKAGE.to_string(), SemVS::singleton(version))]);
            return Ok(Dependencies::Available(isabelle_dep));
        }

        let package = PackageIdentifier::new(package, version);
        let manifest = package.get_package_manifest()?.report_package_nonexistent(package)?;

        let mut deps: HashMap<String, SemVS, rustc_hash::FxBuildHasher> = HashMap::with_hasher(FxBuildHasher);

        match manifest {
            RegisteredPackage::Alias(alias) => {
                // If this package is an alias then just add its aliases package as a version
                deps.insert(alias.alias.name, SemVS::singleton(alias.alias.version));
            }
            RegisteredPackage::Package(meta) => {
                // Get list of isabelle versions allowed for this package
                let isabelle_versions = meta.isabelles;
                let isabelle_versions_range = isabelle_versions
                    .iter()
                    .fold(SemVS::empty(), |acc, version| acc.union(&SemVS::singleton(version)));

                for (name, version) in meta.dependencies {
                    // If the dependency is an isabelle package then, we can accept any versions of isabelle which this package accepts
                    if BelleConfig::read_config(|c| c.isabelle_packages.contains(&name)) {
                        deps.insert(name, isabelle_versions_range.clone());
                        continue;
                    }

                    // For regular dependencies
                    // Currently use a singleton so ony the exact package will match
                    // This is to ensure 1:1 reproducibility between environments
                    deps.insert(name, SemVS::singleton(version));
                }

                // Add isabelle itself as a dependency
                deps.insert(ISABELLE_PACKAGE.to_string(), isabelle_versions_range);

                // If we must collect all possible isabelle versions, then add this packages possible versions here
                if self.update_isabelle_versions {
                    let mut isabelle_version_list = self.isabelle_versions.borrow_mut();
                    isabelle_version_list.extend(isabelle_versions);
                }
            }
        }

        Ok(Dependencies::Available(deps))
    }

    type Err = AppError;
    type P = String;
    type V = SemanticVersion;
    type VS = SemVS;
    type M = String;
}

impl BelleDependencyProvider {
    pub fn resolve(
        isabelle: VersionReq,
        packages: HashMap<String, VersionReq>,
    ) -> Result<HashMap<String, SemanticVersion>, AppError> {
        let resolver = BelleDependencyProvider::new(isabelle, packages)?;

        let mut resolved_dependencies =
            resolve(&resolver, ".".to_string(), SemanticVersion::zero()).report_no_solution()?;
        resolved_dependencies.remove(".");

        Ok(resolved_dependencies.into_iter().collect())
    }
}
