use ignore::WalkBuilder;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_filename() {
        assert_eq!(
            LockFileType::from_filename("yarn.lock"),
            Some(LockFileType::Yarn)
        );

        assert_eq!(
            LockFileType::from_filename("package-lock.json"),
            Some(LockFileType::Npm)
        );
    }

    #[test]
    fn test_parse_lockfile_reads_contents() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "some lockfile content").unwrap();

        let result = parse_lockfile(tmp.path());
        assert_eq!(result, Ok("some lockfile content".to_string()));
    }

    #[test]
    fn test_find_lockfiles_recursive() {
        use std::fs;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        fs::write(dir_path.join("yarn.lock"), "").unwrap();

        let sub = dir_path.join("subfolder");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("package-lock.json"), "").unwrap();

        let non_recursive = find_lockfiles(dir_path.to_str().unwrap(), false);
        assert_eq!(non_recursive.len(), 1);

        let recursive = find_lockfiles(dir_path.to_str().unwrap(), true);
        assert_eq!(recursive.len(), 2);
    }
}
