use anyhow::Context;
use pubgrub::{Dependencies, DependencyProvider, PackageResolutionStatistics, Ranges, SemanticVersion, resolve};
use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::{HashMap, hash_map::Entry},
};

// todo get a list of these and move to somewhere new
static ISA_PACKAGES: &[&str] = &["HOL-Real_Asymp", "HOL-Eisbach", "HOL-Analysis", "HOL-Cardinals"];
// todo make this all depend on isa_version which ensures a valid version across all isabelle packages

use crate::{
    registry::{PackageIdentifier, get_package_versions},
    resolver::SolverError,
};

type SemVS = Ranges<SemanticVersion>;

pub struct BelleDependencyProvider {
    root_packages: HashMap<String, Option<SemanticVersion>>,
    isabelle_versions: Vec<SemanticVersion>,

    /// Cache for package versions
    package_versions: RefCell<HashMap<String, Vec<SemanticVersion>>>,
}

impl BelleDependencyProvider {
    fn new(root_packages: HashMap<String, Option<SemanticVersion>>) -> Self {
        // todo get this from root packages
        let isabelle_versions = vec![SemanticVersion::new(2025, 2, 0), SemanticVersion::new(2025, 1, 0)];

        return Self {
            root_packages,
            isabelle_versions,
            package_versions: RefCell::new(HashMap::new()),
        };
    }

    fn get_package_versions(&self, name: &String) -> anyhow::Result<Vec<SemanticVersion>> {
        if let Some(versions) = self.package_versions.borrow().get(name) {
            return Ok(versions.clone());
        }

        let mut cache = self.package_versions.borrow_mut();
        let fetched = get_package_versions(name)?.into_iter().map(|v| v.version).collect::<Vec<_>>();
        cache.insert(name.clone(), fetched.clone());

        return Ok(fetched);
    }
}

impl DependencyProvider for BelleDependencyProvider {
    fn choose_version(&self, package: &String, range: &SemVS) -> Result<Option<SemanticVersion>, SolverError> {
        if package.eq(".") {
            return Ok(Some(SemanticVersion::zero()));
        }

        let versions = if !ISA_PACKAGES.contains(&package.as_str()) {
            self.get_package_versions(package)?
        } else {
            self.isabelle_versions.clone()
        };

        // Return the highest version of the package that satisfies the range
        let top_valid_version = versions.iter().map(|v| v).filter(|v| range.contains(v)).max().cloned();

        return Ok(top_valid_version);
    }

    type Priority = Reverse<usize>;
    fn prioritize(
        &self,
        package: &String,
        range: &SemVS,
        _conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        // Prioritize packages with fewer compatible versions

        if package.eq(".") {
            return Reverse(0);
        }

        // ! fix unsafe behaviour
        let versions = self.get_package_versions(package).unwrap();
        let valid_versions_count = versions.iter().filter(|v| range.contains(v)).count();

        // Invert to pick packages with fewest versions
        return Reverse(valid_versions_count);
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &SemanticVersion,
    ) -> Result<Dependencies<String, SemVS, Self::M>, SolverError> {
        println!("deps {}", package);

        // If the package name is "." this is our root package so its dependencies are as given
        if package.eq(".") {
            let deps: HashMap<String, Ranges<SemanticVersion>, rustc_hash::FxBuildHasher> = self
                .root_packages
                .iter()
                .map(|(name, version)| {
                    (
                        name.clone(),
                        match version {
                            Some(v) => SemVS::singleton(v),
                            None => SemVS::full(),
                        },
                    )
                })
                .collect();

            return Ok(Dependencies::Available(deps));
        }

        if ISA_PACKAGES.contains(&package.as_str()) {
            return Ok(Dependencies::Available(HashMap::default()));
        }

        let package = PackageIdentifier {
            name: package.clone(),
            version: version.clone(),
        };

        let manifest = package
            .get_package_manifest()?
            .with_context(|| format!("Package '{}' does not exist", package))?;

        let deps: HashMap<String, Ranges<SemanticVersion>, rustc_hash::FxBuildHasher> = manifest
            .dependencies
            .iter()
            // Use a singleton so ony the exact package will match
            .map(|(name, version)| (name.clone(), SemVS::singleton(version)))
            .collect();

        return Ok(Dependencies::Available(deps));
    }

    type Err = SolverError;
    type P = String;
    type V = SemanticVersion;
    type VS = SemVS;
    type M = String;
}

impl BelleDependencyProvider {
    pub fn resolve(
        packages: HashMap<String, Option<SemanticVersion>>,
    ) -> anyhow::Result<HashMap<String, SemanticVersion>> {
        let resolver = BelleDependencyProvider::new(packages);

        // todo what happens to errors here?
        let mut resolved_dependencies = resolve(&resolver, String::from("."), SemanticVersion::zero())?;
        resolved_dependencies.remove(".");

        return Ok(resolved_dependencies.into_iter().collect());
    }
}
