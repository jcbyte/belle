use std::path::PathBuf;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Isabelle {
    pub version: SemanticVersion,
    pub path: PathBuf,
}
