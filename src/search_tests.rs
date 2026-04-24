use super::*;
use crate::lockfile::{LockFileType, parse_dependency_entries};
use std::fs;

fn load_fixture(name: &str) -> String {
    let path = format!("testdata/{}", name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e))
}

fn load_entries(name: &str) -> Vec<DependencyEntry> {
    let content = load_fixture(name);
    parse_dependency_entries(&LockFileType::Yarn, &content).unwrap()
}

mod package_exists_tests {
    use super::*;

    #[test]
    fn returns_true_when_target_package_exists() {
        let entries = load_entries("single_package.lock");
        assert!(package_exists(&entries, "pkg-a", "1.0.0"));
    }

    #[test]
    fn returns_false_when_target_package_is_missing() {
        let entries = load_entries("single_package.lock");
        assert!(!package_exists(&entries, "pkg-a", "2.0.0"));
        assert!(!package_exists(&entries, "pkg-z", "1.0.0"));
    }
}

mod find_dependency_chains_tests {
    use super::*;

    #[test]
    fn returns_empty_when_target_not_found() {
        let entries = load_entries("single_package.lock");
        let chains = find_dependency_chains(&entries, "pkg-z", "1.0.0");
        assert!(chains.is_empty());
    }

    #[test]
    fn returns_empty_chain_for_direct_dependency() {
        let entries = load_entries("single_package.lock");

        let chains = find_dependency_chains(&entries, "pkg-a", "1.0.0");

        assert_eq!(chains.len(), 1);
        assert!(chains[0].is_empty());
    }

    #[test]
    fn returns_single_parent_chain() {
        let entries = load_entries("simple_chain.lock");
        let chains = find_dependency_chains(&entries, "pkg-b", "2.0.0");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].len(), 1);
        assert_eq!(chains[0][0].name, "pkg-a");
        assert_eq!(chains[0][0].version, "1.0.0");
        assert_eq!(chains[0][0].requested_as, "^2.0.0");
    }

    #[test]
    fn returns_deep_chain() {
        let entries = load_entries("deep_chain.lock");
        let chains = find_dependency_chains(&entries, "pkg-c", "3.0.0");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].len(), 2);
        assert_eq!(chains[0][0].name, "pkg-b");
        assert_eq!(chains[0][0].version, "2.0.0");
        assert_eq!(chains[0][0].requested_as, "^3.0.0");
        assert_eq!(chains[0][1].name, "pkg-a");
        assert_eq!(chains[0][1].version, "1.0.0");
        assert_eq!(chains[0][1].requested_as, "^2.0.0");
    }

    #[test]
    fn returns_multiple_chains_for_diamond_graph() {
        let entries = load_entries("diamond_chain.lock");
        let chains = find_dependency_chains(&entries, "pkg-c", "3.0.0");
        assert_eq!(chains.len(), 2);
        assert!(chains.iter().all(|c| c.len() == 1));
        let parent_names: Vec<&str> = chains.iter().map(|c| c[0].name.as_str()).collect();
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
        let entries = parse_dependency_entries(&LockFileType::Npm, content).unwrap();

        let chains = find_dependency_chains(&entries, "pkg-c", "2.0.0");

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].len(), 1);
        assert_eq!(chains[0][0].name, "parent-b");
        assert_eq!(chains[0][0].requested_as, "^2.0.0");
    }
}
