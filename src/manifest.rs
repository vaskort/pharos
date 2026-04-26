use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestDependency {
    pub name: String,
    pub dependency_type: String,
    pub requested_as: String,
}

#[derive(Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "optionalDependencies")]
    optional_dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "peerDependencies")]
    peer_dependencies: BTreeMap<String, String>,
}

pub fn parse_package_json_dependencies(content: &str) -> Result<Vec<ManifestDependency>, String> {
    let package_json: PackageJson = serde_json::from_str(content).map_err(|err| err.to_string())?;
    let mut dependencies = Vec::new();

    dependencies.extend(dependencies_from_section(
        "dependencies",
        package_json.dependencies,
    ));
    dependencies.extend(dependencies_from_section(
        "devDependencies",
        package_json.dev_dependencies,
    ));
    dependencies.extend(dependencies_from_section(
        "optionalDependencies",
        package_json.optional_dependencies,
    ));
    dependencies.extend(dependencies_from_section(
        "peerDependencies",
        package_json.peer_dependencies,
    ));

    Ok(dependencies)
}

pub fn read_package_json_dependencies(path: &Path) -> Result<Vec<ManifestDependency>, String> {
    let content = read_to_string(path).map_err(|err| err.to_string())?;

    parse_package_json_dependencies(&content)
}

fn dependencies_from_section(
    dependency_type: &str,
    dependencies: BTreeMap<String, String>,
) -> Vec<ManifestDependency> {
    dependencies
        .into_iter()
        .map(|(name, requested_as)| ManifestDependency {
            name,
            dependency_type: dependency_type.to_string(),
            requested_as,
        })
        .collect()
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
