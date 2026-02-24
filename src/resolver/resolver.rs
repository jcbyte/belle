use anyhow::Context;
use pubgrub::{Dependencies, DependencyProvider, PackageResolutionStatistics, Ranges, SemanticVersion, resolve};
use std::collections::HashMap;

use crate::registry::{PackageIdentifier, get_package_versions};

type SemVS = Ranges<SemanticVersion>;

struct BelleDependencyProvider {}

impl DependencyProvider for BelleDependencyProvider {
    fn choose_version(&self, package: &String, range: &SemVS) -> anyhow::Result<Option<SemanticVersion>> {
        // Return the highest version of the package that satisfies the range
        // ! fix unsafe behaviour later
        let versions = get_package_versions(package).unwrap();
        let top_valid_version = versions.iter().map(|v| v.version).filter(|v| range.contains(&v)).max();

        return Ok(top_valid_version);
    }

    type Priority = usize;
    fn prioritize(
        &self,
        package: &String,
        range: &SemVS,
        _conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        // Prioritize packages with fewer compatible versions

        // ! fix unsafe behaviour later
        let versions = get_package_versions(package).unwrap();
        let valid_versions_count = versions.iter().filter(|v| range.contains(&v.version)).count();

        // todo why reverse
        return 1000 - valid_versions_count;
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &SemanticVersion,
    ) -> anyhow::Result<Dependencies<String, SemVS, Self::M>> {
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
            .map(|(name, version)| (name.clone(), SemVS::singleton(version)))
            .collect();

        return Ok(Dependencies::Available(deps));
    }

    type Err = anyhow::Error;
    type P = String;
    type V = SemanticVersion;
    type VS = SemVS;
    type M = String;
}

fn main() {
    println!("Hello, world");

    let provider = BelleDependencyProvider {};

    // Resolve dependencies
    match resolve(&provider, String::from("myapp"), SemanticVersion::new(1, 0, 0)) {
        Ok(solution) => {
            println!("Resolution successful!");
            for (package, version) in solution.iter() {
                println!("  {} @ {}", package, version);
            }
        }
        Err(e) => println!("Resolution failed: {:?}", e),
    }
}
