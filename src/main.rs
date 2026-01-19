mod lockfile;
mod registry;
mod search;
mod utils;

use clap::Parser;
use lockfile::{find_lockfiles, parse_lockfile};
use search::{ChainLink, find_dependency_chains, package_exists};
use semver::Version;
use std::collections::HashMap;
use std::path::Path;
use utils::clean_version;
use yarn_lock_parser::parse_str;

use crate::registry::{RegistryCache, find_parent_versions};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    package: String,

    #[arg(short, long, default_value = ".")]
    path: String,

    #[arg(short, long, default_value = "false")]
    recursive: bool,
}

fn format_chain(chain: &Vec<ChainLink>, package_name: &str, package_version: &str) {
    if chain.is_empty() {
        print!(
            "{:}@{:} (is a direct dependency)",
            package_name, package_version
        );

        return;
    }

    let package_name_requested_as = &chain[0].requested_as;
    print!(
        "{:}@{:} (requested as {:})",
        package_name, package_version, package_name_requested_as
    );

    for (i, dep) in chain.iter().enumerate() {
        if i + 1 < chain.len() {
            print!(
                " -> {:}@{:} (requested as {:})",
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

fn show_parent_updates(
    registry_cache: &RegistryCache,
    package_name: &str,
    package_version: &str,
    parent: &str,
) -> Option<String> {
    let mut min_fixed_version: Option<String> = None;

    if let Some(parent_data) = registry_cache.get(parent) {
        let mut versions: Vec<&String> = parent_data.versions.keys().collect();
        versions.sort_by(|a, b| match (Version::parse(a), Version::parse(b)) {
            (Ok(v_a), Ok(v_b)) => v_a.cmp(&v_b),
            (Ok(_), Err(_)) => std::cmp::Ordering::Less,
            (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
            (Err(_), Err(_)) => a.cmp(b),
        });

        for version in &versions {
            // skip pre-release versions
            if let Ok(parsed) = Version::parse(version) {
                if !parsed.pre.is_empty() {
                    continue;
                }
            }

            if let Some(version_info) = parent_data.versions.get(*version) {
                if let Some(deps) = &version_info.dependencies {
                    if let Some(dep_version) = deps.get(package_name) {
                        let clean_version = clean_version(&dep_version);
                        if let (Ok(dep_v), Ok(pkg_v)) = (
                            Version::parse(clean_version),
                            Version::parse(package_version),
                        ) {
                            if dep_v > pkg_v {
                                if min_fixed_version.is_none() {
                                    min_fixed_version = Some(version.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    min_fixed_version
}

fn process_lockfile(
    path: &Path,
    package_name: &str,
    package_version: &str,
    registry_cache: &mut RegistryCache,
) {
    println!("\n════════════════════════════════════════════════════════════");
    println!("📁 {}", path.display());
    println!("════════════════════════════════════════════════════════════");

    let lockfile_content = match parse_lockfile(&path) {
        Ok(content) => content,
        Err(err) => {
            println!("  ✗ Failed to parse lockfile: {}", err);
            return;
        }
    };
    let parsed = parse_str(&lockfile_content).unwrap();

    if !package_exists(&parsed.entries, &package_name, &package_version) {
        println!("  Package {}@{} not found", package_name, package_version);
        return;
    }

    println!("  ✓ Found {}@{}", package_name, package_version);

    let chains = find_dependency_chains(&parsed.entries, &package_name, &package_version);
    find_parent_versions(&chains, registry_cache);

    for (i, chain) in chains.iter().enumerate() {
        println!("\n  ── Chain {} ──", i + 1);
        print!("  ");
        format_chain(&chain, &package_name, &package_version);

        let mut chain_package_name: String = package_name.to_string();
        let mut chain_package_version: String = package_version.to_string();
        let mut fix_path: Vec<(String, String)> = Vec::new();

        for chain_link in chain {
            if let Some(min_updated_version) = show_parent_updates(
                &registry_cache,
                &chain_package_name,
                &chain_package_version,
                &chain_link.name,
            ) {
                fix_path.push((chain_link.name.clone(), min_updated_version.clone()));
                chain_package_name = chain_link.name.clone();
                chain_package_version = min_updated_version;
            } else {
                println!(
                    "  ⚠ No {} version found that updates {} beyond {}",
                    chain_link.name, chain_package_name, chain_package_version
                );
                break;
            }
        }

        if !fix_path.is_empty() {
            println!("\n Fix path:");
            for (pkg, ver) in &fix_path {
                println!("  {} >= {}", pkg, ver);
            }

            let (pkg, ver) = fix_path.last().unwrap();
            println!("\n  → Recommended: Update {} to >= {}", pkg, ver);
        } else {
            println!("  ✗ No fix available for this chain");
        }
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
    let mut registry_cache: RegistryCache = HashMap::new();

    for (_, path) in lockfiles {
        process_lockfile(&path, package_name, package_version, &mut registry_cache);
    }
}
