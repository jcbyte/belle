use thiserror::Error;

use crate::registry::PackageIdentifier;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Package '{package_id}' does not exist")]
    NotExist { package_id: PackageIdentifier },

    #[error("Package '{package}' does not exist.")]
    NameNotExist { package: String },

    #[error("No source defined for package '{package_id}'.")]
    NoSource { package_id: PackageIdentifier },

    #[error("No ROOT file found in package '{package}' source.")]
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
