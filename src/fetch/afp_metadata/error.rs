use thiserror::Error;

#[derive(Error, Debug)]
pub enum RootParserError {
    #[error("The {name} could not be parsed from ROOT file.")]
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

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Package {package} depends on {dependency} which does not seem to exist.")]
    DependencyMissing { package: String, dependency: String },

    #[error("Package {package} does not exist within the metadata")]
    NoPackage { package: String },

    #[error("{name} for package {package} does not exist within the metadata.")]
    MissingData { name: String, package: String },
}
