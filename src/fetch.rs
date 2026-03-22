mod afp;
pub mod afp_metadata;
mod client;
pub mod error;
mod local;
mod remote;
mod types;

pub use afp_metadata::RepoMetadata;
pub use client::BelleClient;
pub use types::{AfpRepo, ReturnedPackages};

pub use local::get_local_package_meta;

pub static PACKAGE_FILE: &str = "belle-pkg.toml";
