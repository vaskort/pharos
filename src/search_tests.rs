use super::*;
use std::fs;
use yarn_lock_parser::parse_str;

fn load_fixture(name: &str) -> String {
    let path = format!("testdata/{}", name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e))
}

mod package_exists_tests {
    use super::*;

    #[test]
    fn returns_true_when_target_package_exists() {
        let content = load_fixture("single_package.lock");
        let lockfile = parse_str(&content).unwrap();
        assert!(package_exists(&lockfile.entries, "pkg-a", "1.0.0"));
    }

    #[test]
    fn returns_false_when_target_package_is_missing() {
        let content = load_fixture("single_package.lock");
        let lockfile = parse_str(&content).unwrap();
        assert!(!package_exists(&lockfile.entries, "pkg-a", "2.0.0"));
        assert!(!package_exists(&lockfile.entries, "pkg-z", "1.0.0"));
    }
}

mod find_dependency_chains_tests {
    use super::*;

    #[test]
    fn returns_empty_when_target_not_found() {
        let content = load_fixture("single_package.lock");
        let lockfile = parse_str(&content).unwrap();
        let chains = find_dependency_chains(&lockfile.entries, "pkg-z", "1.0.0");
        assert!(chains.is_empty());
    }

    #[test]
    fn returns_empty_chain_for_direct_dependency() {
        let content = load_fixture("single_package.lock");
        let lockfile = parse_str(&content).unwrap();

        let chains = find_dependency_chains(&lockfile.entries, "pkg-a", "1.0.0");

        assert_eq!(chains.len(), 1);
        assert!(chains[0].is_empty());
    }

    #[test]
    fn returns_single_parent_chain() {
        let content = load_fixture("simple_chain.lock");
        let lockfile = parse_str(&content).unwrap();
        let chains = find_dependency_chains(&lockfile.entries, "pkg-b", "2.0.0");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].len(), 1);
        assert_eq!(chains[0][0].name, "pkg-a");
        assert_eq!(chains[0][0].version, "1.0.0");
        assert_eq!(chains[0][0].requested_as, "^2.0.0");
    }

    #[test]
    fn returns_deep_chain() {
        let content = load_fixture("deep_chain.lock");
        let lockfile = parse_str(&content).unwrap();
        let chains = find_dependency_chains(&lockfile.entries, "pkg-c", "3.0.0");
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
        let content = load_fixture("diamond_chain.lock");
        let lockfile = parse_str(&content).unwrap();
        let chains = find_dependency_chains(&lockfile.entries, "pkg-c", "3.0.0");
        assert_eq!(chains.len(), 2);
        assert!(chains.iter().all(|c| c.len() == 1));
        let parent_names: Vec<&str> = chains.iter().map(|c| c[0].name.as_str()).collect();
        assert!(parent_names.contains(&"pkg-a"));
        assert!(parent_names.contains(&"pkg-b"));
    }
}
