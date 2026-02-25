use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

use crate::environment::{deserialise_optional_version, serialise_optional_version};

#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
    pub(super) name: String,
    #[serde(
        serialize_with = "serialise_optional_version",
        deserialize_with = "deserialise_optional_version"
    )]
    pub(super) packages: HashMap<String, Option<SemanticVersion>>,
    pub(super) lock: HashMap<String, SemanticVersion>,
}

pub struct PackageListing {
    pub name: String,
    pub version: SemanticVersion,
    pub given_version: bool,
    pub transitive: bool,
}
