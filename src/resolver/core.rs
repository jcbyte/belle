use anyhow::Context;
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
    resolver::{
        ISABELLE_PACKAGE,
        error::{ResolverContext, ResolverError},
    },
};

type SemVS = Ranges<SemanticVersion>;

pub struct BelleDependencyProvider {
    root_packages: HashMap<String, VersionReq>,

    /// List of seen isabelle versions from packages
    isabelle_versions: HashSet<SemanticVersion>,
    /// Cache for package versions
    package_versions: RefCell<HashMap<String, HashSet<SemanticVersion>>>,
}

impl BelleDependencyProvider {
    fn new(isabelle_version: VersionReq, root_packages: HashMap<String, VersionReq>) -> Result<Self, IsabelleError> {
        let linked_versions: HashSet<SemanticVersion> =
            BelleConfig::read_config(|c| c.isabelles.keys().cloned().collect());

        let isabelle_versions = match isabelle_version {
            // If an isabelle version is given, only allow this to be the available version
            // All theories will eventually reference an isabelle package
            VersionReq::Given(version) => {
                // If the given version of isabelle is not linked then throw
                if !linked_versions.contains(&version) {
                    return Err(IsabelleError::VersionNotLinked {
                        version: version.into(),
                    });
                }

                HashSet::from([version])
            }
            // If any version is given, use all registered versions
            VersionReq::Any => linked_versions,
        };

        Ok(Self {
            root_packages,
            isabelle_versions,
            package_versions: RefCell::new(HashMap::new()),
        })
    }

    fn get_package_versions(&self, name: &String) -> Result<HashSet<SemanticVersion>, AppError> {
        if let Some(versions) = self.package_versions.borrow().get(name) {
            return Ok(versions.clone());
        }

        let mut cache = self.package_versions.borrow_mut();
        let fetched: HashSet<SemanticVersion> = get_package_versions(name)?.into_iter().map(|v| v.version).collect();
        cache.insert(name.clone(), fetched.clone());

        Ok(fetched)
    }
}

impl DependencyProvider for BelleDependencyProvider {
    fn choose_version(&self, package: &String, range: &SemVS) -> Result<Option<SemanticVersion>, AppError> {
        if package.eq(".") {
            return Ok(Some(SemanticVersion::zero()));
        }

        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());
        let versions = if package.eq(ISABELLE_PACKAGE) || isabelle_packages.contains(package) {
            // If this is an isabelle package pick a version from the available isabelle versions
            self.isabelle_versions.clone()
        } else {
            // Else pick from the list of the packages versions
            self.get_package_versions(package)?
        };

        // Return the highest version of the package that satisfies the range
        let top_valid_version = versions.iter().filter(|v| range.contains(v)).max().cloned();

        Ok(top_valid_version)
    }

    type Priority = Reverse<usize>;
    fn prioritize(
        &self,
        package: &String,
        range: &SemVS,
        _conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        // Prioritise this package the most
        if package.eq(".") {
            return Reverse(0);
        }

        // Process isabelle packages last
        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());
        if package.eq(ISABELLE_PACKAGE) || isabelle_packages.contains(package) {
            return Reverse(usize::MAX);
        }

        // Prioritise packages with fewer compatible versions
        // If versions cannot be got, an empty HashSet is provided => Reverse(0)
        let versions = self.get_package_versions(package).unwrap_or_default();
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
        if package.eq(".") {
            let deps: HashMap<String, Ranges<SemanticVersion>, rustc_hash::FxBuildHasher> = self
                .root_packages
                .iter()
                .map(|(name, version)| {
                    (
                        name.clone(),
                        match version {
                            &VersionReq::Given(v) => SemVS::singleton(v),
                            VersionReq::Any => SemVS::full(),
                        },
                    )
                })
                .collect();

            return Ok(Dependencies::Available(deps));
        }

        let isabelle_packages = BelleConfig::read_config(|c| c.isabelle_packages.clone());
        if isabelle_packages.contains(package) {
            let isabelle_dep = FxHashMap::from_iter([(String::from(ISABELLE_PACKAGE), SemVS::singleton(version))]);
            return Ok(Dependencies::Available(isabelle_dep));
        }

        if package.eq(ISABELLE_PACKAGE) {
            return Ok(Dependencies::Available(HashMap::default()));
        }

        let package = PackageIdentifier::new(package, *version);

        let manifest = package
            .get_package_manifest()?
            .report_package_nonexistent(package.to_string())?;

        let mut deps: HashMap<String, SemVS, rustc_hash::FxBuildHasher> = HashMap::with_hasher(FxBuildHasher);

        match manifest {
            RegisteredPackage::Alias(alias) => {
                // If this package is an alias then just add its aliases package as a version
                deps.insert(alias.alias.name, SemVS::singleton(alias.alias.version));
            }
            RegisteredPackage::Package(manifest) => {
                let isabelle_versions = manifest
                    .isabelles
                    .iter()
                    .fold(SemVS::empty(), |acc, version| acc.union(&SemVS::singleton(version)));

                for (name, version) in manifest.dependencies {
                    // If the dependency is an isabelle package then, we can accept any versions of isabelle that this package accepts
                    if isabelle_packages.contains(&name) {
                        deps.insert(name, isabelle_versions.clone());
                        continue;
                    }

                    // Use a singleton so ony the exact package will match
                    deps.insert(name, SemVS::singleton(version));
                }

                // Add isabelle itself as a dependency
                deps.insert(String::from(ISABELLE_PACKAGE), isabelle_versions);
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
            resolve(&resolver, String::from("."), SemanticVersion::zero()).report_no_solution()?;
        resolved_dependencies.remove(".");

        Ok(resolved_dependencies.into_iter().collect())
    }
}
