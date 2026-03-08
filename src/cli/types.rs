use std::{convert::Infallible, ops::Deref, str::FromStr};

use pubgrub::SemanticVersion;

use crate::util::get_isabelle_version;

#[derive(Clone)]
pub struct IsabelleVersion(pub SemanticVersion);

impl FromStr for IsabelleVersion {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try standard SemVer conversion
        if let Ok(version) = s.parse::<SemanticVersion>() {
            return Ok(IsabelleVersion(version));
        }

        // If that cannot be parsed assume version is written as name, so parse that way
        return Ok(IsabelleVersion(get_isabelle_version(s)));
    }
}

// For converting references (&IsabelleVersion -> &SemanticVersion)
impl Deref for IsabelleVersion {
    type Target = SemanticVersion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// For converting owned (IsabelleVersion -> SemanticVersion)
impl From<IsabelleVersion> for SemanticVersion {
    fn from(v: IsabelleVersion) -> Self {
        v.0
    }
}
