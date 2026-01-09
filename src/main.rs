use clap::Parser;
use std::path::Path;

#[derive(Debug)]
enum LockFileType {
    Yarn,
    Npm,
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
    let npm_exists = Path::new("package-lock.json").exists();

    match (yarn_exists, npm_exists) {
        (true, _) => Ok(LockFileType::Yarn),
        (_, true) => Ok(LockFileType::Npm),
        _ => Err("No lockfile found".to_string()),
    }
}

fn main() {
    let cli = Cli::parse();
    println!("Hello bro {}", cli.package);

    let lockfile_type = match check_lockfile(&cli.path) {
        Ok(lockfile_type) => lockfile_type,
        Err(err) => panic!("{}", err),
    };

    println!("Found lockfile, {:?}", lockfile_type)
}
