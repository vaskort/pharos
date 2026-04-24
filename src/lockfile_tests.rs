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

    #[test]
    fn rejects_unsupported_lockfiles() {
        assert_eq!(LockFileType::from_filename("pnpm-lock.yaml"), None);
        assert_eq!(LockFileType::from_filename("package.json"), None);
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
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn reads_contents() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "some lockfile content").unwrap();

        let result = parse_lockfile(tmp.path());
        assert_eq!(result, Ok("some lockfile content".to_string()));
    }

    #[test]
    fn returns_error_when_file_cannot_be_read() {
        let result = parse_lockfile(Path::new("missing.lock"));

        assert!(result.is_err());
    }
}

mod yarn_entries_to_dependency_entries {
    use super::*;
    use std::fs;
    use yarn_lock_parser::parse_str;

    #[test]
    fn converts_yarn_entries_to_normalized_entries() {
        let content = fs::read_to_string("testdata/simple_chain.lock").unwrap();
        let lockfile = parse_str(&content).unwrap();

        let entries = yarn_entries_to_dependency_entries(&lockfile.entries);

        let pkg_a = entries.iter().find(|entry| entry.name == "pkg-a").unwrap();
        assert_eq!(pkg_a.version, "1.0.0");
        assert_eq!(pkg_a.descriptors[0].name, "pkg-a");
        assert_eq!(pkg_a.descriptors[0].requested_as, "^1.0.0");
        assert_eq!(pkg_a.dependencies[0].name, "pkg-b");
        assert_eq!(pkg_a.dependencies[0].requested_as, "^2.0.0");

        let pkg_b = entries.iter().find(|entry| entry.name == "pkg-b").unwrap();
        assert_eq!(pkg_b.version, "2.0.0");
        assert!(pkg_b.dependencies.is_empty());
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

    #[test]
    fn finds_supported_lockfiles_in_root_directory() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        fs::write(dir_path.join("yarn.lock"), "").unwrap();
        fs::write(dir_path.join("package-lock.json"), "").unwrap();
        fs::write(dir_path.join("package.json"), "{}").unwrap();

        let mut found: Vec<LockFileType> = find_lockfiles(dir_path.to_str().unwrap(), false)
            .into_iter()
            .map(|(lockfile_type, _)| lockfile_type)
            .collect();
        found.sort_by_key(|lockfile_type| lockfile_type.file_name().to_string());

        assert_eq!(found, vec![LockFileType::Npm, LockFileType::Yarn]);
    }
}
