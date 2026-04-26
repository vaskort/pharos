mod lockfile;
mod manifest;
mod registry;
mod search;
mod utils;

use clap::Parser;
use colored::Colorize;
use lockfile::{find_lockfiles, parse_dependency_entries, parse_lockfile};
use manifest::{ManifestDependency, read_package_json_dependencies};
use search::{ChainLink, find_dependency_chains, package_exists};
use semver::Version;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use utils::clean_version;

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

    /// Print machine-readable JSON output
    #[arg(long)]
    json: bool,
}

#[derive(Debug, PartialEq)]
struct PackageSpec<'a> {
    name: &'a str,
    version: &'a str, // the cleaned/validated version string
}

#[derive(Debug, PartialEq)]
enum ParseError {
    Empty,
    MissingVersion,
    InvalidVersion(String),
}

#[derive(Debug, Serialize)]
struct Report {
    package: ReportPackage,
    lockfiles: Vec<LockfileReport>,
}

#[derive(Debug, Serialize)]
struct ReportPackage {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct LockfileReport {
    path: String,
    lockfile_type: String,
    status: LockfileStatus,
    chains: Vec<ChainReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum LockfileStatus {
    Found,
    NotFound,
    Error,
}

#[derive(Debug, Serialize)]
struct ChainReport {
    links: Vec<ChainLinkReport>,
    owner: Option<DependencyOwner>,
    fix_path: Vec<FixStep>,
    recommended: Option<FixStep>,
    warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DependencyOwner {
    name: String,
    dependency_type: String,
    requested_as: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ChainOwnerGroup {
    owner: Option<DependencyOwner>,
    chain_indexes: Vec<usize>,
}

#[derive(Clone, Debug, Serialize)]
struct ChainLinkReport {
    name: String,
    version: String,
    requested_as: String,
}

#[derive(Clone, Debug, Serialize)]
struct FixStep {
    package: String,
    minimum_version: String,
}

fn format_chain(chain: &[ChainLinkReport], package_name: &str, package_version: &str) {
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

fn parse_package<'a>(input: &'a str) -> Result<PackageSpec<'a>, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    if let Some((package_name, package_version)) = input.rsplit_once("@") {
        let package_version = package_version.strip_prefix('v').unwrap_or(package_version);
        if package_version.starts_with(|c: char| c.is_ascii_digit()) {
            Ok(PackageSpec {
                name: package_name,
                version: package_version,
            })
        } else {
            Err(ParseError::InvalidVersion(package_version.to_string()))
        }
    } else {
        Err(ParseError::MissingVersion)
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
            if is_prerelease(version) {
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

fn is_prerelease(version: &str) -> bool {
    match Version::parse(version) {
        Ok(parsed) => !parsed.pre.is_empty(),
        Err(_) => false,
    }
}

fn lockfile_type_name(lockfile_type: &LockFileType) -> String {
    match lockfile_type {
        LockFileType::Yarn => "yarn".to_string(),
        LockFileType::Npm => "npm".to_string(),
    }
}

fn report_chain(
    chain: &[ChainLink],
    package_name: &str,
    package_version: &str,
    registry_cache: &RegistryCache,
    manifest_dependencies: &[ManifestDependency],
) -> ChainReport {
    let mut chain_package_name: String = package_name.to_string();
    let mut chain_package_version: String = package_version.to_string();
    let mut fix_path: Vec<FixStep> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for chain_link in chain {
        if let Some(min_updated_version) = show_parent_updates(
            registry_cache,
            &chain_package_name,
            &chain_package_version,
            &chain_link.name,
        ) {
            fix_path.push(FixStep {
                package: chain_link.name.clone(),
                minimum_version: min_updated_version.clone(),
            });
            chain_package_name = chain_link.name.clone();
            chain_package_version = min_updated_version;
        } else {
            warnings.push(format!(
                "No {} version found that updates {} beyond {}",
                chain_link.name, chain_package_name, chain_package_version
            ));

            break;
        }
    }

    ChainReport {
        links: chain
            .iter()
            .map(|chain_link| ChainLinkReport {
                name: chain_link.name.clone(),
                version: chain_link.version.clone(),
                requested_as: chain_link.requested_as.clone(),
            })
            .collect(),
        owner: find_chain_owner(chain, package_name, manifest_dependencies),
        recommended: fix_path.last().cloned(),
        fix_path,
        warnings,
    }
}

fn find_chain_owner(
    chain: &[ChainLink],
    package_name: &str,
    manifest_dependencies: &[ManifestDependency],
) -> Option<DependencyOwner> {
    let owner_name = chain
        .last()
        .map(|chain_link| chain_link.name.as_str())
        .unwrap_or(package_name);

    manifest_dependencies
        .iter()
        .find(|dependency| dependency.name == owner_name)
        .map(|dependency| DependencyOwner {
            name: dependency.name.clone(),
            dependency_type: dependency.dependency_type.clone(),
            requested_as: dependency.requested_as.clone(),
        })
}

fn manifest_dependencies_for_lockfile(path: &Path) -> (Vec<ManifestDependency>, Option<String>) {
    let Some(parent) = path.parent() else {
        return (Vec::new(), None);
    };
    let manifest_path = parent.join("package.json");

    if !manifest_path.exists() {
        return (Vec::new(), None);
    }

    match read_package_json_dependencies(&manifest_path) {
        Ok(dependencies) => (dependencies, None),
        Err(err) => (
            Vec::new(),
            Some(format!(
                "Failed to parse package.json at {}: {}",
                manifest_path.display(),
                err
            )),
        ),
    }
}

fn group_chains_by_owner(chains: &[ChainReport]) -> Vec<ChainOwnerGroup> {
    let mut groups: Vec<ChainOwnerGroup> = Vec::new();

    for (index, chain) in chains.iter().enumerate() {
        if let Some(group) = groups.iter_mut().find(|group| group.owner == chain.owner) {
            group.chain_indexes.push(index);
        } else {
            groups.push(ChainOwnerGroup {
                owner: chain.owner.clone(),
                chain_indexes: vec![index],
            });
        }
    }

    groups
}

fn analyze_lockfile(
    lockfile_type: &LockFileType,
    path: &Path,
    package_name: &str,
    package_version: &str,
    registry_cache: &mut RegistryCache,
) -> LockfileReport {
    let path_display = path.display().to_string();
    let lockfile_type_name = lockfile_type_name(lockfile_type);

    let lockfile_content = match parse_lockfile(path) {
        Ok(content) => content,
        Err(err) => {
            return LockfileReport {
                path: path_display,
                lockfile_type: lockfile_type_name,
                status: LockfileStatus::Error,
                chains: Vec::new(),
                error: Some(format!("Failed to parse lockfile: {}", err)),
            };
        }
    };
    let entries = match parse_dependency_entries(lockfile_type, &lockfile_content) {
        Ok(entries) => entries,
        Err(err) => {
            return LockfileReport {
                path: path_display,
                lockfile_type: lockfile_type_name,
                status: LockfileStatus::Error,
                chains: Vec::new(),
                error: Some(format!(
                    "Failed to parse {}: {}",
                    lockfile_type.file_name(),
                    err
                )),
            };
        }
    };

    if !package_exists(&entries, package_name, package_version) {
        return LockfileReport {
            path: path_display,
            lockfile_type: lockfile_type_name,
            status: LockfileStatus::NotFound,
            chains: Vec::new(),
            error: None,
        };
    }

    let chains = find_dependency_chains(&entries, package_name, package_version);
    find_parent_versions(&chains, registry_cache);
    let (manifest_dependencies, manifest_warning) = manifest_dependencies_for_lockfile(path);
    let chains = chains
        .iter()
        .map(|chain| {
            let mut report = report_chain(
                chain,
                package_name,
                package_version,
                registry_cache,
                &manifest_dependencies,
            );
            if let Some(warning) = &manifest_warning {
                report.warnings.push(warning.clone());
            }
            report
        })
        .collect();

    LockfileReport {
        path: path_display,
        lockfile_type: lockfile_type_name,
        status: LockfileStatus::Found,
        chains,
        error: None,
    }
}

fn print_lockfile_report(report: &LockfileReport, package_name: &str, package_version: &str) {
    println!(
        "\n{}",
        "════════════════════════════════════════════════════════════".cyan()
    );
    println!("{}", format!("📁 {}", report.path).cyan());
    println!(
        "{}",
        "════════════════════════════════════════════════════════════".cyan()
    );

    match report.status {
        LockfileStatus::Error => {
            if let Some(error) = &report.error {
                println!("  {}", format!("✗ {}", error).red());
            }

            return;
        }
        LockfileStatus::NotFound => {
            println!(
                "  {}",
                format!("Package {}@{} not found", package_name, package_version).red()
            );

            return;
        }
        LockfileStatus::Found => {}
    }

    println!(
        "  {}",
        format!("✓ Found {}@{}", package_name, package_version).green()
    );

    for group in group_chains_by_owner(&report.chains) {
        println!("\n Owner:");
        match &group.owner {
            Some(owner) => println!(
                "  {} from {}, requested as {}",
                owner.name, owner.dependency_type, owner.requested_as
            ),
            None => println!("  Not declared in package.json"),
        }

        for chain_index in group.chain_indexes {
            let chain = &report.chains[chain_index];
            println!("\n  {}", format!("── Chain {} ──", chain_index + 1).cyan());
            print!("  ");
            format_chain(&chain.links, package_name, package_version);

            for warning in &chain.warnings {
                println!("  {}", format!("⚠ {}", warning).yellow());
            }

            if let Some(recommended) = &chain.recommended {
                println!("\n Fix path:");
                for step in &chain.fix_path {
                    println!("  {} >= {}", step.package, step.minimum_version);
                }

                println!(
                    "  {}",
                    format!(
                        "→ Recommended: Update {} to >= {}",
                        recommended.package, recommended.minimum_version
                    )
                    .green()
                    .bold()
                );
            } else {
                println!("  {}", "✗ No fix available for this chain".red());
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let spec = match parse_package(&cli.package) {
        Ok(spec) => spec,
        Err(e) => {
            match e {
                ParseError::Empty => eprintln!("No package provided."),
                ParseError::MissingVersion => eprintln!("Missing version. Use: pharos pkg@1.2.3"),
                ParseError::InvalidVersion(v) => eprintln!(
                    "Invalid version '{}'. Please provide an exact semver version (e.g. 1.2.3)",
                    v
                ),
            }
            std::process::exit(1);
        }
    };

    let lockfiles = find_lockfiles(&cli.path, cli.recursive);
    if lockfiles.is_empty() {
        eprintln!("No lockfiles found in {}", cli.path);
        std::process::exit(2);
    }

    if cli.json {
        let mut registry_cache: RegistryCache = HashMap::new();
        let lockfile_reports: Vec<LockfileReport> = lockfiles
            .iter()
            .map(|(lockfile_type, path)| {
                analyze_lockfile(
                    lockfile_type,
                    path,
                    spec.name,
                    spec.version,
                    &mut registry_cache,
                )
            })
            .collect();
        let report = Report {
            package: ReportPackage {
                name: spec.name.to_string(),
                version: spec.version.to_string(),
            },
            lockfiles: lockfile_reports,
        };

        match serde_json::to_string_pretty(&report) {
            Ok(output) => println!("{}", output),
            Err(err) => {
                eprintln!("Failed to serialize JSON output: {}", err);
                std::process::exit(1);
            }
        }

        return;
    }

    let mut registry_cache: RegistryCache = HashMap::new();
    for (lockfile_type, path) in &lockfiles {
        let report = analyze_lockfile(
            lockfile_type,
            path,
            spec.name,
            spec.version,
            &mut registry_cache,
        );
        print_lockfile_report(&report, spec.name, spec.version);
    }
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
