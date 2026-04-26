use super::*;

mod parse_package_json_dependencies {
    use super::*;

    #[test]
    fn returns_dependencies_from_supported_sections() {
        let content = r#"{
            "dependencies": {
                "prod-pkg": "^1.0.0"
            },
            "devDependencies": {
                "dev-pkg": "~2.0.0"
            },
            "optionalDependencies": {
                "optional-pkg": "3.0.0"
            },
            "peerDependencies": {
                "peer-pkg": ">=4.0.0"
            }
        }"#;

        let dependencies = parse_package_json_dependencies(content).unwrap();

        assert_eq!(
            dependencies,
            vec![
                ManifestDependency {
                    name: "prod-pkg".to_string(),
                    dependency_type: "dependencies".to_string(),
                    requested_as: "^1.0.0".to_string(),
                },
                ManifestDependency {
                    name: "dev-pkg".to_string(),
                    dependency_type: "devDependencies".to_string(),
                    requested_as: "~2.0.0".to_string(),
                },
                ManifestDependency {
                    name: "optional-pkg".to_string(),
                    dependency_type: "optionalDependencies".to_string(),
                    requested_as: "3.0.0".to_string(),
                },
                ManifestDependency {
                    name: "peer-pkg".to_string(),
                    dependency_type: "peerDependencies".to_string(),
                    requested_as: ">=4.0.0".to_string(),
                },
            ]
        );
    }

    #[test]
    fn returns_empty_when_no_supported_sections_exist() {
        let dependencies = parse_package_json_dependencies(r#"{"name":"app"}"#).unwrap();

        assert!(dependencies.is_empty());
    }

    #[test]
    fn returns_error_for_invalid_json() {
        let result = parse_package_json_dependencies("not json");

        assert!(result.is_err());
    }
}
