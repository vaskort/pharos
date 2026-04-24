use super::*;
use crate::registry::{RegistryResponse, VersionInfo};
use std::collections::HashMap;

mod parse_package_tests {
    use super::*;

    #[test]
    fn parses_simple_package() {
        let expected = Ok(PackageSpec {
            name: "pkg-a",
            version: "1.0.0",
        });

        let result = parse_package("pkg-a@1.0.0");

        assert_eq!(result, expected);
    }

    #[test]
    fn parses_scoped_package() {
        let expected = Ok(PackageSpec {
            name: "@scope/pkg-a",
            version: "1.0.0",
        });

        let result = parse_package("@scope/pkg-a@1.0.0");

        assert_eq!(result, expected);
    }

    #[test]
    fn rejects_missing_version() {
        let expected = Err(ParseError::MissingVersion);

        let result = parse_package("pkg-a");
        assert_eq!(result, expected);

        let expected_invalid = Err(ParseError::InvalidVersion("".to_string()));

        let result = parse_package("pkg-a@");
        assert_eq!(result, expected_invalid);
    }

    #[test]
    fn rejects_non_version_tag() {
        let expected = Err(ParseError::InvalidVersion("latest".to_string()));

        let result = parse_package("pkg-a@latest");

        assert_eq!(result, expected);
    }

    #[test]
    fn rejects_empty_input() {
        let expected = Err(ParseError::Empty);

        let result = parse_package("");
        assert_eq!(result, expected);
    }

    #[test]
    fn supports_v_prefix() {
        let result = parse_package("pkg@v1.2.3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().version, "1.2.3");
    }

    #[test]
    fn trims_surrounding_whitespace() {
        let expected = Ok(PackageSpec {
            name: "pkg-a",
            version: "1.0.0",
        });

        let result = parse_package("  pkg-a@1.0.0  ");

        assert_eq!(result, expected);
    }
}

mod show_parent_updates_prerelease_tests {
    use super::*;

    fn version_info_for(dep: &str, dep_version: &str) -> VersionInfo {
        VersionInfo {
            dependencies: Some(HashMap::from([(dep.to_string(), dep_version.to_string())])),
        }
    }

    #[test]
    fn skips_prerelease_and_uses_stable_version() {
        let mut registry_cache: RegistryCache = HashMap::new();
        let parent_data = RegistryResponse {
            versions: HashMap::from([
                ("1.1.0-beta.1".to_string(), version_info_for("pkg", "2.0.0")),
                ("1.1.0".to_string(), version_info_for("pkg", "2.0.0")),
            ]),
        };
        registry_cache.insert("parent".to_string(), parent_data);

        let result = show_parent_updates(&registry_cache, "pkg", "1.0.0", "parent");
        assert_eq!(result, Some("1.1.0".to_string()));
    }

    #[test]
    fn returns_none_when_only_prerelease_versions_update_dependency() {
        let mut registry_cache: RegistryCache = HashMap::new();
        let parent_data = RegistryResponse {
            versions: HashMap::from([(
                "1.1.0-beta.1".to_string(),
                version_info_for("pkg", "2.0.0"),
            )]),
        };
        registry_cache.insert("parent".to_string(), parent_data);

        let result = show_parent_updates(&registry_cache, "pkg", "1.0.0", "parent");
        assert_eq!(result, None);
    }

    #[test]
    fn picks_smallest_stable_parent_version_that_updates_dependency() {
        let mut registry_cache: RegistryCache = HashMap::new();
        let parent_data = RegistryResponse {
            versions: HashMap::from([
                ("1.0.0".to_string(), version_info_for("pkg", "1.0.0")),
                ("1.1.0".to_string(), version_info_for("pkg", "1.0.1")),
                ("2.0.0".to_string(), version_info_for("pkg", "2.0.0")),
            ]),
        };
        registry_cache.insert("parent".to_string(), parent_data);

        let result = show_parent_updates(&registry_cache, "pkg", "1.0.0", "parent");

        assert_eq!(result, Some("1.1.0".to_string()));
    }

    #[test]
    fn returns_none_when_parent_is_not_cached() {
        let registry_cache: RegistryCache = HashMap::new();

        let result = show_parent_updates(&registry_cache, "pkg", "1.0.0", "parent");

        assert_eq!(result, None);
    }
}
