mod core;
pub mod error;
pub mod manager;
mod types;

pub use types::{Environment, PackageListing, PackageType, VersionReq};

pub static LOCKFILE_NAME: &str = "belle.toml";
