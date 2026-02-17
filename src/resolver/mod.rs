use pubgrub::{
    Dependencies, DependencyConstraints, DependencyProvider, PackageResolutionStatistics, Ranges, SemanticVersion,
    resolve,
};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::convert::Infallible;

type SemV = SemanticVersion;
type SemVS = Ranges<SemV>;

struct IsabelleDependencyProvider {
    package_versions: HashMap<String, Vec<SemV>>,
    package_dependencies: HashMap<(String, SemV), HashMap<String, SemVS>>,
}

impl IsabelleDependencyProvider {
    fn new() -> Self {
        IsabelleDependencyProvider {
            // Initially test with random data
            package_versions: HashMap::from([
                (String::from("myapp"), vec![SemV::new(1, 0, 0)]),
                (String::from("serde"), vec![SemV::new(1, 0, 0), SemV::new(1, 1, 0)]),
                (String::from("tokio"), vec![SemV::new(1, 0, 0)]),
            ]),
            package_dependencies: HashMap::from([
                (
                    (String::from("myapp"), SemV::new(1, 0, 0)),
                    HashMap::from([(String::from("serde"), SemVS::higher_than(SemV::new(1, 0, 0)))]),
                ),
                ((String::from("serde"), SemV::new(1, 1, 0)), HashMap::new()),
                ((String::from("serde"), SemV::new(1, 0, 0)), HashMap::new()),
                ((String::from("tokio"), SemV::new(1, 0, 0)), HashMap::new()),
            ]),
        }
    }
}

impl DependencyProvider for IsabelleDependencyProvider {
    fn choose_version(&self, package: &String, range: &SemVS) -> Result<Option<SemV>, Infallible> {
        // Return the highest version of the package that satisfies the range
        println!("1 {}", package);

        // ! fix unsafe behaviour later
        let versions = self.package_versions.get(package).unwrap();
        let top_valid_version = versions.iter().filter(|v| range.contains(v)).max();

        return Ok(top_valid_version.cloned());
    }

    type Priority = usize;
    fn prioritize(
        &self,
        package: &String,
        range: &SemVS,
        _conflicts_counts: &PackageResolutionStatistics,
    ) -> Self::Priority {
        // Prioritize packages with fewer compatible versions
        println!("2 {}", package);

        // ! fix unsafe behaviour later
        let versions = self.package_versions.get(package).unwrap();
        let valid_versions_count = versions.iter().filter(|v| range.contains(v)).count();

        return 1000 - valid_versions_count;
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &SemV,
    ) -> Result<Dependencies<String, SemVS, Self::M>, Infallible> {
        println!("3 {} {}", package, version.to_string());

        let dependencies = self
            .package_dependencies
            .get(&(package.clone(), version.clone()))
            // ! fix unsafe behaviour later
            .unwrap();

        // todo check this later after working out package structure/retrieval
        let owned_map: HashMap<String, Ranges<SemanticVersion>, rustc_hash::FxBuildHasher> =
            dependencies.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        let e = Dependencies::Available(owned_map);

        return Ok(e);
    }

    type Err = Infallible;
    type P = String;
    type V = SemV;
    type VS = SemVS;
    type M = String;
}

fn main() {
    println!("Hello, world");

    let provider = IsabelleDependencyProvider::new();

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
