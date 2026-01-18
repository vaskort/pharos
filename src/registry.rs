use crate::search::ChainLink;
use reqwest::blocking::get;
use serde::Deserialize;
use std::collections::HashMap;

pub type RegistryCache = HashMap<String, RegistryResponse>;

#[derive(Deserialize, Debug)]
pub struct RegistryResponse {
    versions: HashMap<String, VersionInfo>,
}

#[derive(Deserialize, Debug)]
struct VersionInfo {
    dependencies: Option<HashMap<String, String>>,
}

pub fn get_package_data(package: &str) -> Result<RegistryResponse, reqwest::Error> {
    let registry_url: String = format!("https://registry.npmjs.org/{}", package);
    let result = get(registry_url);

    match result {
        Ok(value) => {
            let parsed = value.json::<RegistryResponse>();
            parsed
        }
        Err(err) => Err(err),
    }
}

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

pub fn find_parent_versions(chains: &Vec<Vec<ChainLink>>, registry_cache: &mut RegistryCache) {
    let unique_parents_to_get_data_for = find_unique_parents(&chains);

    for parent in unique_parents_to_get_data_for {
        if !registry_cache.contains_key(parent) {
            if let Ok(data) = get_package_data(parent) {
                registry_cache.insert(parent.to_string(), data);
            }
        }

        dbg!(&registry_cache.keys());
    }
}
