use anyhow::Context;
use pubgrub::{Dependencies, DependencyProvider, PackageResolutionStatistics, Ranges, SemanticVersion, resolve};
use std::{cmp::Reverse, collections::HashMap};

use crate::{
    registry::{PackageIdentifier, get_package_versions},
    resolver::SolverError,
};

type SemVS = Ranges<SemanticVersion>;

pub struct BelleDependencyProvider {
    root_packages: Vec<PackageIdentifier>,
}

impl BelleDependencyProvider {
    fn new(root_packages: Vec<PackageIdentifier>) -> Self {
        return Self { root_packages };
    }
}

impl DependencyProvider for BelleDependencyProvider {
    fn choose_version(&self, package: &String, range: &SemVS) -> Result<Option<SemanticVersion>, SolverError> {
        // Return the highest version of the package that satisfies the range
        let versions = get_package_versions(package)?;
        let top_valid_version = versions.iter().map(|v| v.version).filter(|v| range.contains(&v)).max();

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

        // Always prioritise the root package
        if package == "." {
            return Reverse(0);
        }

        // ! fix unsafe behaviour
        let versions = get_package_versions(package).unwrap();
        let valid_versions_count = versions.iter().filter(|v| range.contains(&v.version)).count();

        // Invert to pick packages with fewest versions
        return Reverse(valid_versions_count);
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &SemanticVersion,
    ) -> Result<Dependencies<String, SemVS, Self::M>, SolverError> {
        // If the package name is "." this is our root package so its dependencies are as given
        if package.eq(".") {
            let deps: HashMap<String, Ranges<SemanticVersion>, rustc_hash::FxBuildHasher> = self
                .root_packages
                .iter()
                .map(|p| (p.name.clone(), SemVS::singleton(p.version)))
                .collect();

            return Ok(Dependencies::Available(deps));
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
    pub fn resolve(packages: Vec<PackageIdentifier>) -> anyhow::Result<Vec<PackageIdentifier>> {
        let resolver = BelleDependencyProvider::new(packages);

        // todo what happens to errors here?
        let resolved_dependencies =
            resolve(&resolver, String::from("."), SemanticVersion::zero()).context("Dependency resolution failed")?;

        return Ok(resolved_dependencies
            .into_iter()
            .filter(|(name, _version)| !name.eq("."))
            .map(|(name, version)| PackageIdentifier { name, version })
            .collect());
    }
}
