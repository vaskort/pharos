use super::*;
use crate::search::ChainLink;

fn make_link(name: &str, version: &str, requested_as: &str) -> ChainLink {
    ChainLink {
        name: name.to_string(),
        version: version.to_string(),
        requested_as: requested_as.to_string(),
    }
}

mod find_unique_parents_tests {
    use super::*;

    #[test]
    fn returns_empty_for_empty_input() {
        let chains: Vec<Vec<ChainLink>> = vec![];
        let result = find_unique_parents(&chains);
        assert!(result.is_empty());
    }

    #[test]
    fn returns_empty_for_empty_inner_chains() {
        let chains: Vec<Vec<ChainLink>> = vec![vec![], vec![]];
        let result = find_unique_parents(&chains);
        assert!(result.is_empty());
    }

    #[test]
    fn returns_single_parent() {
        let chains = vec![vec![make_link("pkg-a", "1.0.0", "^1.0.0")]];
        let result = find_unique_parents(&chains);
        assert_eq!(result, vec!["pkg-a"]);
    }

    #[test]
    fn keeps_all_unique_names() {
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
    fn deduplicates_shared_parents() {
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
    fn preserves_first_seen_order() {
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
}

mod find_parent_versions_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn leaves_cache_unchanged_for_empty_chains() {
        let chains: Vec<Vec<ChainLink>> = vec![];
        let mut cache: RegistryCache = HashMap::new();
        find_parent_versions(&chains, &mut cache);
        assert!(cache.is_empty());
    }

    #[test]
    fn skips_cached_packages() {
        let mut cache: RegistryCache = HashMap::new();
        let existing_data = RegistryResponse {
            versions: HashMap::from([("1.0.0".to_string(), VersionInfo { dependencies: None })]),
        };
        cache.insert("pkg-a".to_string(), existing_data);

        let chains = vec![vec![make_link("pkg-a", "1.0.0", "^1.0.0")]];
        find_parent_versions(&chains, &mut cache);

        assert_eq!(cache.len(), 1);
        let cached = cache.get("pkg-a").unwrap();
        assert!(cached.versions.contains_key("1.0.0"));
        assert!(cached.versions.get("1.0.0").unwrap().dependencies.is_none());
    }
}
