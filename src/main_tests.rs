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

mod report_chain_tests {
    use super::*;
    use crate::manifest::ManifestDependency;

    fn version_info_for(dep: &str, dep_version: &str) -> VersionInfo {
        VersionInfo {
            dependencies: Some(HashMap::from([(dep.to_string(), dep_version.to_string())])),
        }
    }

    fn chain_link(name: &str, version: &str, requested_as: &str) -> ChainLink {
        ChainLink {
            name: name.to_string(),
            version: version.to_string(),
            requested_as: requested_as.to_string(),
        }
    }

    fn manifest_dependency(
        name: &str,
        dependency_type: &str,
        requested_as: &str,
    ) -> ManifestDependency {
        ManifestDependency {
            name: name.to_string(),
            dependency_type: dependency_type.to_string(),
            requested_as: requested_as.to_string(),
        }
    }

    #[test]
    fn records_fix_path_and_recommended_upgrade() {
        let registry_cache = HashMap::from([(
            "parent".to_string(),
            RegistryResponse {
                versions: HashMap::from([
                    ("1.0.0".to_string(), version_info_for("pkg", "1.0.0")),
                    ("1.1.0".to_string(), version_info_for("pkg", "1.0.1")),
                ]),
            },
        )]);
        let chain = vec![chain_link("parent", "1.0.0", "^1.0.0")];
        let manifest_dependencies = Vec::new();

        let report = report_chain(
            &chain,
            "pkg",
            "1.0.0",
            &registry_cache,
            &manifest_dependencies,
        );

        assert_eq!(report.links.len(), 1);
        assert_eq!(report.links[0].name, "parent");
        assert_eq!(report.links[0].version, "1.0.0");
        assert_eq!(report.links[0].requested_as, "^1.0.0");
        assert_eq!(report.fix_path.len(), 1);
        assert_eq!(report.fix_path[0].package, "parent");
        assert_eq!(report.fix_path[0].minimum_version, "1.1.0");
        assert_eq!(report.recommended.as_ref().unwrap().package, "parent");
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn records_warning_when_no_fix_is_available() {
        let registry_cache: RegistryCache = HashMap::new();
        let chain = vec![chain_link("parent", "1.0.0", "^1.0.0")];
        let manifest_dependencies = Vec::new();

        let report = report_chain(
            &chain,
            "pkg",
            "1.0.0",
            &registry_cache,
            &manifest_dependencies,
        );

        assert!(report.fix_path.is_empty());
        assert!(report.recommended.is_none());
        assert_eq!(
            report.warnings,
            vec!["No parent version found that updates pkg beyond 1.0.0"]
        );
    }

    #[test]
    fn records_direct_dependency_owner_from_package_json() {
        let registry_cache: RegistryCache = HashMap::new();
        let chain = Vec::new();
        let manifest_dependencies = vec![manifest_dependency("pkg", "dependencies", "^1.0.0")];

        let report = report_chain(
            &chain,
            "pkg",
            "1.0.0",
            &registry_cache,
            &manifest_dependencies,
        );

        let owner = report.owner.unwrap();
        assert_eq!(owner.name, "pkg");
        assert_eq!(owner.dependency_type, "dependencies");
        assert_eq!(owner.requested_as, "^1.0.0");
    }

    #[test]
    fn records_transitive_chain_owner_from_package_json() {
        let registry_cache: RegistryCache = HashMap::new();
        let chain = vec![
            chain_link("intermediate", "2.0.0", "^1.0.0"),
            chain_link("root", "3.0.0", "^2.0.0"),
        ];
        let manifest_dependencies = vec![manifest_dependency("root", "devDependencies", "^3.0.0")];

        let report = report_chain(
            &chain,
            "pkg",
            "1.0.0",
            &registry_cache,
            &manifest_dependencies,
        );

        let owner = report.owner.unwrap();
        assert_eq!(owner.name, "root");
        assert_eq!(owner.dependency_type, "devDependencies");
        assert_eq!(owner.requested_as, "^3.0.0");
    }

    #[test]
    fn leaves_owner_empty_when_package_json_does_not_declare_chain_root() {
        let registry_cache: RegistryCache = HashMap::new();
        let chain = vec![chain_link("root", "3.0.0", "^2.0.0")];
        let manifest_dependencies = vec![manifest_dependency("other", "dependencies", "^1.0.0")];

        let report = report_chain(
            &chain,
            "pkg",
            "1.0.0",
            &registry_cache,
            &manifest_dependencies,
        );

        assert!(report.owner.is_none());
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
            links: Vec::new(),
            owner,
            fix_path: Vec::new(),
            recommended: None,
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
