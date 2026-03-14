use crate::search::ChainLink;
use reqwest::blocking::get;
use serde::Deserialize;
use std::collections::HashMap;

/// A cache that maps package names to their registry data,
/// so we don't make duplicate HTTP requests for the same package.
pub type RegistryCache = HashMap<String, RegistryResponse>;

/// The response from the npm registry for a given package.
/// Contains a map of all published versions and their metadata.
#[derive(Deserialize, Debug)]
pub struct RegistryResponse {
    pub versions: HashMap<String, VersionInfo>,
}

/// Metadata for a single published version of a package.
///
/// For example, version `4.17.21` of `lodash` might have:
/// - `dependencies`: `{"some-lib": "^2.0.0", "other-lib": "~1.5.0"}`
///
/// `dependencies` is `None` when the version has no dependencies.
#[derive(Deserialize, Debug)]
pub struct VersionInfo {
    pub dependencies: Option<HashMap<String, String>>,
}

/// Fetches package metadata from the npm registry.
///
/// Makes a GET request to `https://registry.npmjs.org/{package}`
/// and parses the JSON response into a `RegistryResponse`.
///
/// # Arguments
/// * `package` - The npm package name (e.g. "lodash", "react")
pub fn get_package_data(package: &str) -> Result<RegistryResponse, reqwest::Error> {
    let registry_url: String = format!("https://registry.npmjs.org/{}", package);
    let result = get(registry_url);

    match result {
        Ok(value) => {
            
            value.json::<RegistryResponse>()
        }
        Err(err) => Err(err),
    }
}

/// Collects all unique package names from a list of dependency chains.
///
/// A package may appear in multiple chains (e.g. `lodash` could be a dependency
/// of both `react` and `express`), but we only need its name once when looking
/// up registry data.
///
/// # Arguments
/// * `chains` - A list of dependency chains, where each chain is a `Vec<ChainLink>`
///   representing the path from a direct dependency down to the vulnerable package.
///
/// # Returns
/// A deduplicated list of package names, in the order they were first encountered.
pub fn find_unique_parents(chains: &Vec<Vec<ChainLink>>) -> Vec<&str> {
    let mut unique_parents = Vec::new();

    for chain in chains {
        for chain_link in chain {
            if !unique_parents.contains(&chain_link.name.as_str()) {
                unique_parents.push(&chain_link.name);
            } else {
                continue;
            }
        }
    }

    unique_parents
}

/// Fetches registry data for all unique parent packages in the dependency chains.
///
/// Uses `find_unique_parents` to determine which packages need data,
/// then fetches from the npm registry for any package not already in the cache.
/// Results are stored in `registry_cache` to avoid duplicate requests.
///
/// # Arguments
/// * `chains` - The dependency chains to extract parent package names from.
/// * `registry_cache` - A mutable cache that stores previously fetched registry data.
pub fn find_parent_versions(chains: &Vec<Vec<ChainLink>>, registry_cache: &mut RegistryCache) {
    let unique_parents_to_get_data_for = find_unique_parents(chains);

    for parent in unique_parents_to_get_data_for {
        if !registry_cache.contains_key(parent)
            && let Ok(data) = get_package_data(parent) {
                registry_cache.insert(parent.to_string(), data);
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::ChainLink;

    fn make_link(name: &str, version: &str, requested_as: &str) -> ChainLink {
        ChainLink {
            name: name.to_string(),
            version: version.to_string(),
            requested_as: requested_as.to_string(),
        }
    }

    #[test]
    fn test_find_unique_parents_empty() {
        let chains: Vec<Vec<ChainLink>> = vec![];
        let result = find_unique_parents(&chains);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_unique_parents_empty_inner_chains() {
        // Chains exist but contain no links
        let chains: Vec<Vec<ChainLink>> = vec![vec![], vec![]];
        let result = find_unique_parents(&chains);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_unique_parents_single_link() {
        let chains = vec![vec![make_link("pkg-a", "1.0.0", "^1.0.0")]];
        let result = find_unique_parents(&chains);
        assert_eq!(result, vec!["pkg-a"]);
    }

    #[test]
    fn test_find_unique_parents_all_unique() {
        // Two chains, no shared packages — nothing to deduplicate
        let chains = vec![
            vec![
                make_link("pkg-a", "1.0.0", "^1.0.0"),
                make_link("pkg-b", "2.0.0", "^2.0.0"),
            ],
            vec![
                make_link("pkg-c", "3.0.0", "^3.0.0"),
                make_link("pkg-d", "4.0.0", "^4.0.0"),
            ],
        ];
        let result = find_unique_parents(&chains);
        assert_eq!(result.len(), 4);
        assert_eq!(result, vec!["pkg-a", "pkg-b", "pkg-c", "pkg-d"]);
    }

    #[test]
    fn test_find_unique_parents_deduplicates() {
        let chains = vec![
            vec![
                make_link("pkg-a", "1.0.0", "^1.0.0"),
                make_link("pkg-b", "2.0.0", "^2.0.0"),
                make_link("pkg-c", "3.0.0", "^3.0.0"),
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
            ],
            vec![
                make_link("pkg-d", "1.0.0", "^1.0.0"),
                make_link("pkg-e", "2.0.0", "^2.0.0"),
                make_link("pkg-f", "3.0.0", "^3.0.0"),
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
            ],
        ];

        let result = find_unique_parents(&chains);
        assert_eq!(result.len(), 7);
        assert_eq!(
            result,
            vec![
                "pkg-a",
                "pkg-b",
                "pkg-c",
                "pkg-shared",
                "pkg-d",
                "pkg-e",
                "pkg-f"
            ]
        );
    }

    #[test]
    fn test_find_unique_parents_preserves_first_seen_order() {
        // Same package appears at different positions across chains —
        // it should appear once, at the position it was first encountered.
        let chains = vec![
            vec![
                make_link("pkg-x", "1.0.0", "^1.0.0"),
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
            ],
            vec![
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
                make_link("pkg-y", "1.0.0", "^1.0.0"),
            ],
        ];
        let result = find_unique_parents(&chains);
        assert_eq!(result, vec!["pkg-x", "pkg-shared", "pkg-y"]);
    }

    #[test]
    fn test_find_parent_versions_empty_chains_leaves_cache_unchanged() {
        let chains: Vec<Vec<ChainLink>> = vec![];
        let mut cache: RegistryCache = HashMap::new();
        find_parent_versions(&chains, &mut cache);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_find_parent_versions_skips_cached_packages() {
        // Pre-populate the cache with pkg-a data
        let mut cache: RegistryCache = HashMap::new();
        let existing_data = RegistryResponse {
            versions: HashMap::from([("1.0.0".to_string(), VersionInfo { dependencies: None })]),
        };
        cache.insert("pkg-a".to_string(), existing_data);

        let chains = vec![vec![make_link("pkg-a", "1.0.0", "^1.0.0")]];
        find_parent_versions(&chains, &mut cache);

        // Cache should still have exactly 1 entry — no duplicate fetch
        assert_eq!(cache.len(), 1);
        // And it should still be our original data (version "1.0.0" with no deps)
        let cached = cache.get("pkg-a").unwrap();
        assert!(cached.versions.contains_key("1.0.0"));
        assert!(cached.versions.get("1.0.0").unwrap().dependencies.is_none());
    }
}
