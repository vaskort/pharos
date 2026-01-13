use clap::Parser;
use ignore::WalkBuilder;
use std::fs::read_to_string;
use std::path::Path;
use std::path::PathBuf;
use yarn_lock_parser::parse_str;

#[derive(Debug)]
enum LockFileType {
    Yarn,
    Npm,
}

impl LockFileType {
    fn file_name(&self) -> &str {
        match self {
            LockFileType::Yarn => "yarn.lock",
            LockFileType::Npm => "package-lock.json",
        }
    }

    fn from_filename(name: &str) -> Option<LockFileType> {
        match name {
            "yarn.lock" => Some(LockFileType::Yarn),
            "package-lock.json" => Some(LockFileType::Yarn),
            _ => None,
        }
    }
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    package: String,

    #[arg(short, long, default_value = ".")]
    path: String,
}

fn check_lockfile(project_path: &str) -> Result<LockFileType, String> {
    let yarn_exists = Path::new(project_path).join("yarn.lock").exists();
    let npm_exists = Path::new(project_path).join("package-lock.json").exists();

    match (yarn_exists, npm_exists) {
        (true, _) => Ok(LockFileType::Yarn),
        (_, true) => Ok(LockFileType::Npm),
        _ => Err("No lockfile found".to_string()),
    }
}

fn parse_lockfile(lock_file: &Path) -> Result<String, String> {
    read_to_string(lock_file).map_err(|err| err.to_string())
}

fn find_lockfiles(project_path: &str) -> Vec<(LockFileType, PathBuf)> {
    let mut locks = Vec::new();

    for entry in WalkBuilder::new(project_path)
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

fn main() {
    let cli = Cli::parse();

    let lockfile_type = match check_lockfile(&cli.path) {
        Ok(lockfile_type) => lockfile_type,
        Err(err) => panic!("{}", err),
    };

    let lockfiles = find_lockfiles(&cli.path);
    dbg!("lockfiles", &lockfiles);

    for lock_file in lockfiles {
        let lockfile_content = match parse_lockfile(&lock_file.1) {
            Ok(content) => content,
            Err(err) => panic!("{}", err),
        };
        let parsed = parse_str(&lockfile_content);
        dbg!(&parsed);
    }

    println!("Found lockfile, {:?}", lockfile_type)
}
