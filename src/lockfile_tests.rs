use super::*;

mod from_filename {
    use super::*;

    #[test]
    fn recognizes_supported_lockfiles() {
        assert_eq!(
            LockFileType::from_filename("yarn.lock"),
            Some(LockFileType::Yarn)
        );
        assert_eq!(
            LockFileType::from_filename("package-lock.json"),
            Some(LockFileType::Npm)
        );
    }
}

mod file_name {
    use super::*;

    #[test]
    fn returns_expected_name_for_each_lockfile_type() {
        assert_eq!(LockFileType::Yarn.file_name(), "yarn.lock");
        assert_eq!(LockFileType::Npm.file_name(), "package-lock.json");
    }
}

mod parse_lockfile {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn reads_contents() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "some lockfile content").unwrap();

        let result = parse_lockfile(tmp.path());
        assert_eq!(result, Ok("some lockfile content".to_string()));
    }
}

mod find_lockfiles {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn respects_recursive_flag() {
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
