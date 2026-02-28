use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct ConfigData {
    pub home: PathBuf,
    #[serde(rename = "afp-group")]
    pub afp_group: String,
    #[serde(rename = "isabelle-packages")]
    pub isabelle_packages: Vec<String>,
}

// todo make isabelles all depend on isa_version which ensures a valid version across all isabelle packages

impl Default for ConfigData {
    /// Defaults for config
    fn default() -> Self {
        // Default root directory under the user's application data
        let data_dir = dirs::data_dir().expect("Could not get users data folder");
        let home_dir = data_dir.join("belle");

        return Self {
            home: home_dir,
            afp_group: String::from("isa-afp"),
            isabelle_packages: vec![
                String::from("HOL-Real_Asymp"),
                String::from("HOL-Eisbach"),
                String::from("HOL-Analysis"),
                String::from("HOL-Cardinals"),
            ],
        };
    }
}
