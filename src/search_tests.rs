use super::*;
use crate::lockfile::{LockFileType, parse_dependency_entries};
use std::fs;

fn load_fixture(name: &str) -> String {
    let path = format!("testdata/{}", name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e))
}

fn load_graph(name: &str) -> DependencyGraph {
    let content = load_fixture(name);
    parse_dependency_entries(&LockFileType::Yarn, &content).unwrap()
}

mod package_exists_tests {
    use super::*;

    #[test]
    fn returns_true_when_target_package_exists() {
        let graph = load_graph("single_package.lock");
        assert!(package_exists(&graph, "pkg-a", "1.0.0"));
    }

    #[test]
    fn returns_false_when_target_package_is_missing() {
        let graph = load_graph("single_package.lock");
        assert!(!package_exists(&graph, "pkg-a", "2.0.0"));
        assert!(!package_exists(&graph, "pkg-z", "1.0.0"));
    }
}

mod find_dependency_chains_tests {
    use super::*;

    #[test]
    fn returns_empty_when_target_not_found() {
        let graph = load_graph("single_package.lock");
        let chains = find_dependency_chains(&graph, "pkg-z", "1.0.0");
        assert!(chains.is_empty());
    }

    #[test]
    fn returns_empty_chain_for_direct_dependency() {
        let graph = load_graph("single_package.lock");

        let chains = find_dependency_chains(&graph, "pkg-a", "1.0.0");

        assert_eq!(chains.len(), 1);
        assert!(chains[0].links.is_empty());
        assert_eq!(chains[0].target_locator, "pkg-a@^1.0.0");
    }

    #[test]
    fn returns_single_parent_chain() {
        let graph = load_graph("simple_chain.lock");
        let chains = find_dependency_chains(&graph, "pkg-b", "2.0.0");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].links.len(), 1);
        assert_eq!(chains[0].links[0].name, "pkg-a");
        assert_eq!(chains[0].links[0].version, "1.0.0");
        assert_eq!(chains[0].links[0].requested_as, "^2.0.0");
    }

    #[test]
    fn returns_deep_chain() {
        let graph = load_graph("deep_chain.lock");
        let chains = find_dependency_chains(&graph, "pkg-c", "3.0.0");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].links.len(), 2);
        assert_eq!(chains[0].links[0].name, "pkg-b");
        assert_eq!(chains[0].links[0].version, "2.0.0");
        assert_eq!(chains[0].links[0].requested_as, "^3.0.0");
        assert_eq!(chains[0].links[1].name, "pkg-a");
        assert_eq!(chains[0].links[1].version, "1.0.0");
        assert_eq!(chains[0].links[1].requested_as, "^2.0.0");
    }

    #[test]
    fn returns_multiple_chains_for_diamond_graph() {
        let graph = load_graph("diamond_chain.lock");
        let chains = find_dependency_chains(&graph, "pkg-c", "3.0.0");
        assert_eq!(chains.len(), 2);
        assert!(chains.iter().all(|c| c.links.len() == 1));
        let parent_names: Vec<&str> = chains.iter().map(|c| c.links[0].name.as_str()).collect();
        assert!(parent_names.contains(&"pkg-a"));
        assert!(parent_names.contains(&"pkg-b"));
    }

    #[test]
    fn uses_package_lock_paths_to_distinguish_duplicate_package_versions() {
        let content = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "dependencies": {
                        "parent-a": "^1.0.0",
                        "parent-b": "^1.0.0"
                    }
                },
                "node_modules/parent-a": {
                    "version": "1.0.0",
                    "dependencies": {
                        "pkg-c": "^1.0.0"
                    }
                },
                "node_modules/parent-b": {
                    "version": "1.0.0",
                    "dependencies": {
                        "pkg-c": "^2.0.0"
                    }
                },
                "node_modules/pkg-c": {
                    "version": "1.0.0"
                },
                "node_modules/parent-b/node_modules/pkg-c": {
                    "version": "2.0.0"
                }
            }
        }"#;
        let graph = parse_dependency_entries(&LockFileType::Npm, content).unwrap();

        let chains = find_dependency_chains(&graph, "pkg-c", "2.0.0");

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].links.len(), 1);
        assert_eq!(chains[0].links[0].name, "parent-b");
        assert_eq!(chains[0].links[0].requested_as, "^2.0.0");
    }

    #[test]
    fn returns_each_identical_target_instance_once_by_locator() {
        let content = r#"{
            "lockfileVersion": 3,
            "packages": {
                "": { "dependencies": { "parent-a": "1.0.0", "parent-b": "1.0.0" } },
                "node_modules/parent-a": {
                    "version": "1.0.0",
                    "dependencies": { "target": "1.0.0" }
                },
                "node_modules/parent-a/node_modules/target": { "version": "1.0.0" },
                "node_modules/parent-b": {
                    "version": "1.0.0",
                    "dependencies": { "target": "1.0.0" }
                },
                "node_modules/parent-b/node_modules/target": { "version": "1.0.0" }
            }
        }"#;
        let graph = parse_dependency_entries(&LockFileType::Npm, content).unwrap();

        let chains = find_dependency_chains(&graph, "target", "1.0.0");

        assert_eq!(chains.len(), 2);
        assert_eq!(
            chains[0].target_locator,
            "node_modules/parent-a/node_modules/target"
        );
        assert_eq!(
            chains[1].target_locator,
            "node_modules/parent-b/node_modules/target"
        );
    }

    #[test]
    fn deduplicates_hoisted_edges_without_losing_real_parents() {
        let content = r#"{
            "lockfileVersion": 3,
            "packages": {
                "": { "dependencies": { "parent-a": "1.0.0", "parent-b": "1.0.0" } },
                "node_modules/parent-a": {
                    "version": "1.0.0",
                    "dependencies": { "target": "^1.0.0" }
                },
                "node_modules/parent-b": {
                    "version": "1.0.0",
                    "dependencies": { "target": "^1.0.0" }
                },
                "node_modules/target": { "version": "1.0.0" }
            }
        }"#;
        let graph = parse_dependency_entries(&LockFileType::Npm, content).unwrap();

        let chains = find_dependency_chains(&graph, "target", "1.0.0");

        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].links[0].name, "parent-a");
        assert_eq!(chains[1].links[0].name, "parent-b");
    }

    #[test]
    fn stops_cycles_and_reports_a_warning() {
        let graph = DependencyGraph {
            nodes: vec![
                DependencyNode {
                    name: "a".to_string(),
                    version: "1.0.0".to_string(),
                    locator: "a@1.0.0".to_string(),
                    dependencies: vec![DependencyEdge {
                        target: 1,
                        requested_as: "1.0.0".to_string(),
                        kind: DependencyKind::Normal,
                    }],
                },
                DependencyNode {
                    name: "b".to_string(),
                    version: "1.0.0".to_string(),
                    locator: "b@1.0.0".to_string(),
                    dependencies: vec![DependencyEdge {
                        target: 0,
                        requested_as: "1.0.0".to_string(),
                        kind: DependencyKind::Normal,
                    }],
                },
            ],
        };

        let chains = find_dependency_chains(&graph, "a", "1.0.0");

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].links.len(), 1);
        assert!(chains[0].warnings[0].contains("cycle"));
    }
}
