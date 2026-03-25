use hinted::Hint;
use pubgrub::{PubGrubError, Reporter};
use thiserror::Error;

use crate::{
    error::AppError,
    registry::PackageIdentifier,
    resolver::{BelleDepsProvider, reporter::BelleReporter},
};

#[derive(Error, Debug, Hint)]
pub enum ResolverError {
    #[error("{report}")]
    Conflict { report: String },

    #[error("dependency resolution failed to retrieve dependencies for {package}")]
    #[hint(
        "the package may not be known, and may need to be sourced from an afp with `belle source afp update`, or externally"
    )]
    DependencyRetrieval {
        package: PackageIdentifier,
        #[source]
        source: Box<AppError>,
    },

    #[error("dependency resolution failed to choose a version for {package}")]
    #[hint(
        "the package may not be known, and may need to be sourced from an afp with `belle source afp update`, or externally"
    )]
    VersionSelectionFailed {
        package: String,
        #[source]
        source: Box<AppError>,
    },

    #[error("dependency resolution was cancelled")]
    Cancelled,
}

pub trait ResolverContext<T> {
    fn report_no_solution(self) -> Result<T, ResolverError>;
}

impl<T> ResolverContext<T> for Result<T, PubGrubError<BelleDepsProvider>> {
    fn report_no_solution(self) -> Result<T, ResolverError> {
        self.map_err(|e| match e {
            PubGrubError::NoSolution(mut derivation_tree) => {
                // Collapse no versions as missing dependency errors are already identified
                derivation_tree.collapse_no_versions();
                ResolverError::Conflict {
                    report: BelleReporter::report(&derivation_tree),
                }
            }
            PubGrubError::ErrorRetrievingDependencies {
                package,
                version,
                source,
            } => ResolverError::DependencyRetrieval {
                package: PackageIdentifier::new(package, version),
                source: source.into(),
            },
            PubGrubError::ErrorChoosingVersion { package, source } => ResolverError::VersionSelectionFailed {
                package: package.to_string(),
                source: source.into(),
            },
            PubGrubError::ErrorInShouldCancel(_) => ResolverError::Cancelled,
        })
    }
}
