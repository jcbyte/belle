use std::path::PathBuf;

use pubgrub::SemanticVersion;

#[derive(Debug)]
pub struct Isabelle {
    pub version: SemanticVersion,
    pub path: PathBuf,
}
