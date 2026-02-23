use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct ConfigData {
    pub home: PathBuf,
    #[serde(rename = "afp-group")]
    pub afp_group: String,
}

impl Default for ConfigData {
    /// Defaults for config
    fn default() -> Self {
        // Default root directory under the user's application data
        let data_dir = dirs::data_dir().expect("Could not get users data folder");
        let home_dir = data_dir.join("belle");

        return Self {
            home: home_dir,
            afp_group: String::from("isa-afp"),
        };
    }
}
