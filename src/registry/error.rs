use thiserror::Error;

use crate::registry::PackageIdentifier;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Package '{package_id}' does not exist")]
    NotExist { package_id: PackageIdentifier },

    #[error("No source defined for package '{package_id}'.")]
    NoSource { package_id: PackageIdentifier },

    #[error("No versions found for package '{package}'.")]
    NoVersions { package: String },
}

pub trait RegistryNotExistContext<T> {
    fn report_package_nonexistent(self, package_id: impl Into<PackageIdentifier>) -> Result<T, RegistryError>;
    fn report_no_package_versions(self, package: impl Into<String>) -> Result<T, RegistryError>;
}

impl<T> RegistryNotExistContext<T> for Option<T> {
    fn report_package_nonexistent(self, package_id: impl Into<PackageIdentifier>) -> Result<T, RegistryError> {
        self.ok_or_else(|| RegistryError::NotExist {
            package_id: package_id.into(),
        })
    }

    fn report_no_package_versions(self, package: impl Into<String>) -> Result<T, RegistryError> {
        self.ok_or_else(|| RegistryError::NoVersions {
            package: package.into(),
        })
    }
}
