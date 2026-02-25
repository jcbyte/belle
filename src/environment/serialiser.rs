use std::{collections::HashMap, str::FromStr};

use pubgrub::SemanticVersion;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialise_optional_version<S>(map: &HashMap<String, Option<SemanticVersion>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert the Map into a temporary one where None is "*"
    let transformed: HashMap<&String, String> = map
        .iter()
        .map(|(k, v)| (k, v.map(|v| v.to_string()).unwrap_or_else(|| String::from("*"))))
        .collect();

    return transformed.serialize(s);
}

pub fn deserialise_optional_version<'de, D>(d: D) -> Result<HashMap<String, Option<SemanticVersion>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_map: HashMap<String, String> = HashMap::deserialize(d)?;

    let parsed_map = raw_map
        .into_iter()
        .map(|(name, version)| {
            let parsed_version = if version.eq("*") {
                None
            } else {
                let v = SemanticVersion::from_str(&version).map_err(<D::Error as serde::de::Error>::custom)?;
                Some(v)
            };
            Ok((name, parsed_version))
        })
        .collect::<Result<HashMap<String, Option<SemanticVersion>>, D::Error>>()?;

    return Ok(parsed_map);
}
