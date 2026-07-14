use super::*;
use crate::search::{ChainLink, DependencyChain, DependencyKind};

fn make_link(name: &str, version: &str, requested_as: &str) -> ChainLink {
    ChainLink {
        node_id: 0,
        name: name.to_string(),
        version: version.to_string(),
        locator: format!("{}@{}", name, version),
        requested_as: requested_as.to_string(),
        dependency_kind: DependencyKind::Normal,
    }
}

fn make_chain(links: Vec<ChainLink>) -> DependencyChain {
    DependencyChain {
        target_node_id: 0,
        target_locator: "target@1.0.0".to_string(),
        links,
        warnings: Vec::new(),
    }
}

mod find_unique_parents_tests {
    use super::*;

    #[test]
    fn returns_empty_for_empty_input() {
        let chains: Vec<DependencyChain> = vec![];
        let result = find_unique_parents(&chains);
        assert!(result.is_empty());
    }

    #[test]
    fn returns_empty_for_empty_inner_chains() {
        let chains = vec![make_chain(vec![]), make_chain(vec![])];
        let result = find_unique_parents(&chains);
        assert!(result.is_empty());
    }

    #[test]
    fn returns_single_parent() {
        let chains = vec![make_chain(vec![make_link("pkg-a", "1.0.0", "^1.0.0")])];
        let result = find_unique_parents(&chains);
        assert_eq!(result, vec!["pkg-a"]);
    }

    #[test]
    fn keeps_all_unique_names() {
        let chains = vec![
            make_chain(vec![
                make_link("pkg-a", "1.0.0", "^1.0.0"),
                make_link("pkg-b", "2.0.0", "^2.0.0"),
            ]),
            make_chain(vec![
                make_link("pkg-c", "3.0.0", "^3.0.0"),
                make_link("pkg-d", "4.0.0", "^4.0.0"),
            ]),
        ];
        let result = find_unique_parents(&chains);
        assert_eq!(result.len(), 4);
        assert_eq!(result, vec!["pkg-a", "pkg-b", "pkg-c", "pkg-d"]);
    }

    #[test]
    fn deduplicates_shared_parents() {
        let chains = vec![
            make_chain(vec![
                make_link("pkg-a", "1.0.0", "^1.0.0"),
                make_link("pkg-b", "2.0.0", "^2.0.0"),
                make_link("pkg-c", "3.0.0", "^3.0.0"),
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
            ]),
            make_chain(vec![
                make_link("pkg-d", "1.0.0", "^1.0.0"),
                make_link("pkg-e", "2.0.0", "^2.0.0"),
                make_link("pkg-f", "3.0.0", "^3.0.0"),
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
            ]),
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
            make_chain(vec![
                make_link("pkg-x", "1.0.0", "^1.0.0"),
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
            ]),
            make_chain(vec![
                make_link("pkg-shared", "1.0.0", "^1.0.0"),
                make_link("pkg-y", "1.0.0", "^1.0.0"),
            ]),
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
        let chains: Vec<DependencyChain> = vec![];
        let mut cache = RegistryCache::default();
        find_parent_versions(&chains, &[], &mut cache);
        assert!(cache.is_empty());
    }

    #[test]
    fn skips_cached_packages() {
        let mut cache = RegistryCache::default();
        let existing_data = RegistryResponse {
            versions: HashMap::from([("1.0.0".to_string(), VersionInfo::default())]),
        };
        cache.insert("pkg-a".to_string(), existing_data);

        let chains = vec![make_chain(vec![make_link("pkg-a", "1.0.0", "^1.0.0")])];
        find_parent_versions(&chains, &[], &mut cache);

        assert_eq!(cache.len(), 1);
        let cached = cache.get("pkg-a").unwrap();
        assert!(cached.versions.contains_key("1.0.0"));
        assert!(cached.versions.get("1.0.0").unwrap().dependencies.is_none());
    }
}

mod parallel_fetch_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    struct FakeFetcher {
        calls: AtomicUsize,
        active: AtomicUsize,
        max_active: AtomicUsize,
        requested: Mutex<Vec<String>>,
        fail: bool,
    }

    impl FakeFetcher {
        fn new(fail: bool) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                active: AtomicUsize::new(0),
                max_active: AtomicUsize::new(0),
                requested: Mutex::new(Vec::new()),
                fail,
            }
        }
    }

    impl RegistryFetcher for FakeFetcher {
        fn fetch(&self, package: &str) -> Result<RegistryResponse, String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.requested.lock().unwrap().push(package.to_string());
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_active.fetch_max(active, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(10));
            self.active.fetch_sub(1, Ordering::SeqCst);

            if self.fail {
                Err(format!("failed to fetch {}", package))
            } else {
                Ok(RegistryResponse {
                    versions: HashMap::new(),
                })
            }
        }
    }

    #[test]
    fn bounds_parallel_fetches_and_collects_every_result() {
        let fetcher = Arc::new(FakeFetcher::new(false));
        let packages = vec!["a", "b", "c", "d", "e"];
        let mut cache = RegistryCache::default();

        fetch_registry_versions_with(fetcher.as_ref(), &packages, &mut cache, 2);

        assert_eq!(cache.len(), 5);
        assert_eq!(fetcher.calls.load(Ordering::SeqCst), 5);
        assert!(fetcher.max_active.load(Ordering::SeqCst) <= 2);
        assert!(fetcher.max_active.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn caches_failures_and_does_not_retry_them_in_the_same_run() {
        let fetcher = FakeFetcher::new(true);
        let mut cache = RegistryCache::default();

        fetch_registry_versions_with(&fetcher, &["broken"], &mut cache, 8);
        fetch_registry_versions_with(&fetcher, &["broken"], &mut cache, 8);

        assert_eq!(fetcher.calls.load(Ordering::SeqCst), 1);
        assert_eq!(cache.error("broken"), Some("failed to fetch broken"));
    }

    #[test]
    fn deduplicates_packages_before_fetching() {
        let fetcher = FakeFetcher::new(false);
        let mut cache = RegistryCache::default();

        fetch_registry_versions_with(&fetcher, &["b", "a", "b", "a"], &mut cache, 8);

        assert_eq!(fetcher.calls.load(Ordering::SeqCst), 2);
        let requested = fetcher.requested.lock().unwrap();
        assert!(requested.contains(&"a".to_string()));
        assert!(requested.contains(&"b".to_string()));
    }
}
