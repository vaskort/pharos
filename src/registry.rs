use crate::search::ChainLink;
use reqwest::blocking::get;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

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
        Ok(value) => value.json::<RegistryResponse>(),
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
pub fn find_unique_parents(chains: &[Vec<ChainLink>]) -> Vec<&str> {
    let mut unique_parents = Vec::new();
    let mut seen = HashSet::new();

    for chain in chains {
        for chain_link in chain {
            if seen.insert(chain_link.name.as_str()) {
                unique_parents.push(chain_link.name.as_str());
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
pub fn find_parent_versions(chains: &[Vec<ChainLink>], registry_cache: &mut RegistryCache) {
    let unique_parents_to_get_data_for = find_unique_parents(chains);

    for parent in unique_parents_to_get_data_for {
        if registry_cache.contains_key(parent) {
            continue;
        }

        match get_package_data(parent) {
            Ok(data) => {
                registry_cache.insert(parent.to_string(), data);
            }
            Err(e) => {
                eprintln!(
                    "Something went wrong fetching data for {} with message {}",
                    parent, e
                )
            }
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
