use std::collections::HashMap;

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Environment {
    pub(super) name: String,
    pub(super) packages: HashMap<String, Option<SemanticVersion>>,
    pub(super) lock: HashMap<String, SemanticVersion>,
}
