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

fn main() {
    let cli = Cli::parse();

    let lockfiles = find_lockfiles(&cli.path);

    for (_, path) in lockfiles {
        println!("\nSearching in: {}", path.display());
        let lockfile_content = match parse_lockfile(&path) {
            Ok(content) => content,
            Err(err) => panic!("{}", err),
        };
        let parsed = parse_str(&lockfile_content).unwrap();

        if package_exists(&parsed.entries, &cli.package) {
            let chains = find_dependency_chains(&parsed.entries, &cli.package);
            for chain in chains {
                format_chain(&chain, &cli.package);
            }
        } else {
            println!("Package {} not found", cli.package);
        }
    }
}
