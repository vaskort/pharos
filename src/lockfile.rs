use crate::search::{DependencyEntry, DependencySpec};
use ignore::WalkBuilder;
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
        LockFileType::Npm => Err("package-lock.json parsing not yet supported".to_string()),
    }
}

fn parse_yarn_dependency_entries(content: &str) -> Result<Vec<DependencyEntry>, String> {
    let parsed = parse_str(content).map_err(|err| err.to_string())?;

    Ok(yarn_entries_to_dependency_entries(&parsed.entries))
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
