use serde::{Deserialize, Serialize};

use crate::registry::PackageIdentifier;

#[derive(Serialize, Deserialize)]
pub struct Environment {
    pub(super) name: String,
    pub(super) packages: Vec<PackageIdentifier>,
}

#[derive(Serialize, Deserialize)]
pub struct EnvManager {
    pub(super) name: String,
}
