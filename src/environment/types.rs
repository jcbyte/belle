use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VersionReq {
    Given(SemanticVersion),
    #[serde(rename = "*")]
    Any,
}

impl VersionReq {
    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }
}

impl From<VersionReq> for Option<SemanticVersion> {
    fn from(ver: VersionReq) -> Self {
        match ver {
            VersionReq::Given(v) => Some(v),
            VersionReq::Any => None,
        }
    }
}

impl From<Option<SemanticVersion>> for VersionReq {
    fn from(opt: Option<SemanticVersion>) -> Self {
        match opt {
            Some(v) => Self::Given(v),
            None => Self::Any,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Environment {
    pub name: String,
    pub packages: HashMap<String, VersionReq>,
    pub isabelle: VersionReq,
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
