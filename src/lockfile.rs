use ignore::WalkBuilder;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
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
}

pub fn check_lockfile(project_path: &str) -> Result<LockFileType, String> {
    let yarn_exists = Path::new(project_path).join("yarn.lock").exists();
    let npm_exists = Path::new(project_path).join("package-lock.json").exists();

    match (yarn_exists, npm_exists) {
        (true, _) => Ok(LockFileType::Yarn),
        (_, true) => Ok(LockFileType::Npm),
        _ => Err("No lockfile found".to_string()),
    }
}

pub fn parse_lockfile(lock_file: &Path) -> Result<String, String> {
    read_to_string(lock_file).map_err(|err| err.to_string())
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
