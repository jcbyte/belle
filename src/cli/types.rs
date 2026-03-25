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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isabelle_version_parsing() {
        // Test as isabelle version prefix
        let v_isabelle = "2025-1".parse::<IsabelleVersion>();
        assert!(v_isabelle.is_ok());
        assert_eq!(v_isabelle.unwrap().to_string(), "2025.1.0");

        // Test regular version
        let v_regular = "2.0.0".parse::<IsabelleVersion>();
        assert!(v_regular.is_ok());
        assert_eq!(v_regular.unwrap().to_string(), "2.0.0");

        // Test invalid
        let invalid = "not-a-version".parse::<IsabelleVersion>();
        // An invalid version will parse to 0.0.0
        assert!(invalid.is_ok());
        assert_eq!(invalid.unwrap().to_string(), "0.0.0");
    }

    #[test]
    fn test_isabelle_version_deref() {
        let iv = IsabelleVersion(SemanticVersion::new(1, 0, 0));
        let v_ref = &iv;
        assert_eq!(v_ref.to_string(), "1.0.0");
    }

    #[test]
    fn test_package_version_parsing() {
        // Test with 'v' prefix
        let v_prefixed = "v1.2.3".parse::<PackageVersion>();
        assert!(v_prefixed.is_ok());
        assert_eq!(v_prefixed.unwrap().to_string(), "1.2.3");

        // Test without prefix
        let no_prefix = "2.0.0".parse::<PackageVersion>();
        assert!(no_prefix.is_ok());
        assert_eq!(no_prefix.unwrap().to_string(), "2.0.0");

        // Test invalid semver
        let invalid = "not-a-version".parse::<PackageVersion>();
        assert!(invalid.is_err());
    }

    #[test]
    fn test_package_version_deref() {
        let pv = PackageVersion(SemanticVersion::new(1, 0, 0));
        let v_ref = &pv;
        assert_eq!(v_ref.to_string(), "1.0.0");
    }
}
