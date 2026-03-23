use hinted::Hint;
use thiserror::Error;

use crate::registry::PackageIdentifier;

#[derive(Error, Debug, Hint)]
pub enum RegistryError {
    #[error("package {package_id} does not exist")]
    #[hint("it may need to be sourced from an afp, or externally")]
    NotExist { package_id: PackageIdentifier },

    #[error("package with name '{package}' cannot be found")]
    #[hint("it may need to be sourced from an afp, or externally")]
    NameNotExist { package: String },

    #[error("no source defined for package {package_id}")]
    NoSource { package_id: PackageIdentifier },

    #[error("no ROOT file found for package {package}")]
    #[hint("this may not be an Isabelle session, or has been corrupted")]
    NoRootFile { package: PackageIdentifier },
}

pub trait RegistryNotExistContext<T> {
    fn report_package_nonexistent(self, package_id: impl Into<PackageIdentifier>) -> Result<T, RegistryError>;
    fn report_package_name_nonexistent(self, package: impl Into<String>) -> Result<T, RegistryError>;
}

impl<T> RegistryNotExistContext<T> for Option<T> {
    fn report_package_nonexistent(self, package_id: impl Into<PackageIdentifier>) -> Result<T, RegistryError> {
        self.ok_or_else(|| RegistryError::NotExist {
            package_id: package_id.into(),
        })
    }

    fn report_package_name_nonexistent(self, package: impl Into<String>) -> Result<T, RegistryError> {
        self.ok_or_else(|| RegistryError::NameNotExist {
            package: package.into(),
        })
    }
}
