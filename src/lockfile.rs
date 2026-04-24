use crate::search::{DependencyEntry, DependencySpec};
use ignore::WalkBuilder;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use yarn_lock_parser::{Entry, parse_str};

#[derive(Debug, PartialEq)]
pub enum LockFileType {
    Yarn,
    Npm,
}

impl LockFileType {
    pub fn from_filename(name: &str) -> Option<LockFileType> {
        match name {
            "yarn.lock" => Some(LockFileType::Yarn),
            "package-lock.json" => Some(LockFileType::Npm),
            _ => None,
        }
    }

    pub fn file_name(&self) -> &str {
        match self {
            LockFileType::Yarn => "yarn.lock",
            LockFileType::Npm => "package-lock.json",
        }
    }
}

pub fn parse_lockfile(lock_file: &Path) -> Result<String, String> {
    read_to_string(lock_file).map_err(|err| err.to_string())
}

pub fn parse_dependency_entries(
    lockfile_type: &LockFileType,
    content: &str,
) -> Result<Vec<DependencyEntry>, String> {
    match lockfile_type {
        LockFileType::Yarn => parse_yarn_dependency_entries(content),
        LockFileType::Npm => parse_npm_dependency_entries(content),
    }
}

fn parse_yarn_dependency_entries(content: &str) -> Result<Vec<DependencyEntry>, String> {
    let parsed = parse_str(content).map_err(|err| err.to_string())?;

    Ok(yarn_entries_to_dependency_entries(&parsed.entries))
}

#[derive(Deserialize)]
struct NpmPackageLock {
    packages: BTreeMap<String, NpmPackage>,
}

#[derive(Deserialize)]
struct NpmPackage {
    version: Option<String>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
}

fn parse_npm_dependency_entries(content: &str) -> Result<Vec<DependencyEntry>, String> {
    let lockfile: NpmPackageLock = serde_json::from_str(content).map_err(|err| err.to_string())?;
    let mut descriptors_by_path: HashMap<String, Vec<DependencySpec>> = HashMap::new();

    for (path, package) in &lockfile.packages {
        for (name, requested_as) in &package.dependencies {
            let Some(dependency_path) = resolve_npm_dependency_path(path, name, &lockfile.packages)
            else {
                continue;
            };

            descriptors_by_path
                .entry(dependency_path)
                .or_default()
                .push(DependencySpec {
                    name: name.to_string(),
                    requested_as: requested_as.to_string(),
                });
        }
    }

    let entries = lockfile
        .packages
        .iter()
        .filter_map(|(path, package)| {
            let name = package_name_from_npm_path(path)?;
            let version = package.version.as_ref()?;
            let descriptors = descriptors_by_path.remove(path).unwrap_or_default();
            let dependencies = package
                .dependencies
                .iter()
                .map(|(name, requested_as)| DependencySpec {
                    name: name.to_string(),
                    requested_as: requested_as.to_string(),
                })
                .collect();

            Some(DependencyEntry {
                name: name.to_string(),
                version: version.to_string(),
                descriptors,
                dependencies,
            })
        })
        .collect();

    Ok(entries)
}

fn resolve_npm_dependency_path(
    parent_path: &str,
    dependency_name: &str,
    packages: &BTreeMap<String, NpmPackage>,
) -> Option<String> {
    let mut current_path = parent_path;

    loop {
        let candidate = if current_path.is_empty() {
            format!("node_modules/{}", dependency_name)
        } else {
            format!("{}/node_modules/{}", current_path, dependency_name)
        };

        if packages.contains_key(&candidate) {
            return Some(candidate);
        }

        if current_path.is_empty() {
            return None;
        }

        current_path = parent_npm_package_path(current_path);
    }
}

fn parent_npm_package_path(path: &str) -> &str {
    match path.rfind("/node_modules/") {
        Some(index) => &path[..index],
        None => "",
    }
}

fn package_name_from_npm_path(path: &str) -> Option<&str> {
    path.rsplit_once("node_modules/")
        .map(|(_, package_name)| package_name)
}

fn yarn_entries_to_dependency_entries(entries: &[Entry]) -> Vec<DependencyEntry> {
    entries
        .iter()
        .map(|entry| DependencyEntry {
            name: entry.name.to_string(),
            version: entry.version.to_string(),
            descriptors: entry
                .descriptors
                .iter()
                .map(|(name, requested_as)| DependencySpec {
                    name: name.to_string(),
                    requested_as: requested_as.to_string(),
                })
                .collect(),
            dependencies: entry
                .dependencies
                .iter()
                .map(|(name, requested_as)| DependencySpec {
                    name: name.to_string(),
                    requested_as: requested_as.to_string(),
                })
                .collect(),
        })
        .collect()
}

pub fn find_lockfiles(project_path: &str, recursive: bool) -> Vec<(LockFileType, PathBuf)> {
    let mut locks = Vec::new();

    for entry in WalkBuilder::new(project_path)
        .max_depth(if recursive { None } else { Some(1) })
        .build()
        .filter_map(|e| e.ok())
    {
        if let Some(name) = entry.file_name().to_str() {
            let path = entry.path().to_path_buf();

            if let Some(lockfile_type) = LockFileType::from_filename(name) {
                locks.push((lockfile_type, path));
            }
        }
    }

    locks
}

#[cfg(test)]
#[path = "lockfile_tests.rs"]
mod tests;
