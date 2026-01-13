mod lockfile;
mod search;

use clap::Parser;
use lockfile::{check_lockfile, find_lockfiles, parse_lockfile};
use search::{find_parents, package_exists};
use yarn_lock_parser::parse_str;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    package: String,

    #[arg(short, long, default_value = ".")]
    path: String,
}

fn main() {
    let cli = Cli::parse();

    let lockfile_type = match check_lockfile(&cli.path) {
        Ok(lockfile_type) => lockfile_type,
        Err(err) => panic!("{}", err),
    };

    let lockfiles = find_lockfiles(&cli.path);

    for lock_file in lockfiles {
        let lockfile_content = match parse_lockfile(&lock_file.1) {
            Ok(content) => content,
            Err(err) => panic!("{}", err),
        };
        let parsed = parse_str(&lockfile_content).unwrap();

        if package_exists(&parsed.entries, &cli.package) {
            let parents = find_parents(&parsed.entries, &cli.package);
            println!("Found {} - parents: {:?}", cli.package, parents);
        } else {
            println!("Package {} not found", cli.package);
        }
    }
}
