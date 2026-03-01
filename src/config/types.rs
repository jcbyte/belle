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
                String::from("Pure"),
                String::from("HOL"),
                String::from("ZF"),
                String::from("HOLCF"),
                String::from("HOL-Library"),
                String::from("HOL-Analysis"),
                String::from("HOL-Probability"),
                String::from("HOL-Computational_Algebra"),
                String::from("HOL-Number_Theory"),
                String::from("HOL-Complex_Analysis"),
                String::from("HOL-Combinatorics"),
                String::from("HOL-Cardinals"),
                String::from("HOL-Eisbach"),
                String::from("HOL-Imperative_HOL"),
                String::from("HOL-Statespace"),
                String::from("HOL-Types_To_Sets"),
                String::from("HOL-Nominal"),
                String::from("HOL-ex"),
                String::from("Pure-ex"),
                String::from("HOL-Examples"),
                String::from("Prog_Prove"),
                String::from("Isar_Ref"),
                String::from("HOL-Proofs-Lambda"),
                String::from("HOL-Real_Asymp"),
                String::from("HOL-Nonstandard_Analysis"),
                String::from("HOL-ODE-Numerics"),
                String::from("ZF-Constructible"),
                String::from("HOL-ZF"),
                String::from("HOLCF-Library"),
                String::from("HOL-IMP"),
                String::from("HOL-Hoare"),
                String::from("HOL-Hoare_Parallel"),
                String::from("HOL-Algebra"),
                String::from("HOL-Data_Structures"),
                String::from("HOL-Decision_Procs"),
                String::from("HOL-Lattice"),
                String::from("HOL-SPARK-Examples"),
                // String::from("Sepref_Prereq"),
                // String::from("Sepref_IICF"),
                // String::from("Restriction_Spaces-HOLCF"),
            ],
        };
    }
}

// todo breaking change ROOT files can have multiple sessions meaning a "package" might actually live inside another package and a bunch of these "isabelle packages" may be from another package actually
// todo https://foss.heptapod.net/isa-afp/afp-2025-2/-/blob/branch/default/thys/Security_Protocol_Refinement/ROOT?ref_type=heads
