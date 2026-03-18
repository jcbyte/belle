use pubgrub::{DefaultStringReporter, PubGrubError, Reporter};
use thiserror::Error;

use crate::{error::AppError, registry::PackageIdentifier, resolver::BelleDependencyProvider};

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("No solution could be generated:\n{report}")]
    Conflict { report: String },

    #[error("Failed to retrieve dependencies for {package}")]
    DependencyRetrieval {
        package: PackageIdentifier,
        #[source]
        source: Box<AppError>,
    },

    #[error("Failed to choose a version for {package}")]
    VersionSelectionFailed {
        package: String,
        #[source]
        source: Box<AppError>,
    },

    #[error("Resolution was cancelled.")]
    Cancelled,
}

pub trait ResolverContext<T> {
    fn report_no_solution(self) -> Result<T, ResolverError>;
}

impl<T> ResolverContext<T> for Result<T, PubGrubError<BelleDependencyProvider>> {
    fn report_no_solution(self) -> Result<T, ResolverError> {
        self.map_err(|e| match e {
            PubGrubError::NoSolution(derivation_tree) => ResolverError::Conflict {
                // todo test this, can it be printed nicer
                report: DefaultStringReporter::report(&derivation_tree),
            },
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
