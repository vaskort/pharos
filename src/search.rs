use yarn_lock_parser::Entry;

/// A single link in a dependency chain, representing one package
/// in the path from a direct dependency down to the vulnerable package.
///
/// For example, if `pkg-a` depends on `pkg-b` which depends on `pkg-c` (vulnerable),
/// the chain would be: `[ChainLink(pkg-a), ChainLink(pkg-b)]`.
#[derive(Clone, Debug)]
pub struct ChainLink {
    pub name: String,
    pub version: String,
    /// The version range the parent asked for (e.g. "^4.0.0"), not the resolved version.
    pub requested_as: String,
}

/// Checks if a specific package at a specific version exists in the lockfile entries.
///
/// # Arguments
/// * `entries` - The parsed yarn.lock entries
/// * `package_name` - The package name to search for
/// * `package_version` - The exact version to match
pub fn package_exists(entries: &[Entry], package_name: &str, package_version: &str) -> bool {
    for entry in entries.iter() {
        if entry.name == package_name && entry.version == package_version {
            return true;
        }
    }
    false
}

/// Finds all dependency chains that lead to a specific package version.
///
/// Starting from the target package, it walks *up* the dependency tree
/// to find every path from a root dependency down to the target.
/// Each chain is a `Vec<ChainLink>` representing one such path.
///
/// For example, if `pkg-c@1.0.0` is vulnerable and two packages depend on it:
/// - `pkg-a → pkg-b → pkg-c`
/// - `pkg-d → pkg-c`
///
/// This returns two chains: `[[pkg-a, pkg-b], [pkg-d]]`.
///
/// # Arguments
/// * `entries` - The parsed yarn.lock entries
/// * `package_name` - The target package name to trace chains for
/// * `package_version` - The target package version
///
/// # Returns
/// A `Vec<Vec<ChainLink>>` — e.g. `[[pkg-a, pkg-b], [pkg-d]]`
/// where each inner vec is one path leading to the target package.
/// Returns empty if the package is not found.
pub fn find_dependency_chains(
    entries: &[Entry],
    package_name: &str,
    package_version: &str,
) -> Vec<Vec<ChainLink>> {
    let mut chains = Vec::new();
    let initial_chain = Vec::new();
    let target_entry = entries
        .iter()
        .find(|e| e.name == package_name && e.version == package_version);
    let target_descriptors = match target_entry {
        Some(entry) => &entry.descriptors,
        None => return chains,
    };

    helper(entries, target_descriptors, initial_chain, &mut chains);

    fn helper(
        entries: &[Entry],
        descriptors: &Vec<(&str, &str)>,
        current_chain: Vec<ChainLink>,
        chains: &mut Vec<Vec<ChainLink>>,
    ) {
        let mut found_parent = false;
        for entry in entries {
            for (dep_name, dep_version) in &entry.dependencies {
                if (descriptors).contains(&(*dep_name, *dep_version)) {
                    found_parent = true;
                    let mut branch = current_chain.clone();

                    branch.push(ChainLink {
                        name: entry.name.to_string(),
                        version: entry.version.to_string(),
                        requested_as: dep_version.to_string(),
                    });

                    helper(entries, &entry.descriptors, branch, chains);
                }
            }
        }
        if !found_parent {
            chains.push(current_chain);
        }
    }

    chains
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use yarn_lock_parser::parse_str;

    fn load_fixture(name: &str) -> String {
        let path = format!("testdata/{}", name);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path, e))
    }

    #[test]
    fn test_package_exists_found() {
        let content = load_fixture("single_package.lock");
        let lockfile = parse_str(&content).unwrap();
        assert!(package_exists(&lockfile.entries, "pkg-a", "1.0.0"));
    }

    #[test]
    fn test_package_exists_not_found() {
        let content = load_fixture("single_package.lock");
        let lockfile = parse_str(&content).unwrap();
        assert!(!package_exists(&lockfile.entries, "pkg-a", "2.0.0"));
        assert!(!package_exists(&lockfile.entries, "pkg-z", "1.0.0"));
    }

    #[test]
    fn test_find_dependency_chains_not_found() {
        let content = load_fixture("single_package.lock");
        let lockfile = parse_str(&content).unwrap();
        let chains = find_dependency_chains(&lockfile.entries, "pkg-z", "1.0.0");
        assert!(chains.is_empty());
    }

    #[test]
    fn test_find_dependency_chains_with_parent() {
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
    fn test_find_dependency_chains_deep() {
        let content = load_fixture("deep_chain.lock");
        let lockfile = parse_str(&content).unwrap();
        // Trace chains leading to pkg-c@3.0.0
        let chains = find_dependency_chains(&lockfile.entries, "pkg-c", "3.0.0");
        assert_eq!(chains.len(), 1);
        // Chain should be: pkg-b → pkg-a (both are parents above pkg-c)
        assert_eq!(chains[0].len(), 2);
        assert_eq!(chains[0][0].name, "pkg-b");
        assert_eq!(chains[0][1].name, "pkg-a");
    }

    #[test]
    fn test_find_dependency_chains_diamond() {
        let content = load_fixture("diamond_chain.lock");
        let lockfile = parse_str(&content).unwrap();
        // Both pkg-a and pkg-b depend on pkg-c, so there should be 2 chains
        let chains = find_dependency_chains(&lockfile.entries, "pkg-c", "3.0.0");
        assert_eq!(chains.len(), 2);
        // Each chain should have exactly 1 link (direct parent)
        assert!(chains.iter().all(|c| c.len() == 1));
        let parent_names: Vec<&str> = chains.iter().map(|c| c[0].name.as_str()).collect();
        assert!(parent_names.contains(&"pkg-a"));
        assert!(parent_names.contains(&"pkg-b"));
    }
}
