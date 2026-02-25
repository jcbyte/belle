use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Environment {
    pub(super) name: String,
    pub(super) packages: HashMap<String, Option<SemanticVersion>>,
    pub(super) lock: HashMap<String, SemanticVersion>,
}

pub struct PackageListing {
    pub name: String,
    pub version: SemanticVersion,
    pub given_version: bool,
    pub transitive: bool,
}
