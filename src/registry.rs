use reqwest::blocking::get;
use serde::Deserialize;
use std::collections::HashMap;

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
