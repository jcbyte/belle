use std::fmt::Write;
use std::str::FromStr;

use crate::resolver::ISABELLE_PACKAGE;
use crate::util::get_isabelle_name;
use crate::{config::BelleConfig, resolver::core::SemVS};
use pubgrub::{DerivationTree, External, Reporter, SemanticVersion};
use regex::{Captures, Regex};

const LINE_SPLIT: &str = "\n » ";

#[derive(PartialEq)]
enum PackageType {
    Environment,
    Isabelle,
    External,
}

pub struct BelleReporter;

impl BelleReporter {
    fn package_type(name: &str) -> PackageType {
        if name == "." {
            return PackageType::Environment;
        }

        if name == ISABELLE_PACKAGE || BelleConfig::read_config(|c| c.isabelle_packages.iter().any(|n| n == name)) {
            return PackageType::Isabelle;
        }

        PackageType::External
    }

    fn format_pkg_name<'a>(pkg_name: &'a str, pkg_type: &PackageType) -> &'a str {
        match pkg_type {
            PackageType::Isabelle => "Isabelle",
            PackageType::External => pkg_name,
            PackageType::Environment => unimplemented!("Handle `PackageType::Environment` before reaching this call"),
        }
    }

    fn format_pkg_range(range: &SemVS, pkg_type: &PackageType) -> String {
        let range_str = range.to_string();

        // Regex for extracting SemVer versions
        let re = Regex::new(r"(\d+\.\d+\.\d+)").expect("Invalid hardcoded regex expression");

        let result = re.replace_all(&range_str, |caps: &Captures| {
            let raw_version = &caps[0];

            // Format version numbers to expected
            match pkg_type {
                PackageType::External => format!("v{raw_version}"),
                PackageType::Isabelle => {
                    if let Ok(version) = SemanticVersion::from_str(raw_version) {
                        get_isabelle_name(&version)
                    } else {
                        // If version didn't parse correctly, keep the original text
                        raw_version.to_string()
                    }
                }
                PackageType::Environment => {
                    unimplemented!(
                        "Handle `PackageType::Environment` before reaching this call, it should not have a range"
                    )
                }
            }
        });

        result.to_string()
    }

    fn report_external(external: &External<String, SemVS, String>) -> Option<String> {
        Some(match external {
            // As our root is always static "." v0.0.0, this should never occur
            External::NotRoot(..) => unreachable!(),
            // NoVersions errors should be collapsed
            External::NoVersions(..) => {
                unimplemented!("Derivation tree must have collapsed no versions though `collapse_no_versions`")
            }

            // Format standard dependency error
            External::FromDependencyOf(pkg, pkg_range, dep, dep_range) => {
                let pkg_type = Self::package_type(pkg);
                let dep_type = Self::package_type(dep);

                // If the first half is an isabelle package, ignore it as
                // Isabelle packages only depend on isabelle packages
                // meaning Isabelle depends on Isabelle which is trivial,
                if matches!(pkg_type, PackageType::Isabelle) {
                    return None;
                }

                let pkg_info = match pkg_type {
                    PackageType::Environment => "Your environment depends on".to_string(),
                    PackageType::External if *pkg_range == SemVS::full() => format!("All versions of {pkg} depend on"),
                    PackageType::External => {
                        let range = Self::format_pkg_range(pkg_range, &PackageType::External);
                        format!("{pkg} {range} depends on")
                    }
                    PackageType::Isabelle => unreachable!(),
                };

                // As the environment is the root package, this can never be a dependency
                let dep_name = Self::format_pkg_name(dep, &dep_type);
                let dep_info = if *dep_range == SemVS::full() {
                    format!("any version of {dep_name}")
                } else {
                    let range = Self::format_pkg_range(dep_range, &dep_type);
                    format!("{dep_name} {range}")
                };

                format!("{pkg_info} {dep_info}")
            }

            // Print custom errors directly
            External::Custom(pkg, range, reason) => {
                let pkg_type = Self::package_type(pkg);

                match pkg_type {
                    PackageType::Environment => format!("Your environment is unavailable because: {reason}"),
                    PackageType::Isabelle | PackageType::External => {
                        let name = Self::format_pkg_name(pkg, &pkg_type);

                        if *range == SemVS::full() {
                            format!("All versions of {name} are unavailable because: {reason}")
                        } else {
                            let range_str = Self::format_pkg_range(range, &pkg_type);
                            format!("{name} {range_str} is unavailable because: {reason}")
                        }
                    }
                }
            }
        })
    }

    /// Recursive helper to build chain
    fn report_recursive(tree: &DerivationTree<String, SemVS, String>) -> Option<String> {
        match tree {
            DerivationTree::External(ext) => Self::report_external(ext),
            DerivationTree::Derived(derived) => {
                let mut result = String::new();

                // Only write lines if they contain information
                if let Some(c1) = Self::report_recursive(&derived.cause1) {
                    write!(result, "{c1}").expect("Writing to a String failed");
                }
                if let Some(c2) = Self::report_recursive(&derived.cause2) {
                    write!(result, "{LINE_SPLIT}{c2}").expect("Writing to a String failed");
                }

                Some(result)
            }
        }
    }
}

impl Reporter<String, SemVS, String> for BelleReporter {
    type Output = String;

    fn report(derivation_tree: &DerivationTree<String, SemVS, String>) -> Self::Output {
        format!(
            "Your environment cannot be satisfied as:{LINE_SPLIT}{}",
            Self::report_recursive(derivation_tree).expect("The tree root should be `DerivationTree::Derived`")
        )
    }

    fn report_with_formatter(
        derivation_tree: &DerivationTree<String, SemVS, String>,
        _formatter: &impl pubgrub::ReportFormatter<String, SemVS, String, Output = Self::Output>,
    ) -> Self::Output {
        Self::report(derivation_tree)
    }
}
