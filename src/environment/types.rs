use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(PartialEq, Debug)]
pub enum PackageType {
    Transitive,
    ExplicitDirect,
    ImplicitDirect,
}

pub struct PackageListing {
    pub name: String,
    pub version: SemanticVersion,
    pub kind: PackageType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pubgrub::SemanticVersion;

    #[test]
    fn test_version_req_is_any() {
        let any = VersionReq::Any;
        assert!(any.is_any());

        let given = VersionReq::Given(SemanticVersion::new(1, 0, 0));
        assert!(!given.is_any());
    }

    #[test]
    fn test_conversions_with_version() {
        let version = SemanticVersion::new(1, 2, 3);
        let req = VersionReq::Given(version.clone());

        // Test From<VersionReq> for Option<SemanticVersion>
        let opt: Option<SemanticVersion> = req.into();
        assert_eq!(opt, Some(version.clone()));

        // Test From<Option<SemanticVersion>> for VersionReq
        let back_to_req: VersionReq = Some(version).into();
        assert_eq!(back_to_req, VersionReq::Given(SemanticVersion::new(1, 2, 3)));
    }

    #[test]
    fn test_conversions_with_any() {
        let req = VersionReq::Any;

        // Test From<VersionReq> for Option<SemanticVersion>
        let opt: Option<SemanticVersion> = req.into();
        assert!(opt.is_none());

        // Test From<Option<SemanticVersion>> for VersionReq
        let back_to_req: VersionReq = None.into();
        assert_eq!(back_to_req, VersionReq::Any);
    }
}
