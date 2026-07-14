use crate::search::DependencyChain;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const MAX_REGISTRY_WORKERS: usize = 8;

#[derive(Clone, Debug)]
enum RegistryLookup {
    Success(RegistryResponse),
    Failure(String),
}

#[derive(Default, Debug)]
pub struct RegistryCache {
    entries: HashMap<String, RegistryLookup>,
}

impl RegistryCache {
    pub fn get(&self, package: &str) -> Option<&RegistryResponse> {
        match self.entries.get(package) {
            Some(RegistryLookup::Success(response)) => Some(response),
            Some(RegistryLookup::Failure(_)) | None => None,
        }
    }

    pub fn error(&self, package: &str) -> Option<&str> {
        match self.entries.get(package) {
            Some(RegistryLookup::Failure(error)) => Some(error),
            Some(RegistryLookup::Success(_)) | None => None,
        }
    }

    pub fn insert(&mut self, package: String, response: RegistryResponse) {
        self.entries
            .insert(package, RegistryLookup::Success(response));
    }

    fn insert_error(&mut self, package: String, error: String) {
        self.entries.insert(package, RegistryLookup::Failure(error));
    }

    pub fn contains_key(&self, package: &str) -> bool {
        self.entries.contains_key(package)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<const N: usize> From<[(String, RegistryResponse); N]> for RegistryCache {
    fn from(entries: [(String, RegistryResponse); N]) -> Self {
        let mut cache = Self::default();
        for (package, response) in entries {
            cache.insert(package, response);
        }
        cache
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct RegistryResponse {
    pub versions: HashMap<String, VersionInfo>,
}

#[derive(Clone, Default, Deserialize, Debug)]
pub struct VersionInfo {
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: Option<HashMap<String, String>>,
}

pub trait RegistryFetcher: Sync {
    fn fetch(&self, package: &str) -> Result<RegistryResponse, String>;
}

struct NpmRegistryClient {
    client: Client,
    base_url: String,
}

impl NpmRegistryClient {
    fn new() -> Result<Self, String> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.npm.install-v1+json"),
        );
        let client = Client::builder()
            .user_agent(format!("pharos-cli/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(10))
            .default_headers(headers)
            .build()
            .map_err(|err| format!("failed to create npm registry client: {}", err))?;
        let base_url = std::env::var("PHAROS_REGISTRY_URL")
            .unwrap_or_else(|_| "https://registry.npmjs.org".to_string())
            .trim_end_matches('/')
            .to_string();
        Ok(Self { client, base_url })
    }
}

impl RegistryFetcher for NpmRegistryClient {
    fn fetch(&self, package: &str) -> Result<RegistryResponse, String> {
        let registry_url = format!("{}/{}", self.base_url, package);
        self.client
            .get(registry_url)
            .send()
            .map_err(|err| format!("request failed: {}", err))?
            .error_for_status()
            .map_err(|err| format!("registry returned an error: {}", err))?
            .json::<RegistryResponse>()
            .map_err(|err| format!("invalid registry response: {}", err))
    }
}

pub fn find_unique_parents(chains: &[DependencyChain]) -> Vec<&str> {
    let mut unique_parents = Vec::new();
    let mut seen = HashSet::new();

    for chain in chains {
        for chain_link in &chain.links {
            if seen.insert(chain_link.name.as_str()) {
                unique_parents.push(chain_link.name.as_str());
            }
        }
    }

    unique_parents
}

pub fn find_parent_versions(
    chains: &[DependencyChain],
    additional_packages: &[&str],
    registry_cache: &mut RegistryCache,
) {
    let mut packages = find_unique_parents(chains);
    for package in additional_packages {
        if !packages.contains(package) {
            packages.push(package);
        }
    }

    let fetcher = match NpmRegistryClient::new() {
        Ok(fetcher) => fetcher,
        Err(error) => {
            for package in packages {
                if !registry_cache.contains_key(package) {
                    registry_cache.insert_error(package.to_string(), error.clone());
                }
            }
            return;
        }
    };
    fetch_registry_versions_with(&fetcher, &packages, registry_cache, MAX_REGISTRY_WORKERS);
}

pub fn fetch_registry_versions_with<F: RegistryFetcher>(
    fetcher: &F,
    packages: &[&str],
    registry_cache: &mut RegistryCache,
    max_workers: usize,
) {
    let mut seen = HashSet::new();
    let missing = packages
        .iter()
        .filter_map(|package| {
            let package = *package;
            (seen.insert(package) && !registry_cache.contains_key(package))
                .then(|| package.to_string())
        })
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }

    let worker_count = max_workers.max(1).min(missing.len());
    let chunk_size = missing.len().div_ceil(worker_count);
    let results = std::thread::scope(|scope| {
        let handles = missing
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|package| (package.clone(), fetcher.fetch(package)))
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();

        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("registry worker panicked"))
            .collect::<Vec<_>>()
    });

    for (package, result) in results {
        match result {
            Ok(response) => registry_cache.insert(package, response),
            Err(error) => registry_cache.insert_error(package, error),
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
