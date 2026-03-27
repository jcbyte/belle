use std::{collections::HashMap, path::PathBuf};

use pubgrub::SemanticVersion;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct ConfigData {
    #[serde(rename = "afp-group")]
    pub afp_group: String,
    pub isabelles: HashMap<SemanticVersion, PathBuf>,
    #[serde(rename = "isabelle-packages")]
    pub isabelle_packages: Vec<String>,
}

impl Default for ConfigData {
    /// Defaults for config
    fn default() -> Self {
        Self {
            afp_group: "isa-afp".to_string(),
            isabelles: HashMap::new(),
            // This default list has been created from analysing AFP entries, it may be incomplete
            isabelle_packages: vec![
                "Pure".to_string(),
                "HOL".to_string(),
                "ZF".to_string(),
                "HOLCF".to_string(),
                "HOL-Library".to_string(),
                "HOL-Analysis".to_string(),
                "HOL-Probability".to_string(),
                "HOL-Computational_Algebra".to_string(),
                "HOL-Number_Theory".to_string(),
                "HOL-Complex_Analysis".to_string(),
                "HOL-Combinatorics".to_string(),
                "HOL-Cardinals".to_string(),
                "HOL-Eisbach".to_string(),
                "HOL-Imperative_HOL".to_string(),
                "HOL-Statespace".to_string(),
                "HOL-Types_To_Sets".to_string(),
                "HOL-Nominal".to_string(),
                "HOL-ex".to_string(),
                "Pure-ex".to_string(),
                "HOL-Examples".to_string(),
                "Prog_Prove".to_string(),
                "Isar_Ref".to_string(),
                "HOL-Proofs-Lambda".to_string(),
                "HOL-Real_Asymp".to_string(),
                "HOL-Nonstandard_Analysis".to_string(),
                "HOL-ODE-Numerics".to_string(),
                "ZF-Constructible".to_string(),
                "HOL-ZF".to_string(),
                "HOLCF-Library".to_string(),
                "HOL-IMP".to_string(),
                "HOL-Hoare".to_string(),
                "HOL-Hoare_Parallel".to_string(),
                "HOL-Algebra".to_string(),
                "HOL-Data_Structures".to_string(),
                "HOL-Decision_Procs".to_string(),
                "HOL-Lattice".to_string(),
                "HOL-SPARK-Examples".to_string(),
            ],
        }
    }
}
