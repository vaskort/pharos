mod lockfile;
mod registry;
mod search;
mod utils;

use clap::Parser;
use colored::Colorize;
use lockfile::{find_lockfiles, parse_lockfile};
use search::{ChainLink, find_dependency_chains, package_exists};
use semver::Version;
use std::collections::HashMap;
use std::path::Path;
use utils::clean_version;
use yarn_lock_parser::parse_str;

use crate::{
    lockfile::LockFileType,
    registry::{RegistryCache, find_parent_versions},
};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Package to search for in the format name@version (e.g. qs@6.13.0)
    package: String,

    /// Path to the project root
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Search for lockfiles recursively in subdirectories
    #[arg(short, long, default_value = "false")]
    recursive: bool,
}

fn format_chain(chain: &[ChainLink], package_name: &str, package_version: &str) {
    if chain.is_empty() {
        print!(
            "{:}@{:} (is a direct dependency)",
            package_name, package_version
        );

        return;
    }

    let package_name_requested_as = &chain[0].requested_as;
    println!(
        "{:}@{:} (requested as {:})",
        package_name, package_version, package_name_requested_as
    );

    for (i, dep) in chain.iter().enumerate() {
        if i + 1 < chain.len() {
            println!(
                "    -> {:}@{:} (requested as {:})",
                dep.name,
                dep.version,
                chain[i + 1].requested_as
            );
        } else {
            print!("    -> {:}@{:}", dep.name, dep.version,);
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
            if let Ok(parsed) = Version::parse(version)
                && !parsed.pre.is_empty()
            {
                continue;
            }

            if let Some(version_info) = parent_data.versions.get(*version)
                && let Some(deps) = &version_info.dependencies
                && let Some(dep_version) = deps.get(package_name)
            {
                let clean_version = clean_version(dep_version);
                if let (Ok(dep_v), Ok(pkg_v)) = (
                    Version::parse(clean_version),
                    Version::parse(package_version),
                ) && dep_v > pkg_v
                    && min_fixed_version.is_none()
                {
                    min_fixed_version = Some(version.to_string());
                }
            }
        }
    }

    min_fixed_version
}

fn process_lockfile(
    lockfile_type: &LockFileType,
    path: &Path,
    package_name: &str,
    package_version: &str,
    registry_cache: &mut RegistryCache,
) {
    println!(
        "\n{}",
        "════════════════════════════════════════════════════════════".cyan()
    );
    println!("{}", format!("📁 {}", path.display()).cyan());
    println!(
        "{}",
        "════════════════════════════════════════════════════════════".cyan()
    );

    if matches!(lockfile_type, LockFileType::Npm) {
        println!(
            "  {}",
            format!(
                "⚠ {} parsing not yet supported, skipping",
                lockfile_type.file_name()
            )
            .yellow()
        );
        return;
    }

    let lockfile_content = match parse_lockfile(path) {
        Ok(content) => content,
        Err(err) => {
            println!("  {}", format!("✗ Failed to parse lockfile: {}", err).red());

            return;
        }
    };
    let parsed = match parse_str(&lockfile_content) {
        Ok(parsed) => parsed,
        Err(err) => {
            println!("  {}", format!("✗ Failed to parse lockfile: {}", err).red());

            return;
        }
    };

    if !package_exists(&parsed.entries, package_name, package_version) {
        println!(
            "  {}",
            format!("Package {}@{} not found", package_name, package_version).red()
        );

        return;
    }

    println!(
        "  {}",
        format!("✓ Found {}@{}", package_name, package_version).green()
    );

    let chains = find_dependency_chains(&parsed.entries, package_name, package_version);
    find_parent_versions(&chains, registry_cache);

    for (i, chain) in chains.iter().enumerate() {
        println!("\n  {}", format!("── Chain {} ──", i + 1).cyan());
        print!("  ");
        format_chain(chain, package_name, package_version);

        let mut chain_package_name: String = package_name.to_string();
        let mut chain_package_version: String = package_version.to_string();
        let mut fix_path: Vec<(String, String)> = Vec::new();

        for chain_link in chain {
            if let Some(min_updated_version) = show_parent_updates(
                registry_cache,
                &chain_package_name,
                &chain_package_version,
                &chain_link.name,
            ) {
                fix_path.push((chain_link.name.clone(), min_updated_version.clone()));
                chain_package_name = chain_link.name.clone();
                chain_package_version = min_updated_version;
            } else {
                println!(
                    "  {}",
                    format!(
                        "⚠ No {} version found that updates {} beyond {}",
                        chain_link.name, chain_package_name, chain_package_version
                    )
                    .yellow()
                );

                break;
            }
        }

        if let Some((pkg, ver)) = fix_path.last() {
            println!("\n Fix path:");
            for (pkg, ver) in &fix_path {
                println!("  {} >= {}", pkg, ver);
            }

            println!(
                "  {}",
                format!("→ Recommended: Update {} to >= {}", pkg, ver)
                    .green()
                    .bold()
            );
        } else {
            println!("  {}", "✗ No fix available for this chain".red());
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

    let lockfiles = find_lockfiles(&cli.path, cli.recursive);
    if lockfiles.is_empty() {
        eprintln!("No lockfiles found in {}", cli.path);
        std::process::exit(2);
    }

    let mut registry_cache: RegistryCache = HashMap::new();
    for (lockfile_type, path) in lockfiles {
        process_lockfile(
            &lockfile_type,
            &path,
            package_name,
            package_version,
            &mut registry_cache,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_parse_package {
        use super::*;

        #[test]
        fn test_simple() {
            let expected = Some(("pkg-a", "1.0.0"));

            let result = parse_package("pkg-a@1.0.0");

            assert_eq!(result, expected);
        }

        #[test]
        fn test_scoped() {
            let expected = Some(("@scope/pkg-a", "1.0.0"));

            let result = parse_package("@scope/pkg-a@1.0.0");

            assert_eq!(result, expected);
        }

        #[test]
        fn test_no_version() {
            let expected = None;

            let result = parse_package("pkg-a");
            assert_eq!(result, expected);

            let result = parse_package("pkg-a@");
            assert_eq!(result, expected);
        }

        #[test]
        fn test_empty() {
            let expected = None;

            let result = parse_package("");
            assert_eq!(result, expected);
        }
    }
}
