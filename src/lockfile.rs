use crate::search::{DependencyEdge, DependencyGraph, DependencyKind, DependencyNode, NodeId};
use ignore::WalkBuilder;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
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
) -> Result<DependencyGraph, String> {
    match lockfile_type {
        LockFileType::Yarn => parse_yarn_dependency_graph(content),
        LockFileType::Npm => parse_npm_dependency_graph(content),
    }
}

fn parse_yarn_dependency_graph(content: &str) -> Result<DependencyGraph, String> {
    let parsed = parse_str(content).map_err(|err| err.to_string())?;
    Ok(yarn_entries_to_dependency_graph(&parsed.entries))
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
    #[serde(default, rename = "optionalDependencies")]
    optional_dependencies: BTreeMap<String, String>,
}

fn parse_npm_dependency_graph(content: &str) -> Result<DependencyGraph, String> {
    let lockfile: NpmPackageLock = serde_json::from_str(content).map_err(|err| err.to_string())?;
    let package_nodes = lockfile
        .packages
        .iter()
        .filter_map(|(path, package)| {
            let name = package_name_from_npm_path(path)?;
            let version = package.version.as_ref()?;
            Some((path.clone(), name.to_string(), version.clone()))
        })
        .collect::<Vec<_>>();
    let node_ids_by_path = package_nodes
        .iter()
        .enumerate()
        .map(|(node_id, (path, _, _))| (path.clone(), node_id))
        .collect::<HashMap<_, _>>();

    let nodes = package_nodes
        .into_iter()
        .map(|(path, name, version)| {
            let package = &lockfile.packages[&path];
            let mut dependencies = Vec::new();
            add_npm_edges(
                &mut dependencies,
                &path,
                &package.dependencies,
                DependencyKind::Normal,
                &lockfile.packages,
                &node_ids_by_path,
            );
            add_npm_edges(
                &mut dependencies,
                &path,
                &package.optional_dependencies,
                DependencyKind::Optional,
                &lockfile.packages,
                &node_ids_by_path,
            );

            DependencyNode {
                name,
                version,
                locator: path,
                dependencies,
            }
        })
        .collect();

    Ok(DependencyGraph { nodes })
}

fn add_npm_edges(
    edges: &mut Vec<DependencyEdge>,
    parent_path: &str,
    dependencies: &BTreeMap<String, String>,
    kind: DependencyKind,
    packages: &BTreeMap<String, NpmPackage>,
    node_ids_by_path: &HashMap<String, NodeId>,
) {
    for (name, requested_as) in dependencies {
        let Some(dependency_path) = resolve_npm_dependency_path(parent_path, name, packages) else {
            continue;
        };
        let Some(target) = node_ids_by_path.get(&dependency_path).copied() else {
            continue;
        };
        if edges
            .iter()
            .any(|edge| edge.target == target && edge.requested_as == *requested_as)
        {
            continue;
        }
        edges.push(DependencyEdge {
            target,
            requested_as: requested_as.clone(),
            kind,
        });
    }
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

fn yarn_entries_to_dependency_graph(entries: &[Entry]) -> DependencyGraph {
    let descriptor_targets = entries
        .iter()
        .enumerate()
        .flat_map(|(node_id, entry)| {
            entry.descriptors.iter().map(move |(name, requested_as)| {
                ((name.to_string(), requested_as.to_string()), node_id)
            })
        })
        .collect::<HashMap<_, _>>();

    let nodes = entries
        .iter()
        .map(|entry| {
            let mut descriptors = entry
                .descriptors
                .iter()
                .map(|(name, requested_as)| format!("{}@{}", name, requested_as))
                .collect::<Vec<_>>();
            descriptors.sort();
            let locator = descriptors
                .first()
                .cloned()
                .unwrap_or_else(|| format!("{}@{}", entry.name, entry.version));

            let mut dependencies = Vec::new();
            add_yarn_edges(
                &mut dependencies,
                &entry.dependencies,
                DependencyKind::Normal,
                &descriptor_targets,
            );
            add_yarn_edges(
                &mut dependencies,
                &entry.optional_dependencies,
                DependencyKind::Optional,
                &descriptor_targets,
            );

            DependencyNode {
                name: entry.name.to_string(),
                version: entry.version.to_string(),
                locator,
                dependencies,
            }
        })
        .collect();

    DependencyGraph { nodes }
}

fn add_yarn_edges(
    edges: &mut Vec<DependencyEdge>,
    dependencies: &[(&str, &str)],
    kind: DependencyKind,
    descriptor_targets: &HashMap<(String, String), NodeId>,
) {
    for (name, requested_as) in dependencies {
        let key = (name.to_string(), requested_as.to_string());
        let Some(target) = descriptor_targets.get(&key).copied() else {
            continue;
        };
        if edges
            .iter()
            .any(|edge| edge.target == target && edge.requested_as == *requested_as)
        {
            continue;
        }
        edges.push(DependencyEdge {
            target,
            requested_as: requested_as.to_string(),
            kind,
        });
    }
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
