use clap::Parser;
use std::fs::read_to_string;
use std::path::Path;

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

fn main() {
    let cli = Cli::parse();
    dbg!(cli.package);

    let lockfile_type = match check_lockfile(&cli.path) {
        Ok(lockfile_type) => lockfile_type,
        Err(err) => panic!("{}", err),
    };

    dbg!(&lockfile_type);

    let lockfile_path = Path::new(&cli.path).join(lockfile_type.file_name());
    let lockfile_content = match parse_lockfile(&lockfile_path) {
        Ok(content) => content,
        Err(err) => panic!("{}", err),
    };

    dbg!(&lockfile_content);

    println!("Found lockfile, {:?}", lockfile_type)
}
