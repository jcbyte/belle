use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Package '{package}' does not exist")]
    NotExist { package: String },

    #[error("No source defined for package '{package}'.")]
    NoSource { package: String },
}

pub trait RegistryNotExistContext<T> {
    fn report_package_nonexistent(self, package: impl Into<String>) -> Result<T, RegistryError>;
}

impl<T> RegistryNotExistContext<T> for Option<T> {
    fn report_package_nonexistent(self, package: impl Into<String>) -> Result<T, RegistryError> {
        self.ok_or_else(|| RegistryError::NotExist {
            package: package.into(),
        })
    }
}
