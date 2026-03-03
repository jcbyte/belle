use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

use crate::environment::{deserialise_optional_version, serialise_optional_version};

#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
    pub name: String,
    #[serde(
        serialize_with = "serialise_optional_version",
        deserialize_with = "deserialise_optional_version"
    )]
    pub packages: HashMap<String, Option<SemanticVersion>>,
    pub lock: HashMap<String, SemanticVersion>,
}

pub enum PackageType {
    Transitive,
    Direct { given_version: bool },
}

pub struct PackageListing {
    pub name: String,
    pub version: SemanticVersion,
    pub kind: PackageType,
}
