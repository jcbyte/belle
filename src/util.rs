use pubgrub::SemanticVersion;
use toml::value::Date;

pub fn date_to_version(date: &Date) -> SemanticVersion {
    return SemanticVersion::new(date.year.into(), date.month.into(), date.day.into());
}

pub fn get_isabelle_version(name: &String) -> SemanticVersion {
    // Use each number separated with a dash as its SemVer version:
    // > 2019   -> 2019.0.0
    // > 2025-2 -> 2025.2.0
    let name_parts: Vec<u32> = name.split('-').filter_map(|s| s.parse::<u32>().ok()).collect();

    let major = name_parts.get(0).unwrap_or(&0);
    let minor = name_parts.get(1).unwrap_or(&0);
    let patch = name_parts.get(2).unwrap_or(&0);

    SemanticVersion::new(*major, *minor, *patch)
}

pub fn get_isabelle_name(version: &SemanticVersion) -> String {
    let ver_string = version.to_string();
    let ver_parts: Vec<&str> = ver_string.split('.').collect();

    let name_parts: Vec<&str> = ver_parts.into_iter().filter(|vp| !vp.eq(&"0")).collect();
    return name_parts.join("-");
}
