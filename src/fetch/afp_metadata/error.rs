use hinted::Hint;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RootParserError {
    #[error("{name} could not be parsed from package ROOT file")]
    CouldNotParse { name: String },
}

pub trait RootParserContext<T> {
    fn report_failed_parsing(self, name: impl Into<String>) -> Result<T, RootParserError>;
}

impl<T> RootParserContext<T> for Option<T> {
    fn report_failed_parsing(self, name: impl Into<String>) -> Result<T, RootParserError> {
        self.ok_or_else(|| RootParserError::CouldNotParse { name: name.into() })
    }
}

#[derive(Error, Debug, Hint)]
pub enum AfpMetadataError {
    #[error("package '{package}' does not exist in afp metadata")]
    NoPackage { package: String },

    #[error("package '{package}' depends on '{dependency}' which cannot be found")]
    #[hint("'{dependency}' may be an alias that was not resolved previously")]
    DependencyMissing { package: String, dependency: String },

    #[error("missing {name} for package '{package}' within the afp metadata")]
    DataMissing { name: String, package: String },
}
