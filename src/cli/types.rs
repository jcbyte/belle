use std::{convert::Infallible, ops::Deref, str::FromStr};

use pubgrub::{SemanticVersion, VersionParseError};

use crate::util::get_isabelle_version;

#[derive(Clone)]
pub struct IsabelleVersion(pub SemanticVersion);

impl FromStr for IsabelleVersion {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try standard SemVer conversion
        if let Ok(version) = s.parse::<PackageVersion>() {
            return Ok(IsabelleVersion(version.into()));
        }

        // If that cannot be parsed assume version is written as name, so parse that way
        Ok(IsabelleVersion(get_isabelle_version(s)))
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

#[derive(Clone)]
pub struct PackageVersion(pub SemanticVersion);

impl FromStr for PackageVersion {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stripped_string = s.strip_prefix("v").unwrap_or(s);

        // Try standard SemVer conversion
        stripped_string.parse::<SemanticVersion>().map(PackageVersion)
    }
}

// For converting references (&PackageVersion -> &SemanticVersion)
impl Deref for PackageVersion {
    type Target = SemanticVersion;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// For converting owned (PackageVersion -> SemanticVersion)
impl From<PackageVersion> for SemanticVersion {
    fn from(v: PackageVersion) -> Self {
        v.0
    }
}
