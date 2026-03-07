use pubgrub::SemanticVersion;
use toml::value::Date;

/// Convert a toml Date into a SemVer following: year.month.day
pub fn date_to_version(date: &Date) -> SemanticVersion {
    return SemanticVersion::new(date.year.into(), date.month.into(), date.day.into());
}

/// Convert an Isabelle version (e.g. 2025, 2025-1) into a SemVer (e.g. 2025.0.0, 2025.1.0)
pub fn get_isabelle_version(name: &String) -> SemanticVersion {
    // Use each number separated with a dash as its SemVer version:
    // > 2019   -> 2019.0.0
    // > 2025-2 -> 2025.2.0

    // Filter all non-numeric and non-"-" characters
    let sanitized: String = name.chars().filter(|c| c.is_ascii_digit() || *c == '-').collect();

    // Split by each "-" removing any unparsable strings (e.g. empty)
    let name_parts: Vec<u32> = sanitized.split('-').filter_map(|s| s.parse::<u32>().ok()).collect();

    let major = name_parts.get(0).unwrap_or(&0);
    let minor = name_parts.get(1).unwrap_or(&0);
    let patch = name_parts.get(2).unwrap_or(&0);

    SemanticVersion::new(*major, *minor, *patch)
}

/// Convert a SemVer version (e.g. 2025.0.0, 2025.1.0) into an Isabelle version (e.g. 2025, 2025-1)
pub fn get_isabelle_name(version: &SemanticVersion) -> String {
    let ver_string = version.to_string();
    let ver_parts: Vec<&str> = ver_string.split('.').collect();

    let name_parts: Vec<&str> = ver_parts.into_iter().filter(|vp| !vp.eq(&"0")).collect();
    let name = name_parts.join("-");

    // Use a name of "0" if the version was 0.0.0 and no name is generated
    if name.is_empty() {
        return String::from("0");
    }

    return name;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pubgrub::SemanticVersion;
    use toml::value::Date;

    #[test]
    fn test_date_to_version() {
        let date = Date {
            year: 2025,
            month: 3,
            day: 6,
        };
        assert_eq!(date_to_version(&date), SemanticVersion::new(2025, 3, 6));

        let date2 = Date {
            year: 0,
            month: 1,
            day: 0,
        };
        assert_eq!(date_to_version(&date2), SemanticVersion::new(0, 1, 0));
    }

    #[test]
    fn test_get_isabelle_version() {
        assert_eq!(
            get_isabelle_version(&String::from("2019")),
            SemanticVersion::new(2019, 0, 0)
        );
        assert_eq!(
            get_isabelle_version(&String::from("2025-2")),
            SemanticVersion::new(2025, 2, 0)
        );
        assert_eq!(
            get_isabelle_version(&String::from("2026-20-3")),
            SemanticVersion::new(2026, 20, 3)
        );

        assert_eq!(get_isabelle_version(&String::from("")), SemanticVersion::new(0, 0, 0));

        // Test non-numeric fields
        assert_eq!(
            get_isabelle_version(&String::from("afp-4-3")),
            SemanticVersion::new(4, 3, 0)
        );
        assert_eq!(
            get_isabelle_version(&String::from("Isabelle2025-2")),
            SemanticVersion::new(2025, 2, 0)
        );
        assert_eq!(
            get_isabelle_version(&String::from("TextHere10-1-1\n")),
            SemanticVersion::new(10, 1, 1)
        );
        assert_eq!(
            get_isabelle_version(&String::from("TextHere20-2AndBetween2-9\n")),
            SemanticVersion::new(20, 22, 9)
        );
    }

    #[test]
    fn test_get_isabelle_name() {
        assert_eq!(get_isabelle_name(&SemanticVersion::new(10, 0, 0)), "10");
        assert_eq!(get_isabelle_name(&SemanticVersion::new(2025, 3, 0)), "2025-3");
        assert_eq!(get_isabelle_name(&SemanticVersion::new(2020, 1, 2)), "2020-1-2");
        assert_eq!(get_isabelle_name(&SemanticVersion::new(0, 0, 0)), "0");
    }

    #[test]
    fn test_isabelle_round_trip() {
        // Explicit pairs of (isabelle name, expected restored name)
        let cases = vec![
            ("53", "53"),
            ("205-200", "205-200"),
            ("2025-1-3", "2025-1-3"),
            ("0", "0"),
            ("afp-2026-1", "2026-1"),
        ];
        for (name, expected) in cases {
            let ver = get_isabelle_version(&name.to_string());
            let round = get_isabelle_name(&ver);
            assert_eq!(round, expected, "round-trip for {}", name);
        }
    }
}
