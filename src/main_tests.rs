use super::*;

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

mod manifest_dependencies_for_lockfile_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn returns_empty_when_package_json_is_missing() {
        let dir = tempdir().unwrap();
        let lockfile_path = dir.path().join("yarn.lock");
        fs::write(&lockfile_path, "").unwrap();

        let (dependencies, warning) = manifest_dependencies_for_lockfile(&lockfile_path);

        assert!(dependencies.is_empty());
        assert!(warning.is_none());
    }

    #[test]
    fn reads_sibling_package_json_dependencies() {
        let dir = tempdir().unwrap();
        let lockfile_path = dir.path().join("yarn.lock");
        fs::write(&lockfile_path, "").unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{
                "dependencies": {
                    "pkg-a": "^1.0.0"
                }
            }"#,
        )
        .unwrap();

        let (dependencies, warning) = manifest_dependencies_for_lockfile(&lockfile_path);

        assert!(warning.is_none());
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "pkg-a");
        assert_eq!(dependencies[0].dependency_type, "dependencies");
        assert_eq!(dependencies[0].requested_as, "^1.0.0");
    }

    #[test]
    fn returns_warning_for_invalid_package_json() {
        let dir = tempdir().unwrap();
        let lockfile_path = dir.path().join("yarn.lock");
        fs::write(&lockfile_path, "").unwrap();
        fs::write(dir.path().join("package.json"), "not json").unwrap();

        let (dependencies, warning) = manifest_dependencies_for_lockfile(&lockfile_path);

        assert!(dependencies.is_empty());
        assert!(warning.unwrap().contains("Failed to parse package.json"));
    }
}

mod group_chains_by_owner_tests {
    use super::*;

    fn owner(name: &str, dependency_type: &str, requested_as: &str) -> DependencyOwner {
        DependencyOwner {
            name: name.to_string(),
            dependency_type: dependency_type.to_string(),
            requested_as: requested_as.to_string(),
        }
    }

    fn chain_report(owner: Option<DependencyOwner>) -> ChainReport {
        ChainReport {
            target_locator: "pkg@1.0.0".to_string(),
            links: Vec::new(),
            owner,
            fix_path: Vec::new(),
            recommended: None,
            remediation: RemediationPlan {
                status: RemediationStatus::Unavailable,
                primary_action: None,
                alternatives: Vec::new(),
                fix_path: Vec::new(),
                warnings: Vec::new(),
            },
            warnings: Vec::new(),
        }
    }

    #[test]
    fn groups_chains_with_the_same_owner() {
        let webpack_dev_server = owner("webpack-dev-server", "devDependencies", "^4.15.2");
        let express = owner("express", "dependencies", "^4.22.1");
        let chains = vec![
            chain_report(Some(webpack_dev_server.clone())),
            chain_report(Some(express.clone())),
            chain_report(Some(webpack_dev_server.clone())),
            chain_report(Some(express.clone())),
            chain_report(None),
        ];

        let groups = group_chains_by_owner(&chains);

        assert_eq!(
            groups,
            vec![
                ChainOwnerGroup {
                    owner: Some(webpack_dev_server),
                    chain_indexes: vec![0, 2],
                },
                ChainOwnerGroup {
                    owner: Some(express),
                    chain_indexes: vec![1, 3],
                },
                ChainOwnerGroup {
                    owner: None,
                    chain_indexes: vec![4],
                },
            ]
        );
    }
}
