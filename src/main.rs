mod lockfile;
mod search;

use clap::Parser;
use lockfile::{find_lockfiles, parse_lockfile};
use search::{ChainLink, find_dependency_chains, package_exists};
use yarn_lock_parser::parse_str;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    package: String,

    #[arg(short, long, default_value = ".")]
    path: String,
}

fn format_chain(chain: &Vec<ChainLink>, package_name: &str) {
    // TODO: if the dep is the root then this panicks
    let package_name_requested_as = &chain[0].requested_as;
    print!("{:}@{:}", package_name, package_name_requested_as);

    for (i, dep) in chain.iter().enumerate() {
        if i + 1 < chain.len() {
            print!(
                " -> {:}@{:} (Requested as {:})",
                dep.name,
                dep.version,
                chain[i + 1].requested_as
            );
        } else {
            print!(" -> {:}@{:}", dep.name, dep.version,);
        }
    }

    println!();
}

fn parse_package(input: &str) -> Option<(&str, &str)> {
    if let Some((package_name, package_version)) = input.rsplit_once("@") {
        if package_version.starts_with(|c: char| c.is_ascii_digit()) {
            Some((package_name, package_version))
        } else {
            None
        }
    } else {
        None
    }
}

fn main() {
    let cli = Cli::parse();

    let (package_name, package_version) = match parse_package(&cli.package) {
        Some(result) => result,
        None => {
            println!("Invalid package format, did you forget the version?");
            return;
        }
    };

    let lockfiles = find_lockfiles(&cli.path);

    for (_, path) in lockfiles {
        println!("\nSearching in: {}", path.display());
        let lockfile_content = match parse_lockfile(&path) {
            Ok(content) => content,
            Err(err) => panic!("{}", err),
        };
        let parsed = parse_str(&lockfile_content).unwrap();

        if package_exists(&parsed.entries, &package_name, &package_version) {
            let chains = find_dependency_chains(&parsed.entries, &package_name, &package_version);
            for chain in chains {
                format_chain(&chain, &package_name);
            }
        } else {
            println!("Package {} not found", package_name);
        }
    }
}
