mod lockfile;
mod manifest;
mod registry;
mod remediation;
mod search;

use clap::{Parser, error::ErrorKind};
use colored::Colorize;
use lockfile::{find_lockfiles, parse_dependency_entries, parse_lockfile};
use manifest::{ManifestDependency, read_package_json_dependencies};
use remediation::{
    DependencyOwner, FixStep, PackageManager, RemediationPlan, RemediationStatus, SafeRange,
    build_remediation,
};
use search::{ChainLink, DependencyChain, find_dependency_chains, package_exists};
use serde::Serialize;
use std::path::Path;

use crate::{
    lockfile::LockFileType,
    registry::{RegistryCache, find_parent_versions},
};

#[derive(Parser)]
#[command(
    name = "pharos-cli",
    bin_name = "pharos-cli",
    author,
    version,
    about,
    after_help = "Examples:\n  pharos-cli qs@6.13.0 --path .\n  pharos-cli qs@6.13.0 --recursive --json"
)]
struct Cli {
    /// Package to search for in the format name@version (e.g. qs@6.13.0)
    #[arg(value_name = "PACKAGE@VERSION")]
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

    /// Minimum fixed version or complete safe semver range
    #[arg(long, value_name = "VERSION_OR_RANGE")]
    fixed: Option<String>,

    /// Skip npm registry lookups and print dependency chains only
    #[arg(long)]
    no_registry: bool,
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
    schema_version: u8,
    package: ReportPackage,
    lockfiles: Vec<LockfileReport>,
}

#[derive(Debug, Serialize)]
struct ReportPackage {
    name: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fixed_range: Option<String>,
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
    target_locator: String,
    links: Vec<ChainLinkReport>,
    owner: Option<DependencyOwner>,
    fix_path: Vec<FixStep>,
    recommended: Option<FixStep>,
    remediation: RemediationPlan,
    warnings: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct ChainOwnerGroup {
    owner: Option<DependencyOwner>,
    chain_indexes: Vec<usize>,
}

struct RemediationRequest<'a> {
    package_name: &'a str,
    package_version: &'a str,
    safe_range: Option<&'a SafeRange>,
    package_manager: PackageManager,
    no_registry: bool,
}

#[derive(Clone, Debug, Serialize)]
struct ChainLinkReport {
    name: String,
    version: String,
    locator: String,
    requested_as: String,
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

fn lockfile_type_name(lockfile_type: &LockFileType) -> String {
    match lockfile_type {
        LockFileType::Yarn => "yarn".to_string(),
        LockFileType::Npm => "npm".to_string(),
    }
}

fn package_manager(lockfile_type: &LockFileType, content: &str) -> PackageManager {
    match lockfile_type {
        LockFileType::Npm => PackageManager::Npm,
        LockFileType::Yarn
            if content
                .lines()
                .any(|line| line.contains("yarn lockfile v1")) =>
        {
            PackageManager::YarnClassic
        }
        LockFileType::Yarn => PackageManager::YarnModern,
    }
}

fn report_chain(
    chain: &DependencyChain,
    registry_cache: &RegistryCache,
    manifest_dependencies: &[ManifestDependency],
    request: &RemediationRequest<'_>,
) -> ChainReport {
    let owner = find_chain_owner(&chain.links, request.package_name, manifest_dependencies);
    let remediation = if request.no_registry {
        RemediationPlan {
            status: RemediationStatus::Unavailable,
            primary_action: None,
            alternatives: Vec::new(),
            fix_path: Vec::new(),
            warnings: vec!["Registry lookups disabled by --no-registry".to_string()],
        }
    } else {
        build_remediation(
            chain,
            request.package_name,
            request.package_version,
            request.safe_range,
            owner.as_ref(),
            request.package_manager,
            registry_cache,
        )
    };
    let fix_path = remediation.fix_path.clone();
    let recommended = fix_path.last().cloned();
    let mut warnings = chain.warnings.clone();
    warnings.extend(remediation.warnings.clone());
    let mut registry_packages = chain
        .links
        .iter()
        .map(|link| link.name.as_str())
        .collect::<Vec<_>>();
    if request.safe_range.is_some() {
        registry_packages.push(request.package_name);
    }
    registry_packages.sort_unstable();
    registry_packages.dedup();
    for package in registry_packages {
        if let Some(error) = registry_cache.error(package) {
            warnings.push(format!("Registry lookup failed for {}: {}", package, error));
        }
    }

    ChainReport {
        target_locator: chain.target_locator.clone(),
        links: chain
            .links
            .iter()
            .map(|chain_link| ChainLinkReport {
                name: chain_link.name.clone(),
                version: chain_link.version.clone(),
                locator: chain_link.locator.clone(),
                requested_as: chain_link.requested_as.clone(),
            })
            .collect(),
        owner,
        recommended,
        fix_path,
        remediation,
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
    safe_range: Option<&SafeRange>,
    no_registry: bool,
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
    let graph = match parse_dependency_entries(lockfile_type, &lockfile_content) {
        Ok(graph) => graph,
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

    if !package_exists(&graph, package_name, package_version) {
        return LockfileReport {
            path: path_display,
            lockfile_type: lockfile_type_name,
            status: LockfileStatus::NotFound,
            chains: Vec::new(),
            error: None,
        };
    }

    let chains = find_dependency_chains(&graph, package_name, package_version);
    if !no_registry {
        let additional_packages = safe_range.map(|_| vec![package_name]).unwrap_or_default();
        find_parent_versions(&chains, &additional_packages, registry_cache);
    }
    let (manifest_dependencies, manifest_warning) = manifest_dependencies_for_lockfile(path);
    let package_manager = package_manager(lockfile_type, &lockfile_content);
    let remediation_request = RemediationRequest {
        package_name,
        package_version,
        safe_range,
        package_manager,
        no_registry,
    };
    let chains = chains
        .iter()
        .map(|chain| {
            let mut report = report_chain(
                chain,
                registry_cache,
                &manifest_dependencies,
                &remediation_request,
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
            println!("  Locator: {}", chain.target_locator);
            print!("  ");
            format_chain(&chain.links, package_name, package_version);

            for warning in &chain.warnings {
                println!("  {}", format!("⚠ {}", warning).yellow());
            }

            if !chain.fix_path.is_empty() {
                let heading = match chain.remediation.status {
                    RemediationStatus::SemverVerified => "Verified remediation:",
                    RemediationStatus::Candidate => {
                        "Candidate path (not verified; pass --fixed to verify):"
                    }
                    RemediationStatus::Unavailable => "Remediation unavailable:",
                };
                println!("\n {}", heading);
                for step in &chain.fix_path {
                    println!("  {} >= {}", step.package, step.minimum_version);
                }
            }

            if let Some(action) = &chain.remediation.primary_action {
                println!(
                    "  {}",
                    format!(
                        "→ {}: {} {} → {}",
                        action.verification.label(),
                        action.package,
                        action.current_version.as_deref().unwrap_or("unknown"),
                        action.target_version
                    )
                    .green()
                    .bold()
                );
                for instruction in &action.instructions {
                    println!("    {}", instruction);
                }
                for alternative in &chain.remediation.alternatives {
                    println!(
                        "  Alternative ({}): {} {} → {}",
                        alternative.verification.label(),
                        alternative.kind.label(),
                        alternative.package,
                        alternative.target_version
                    );
                    for instruction in &alternative.instructions {
                        println!("    {}", instruction);
                    }
                }
            } else if chain.remediation.status == RemediationStatus::Unavailable {
                println!("  {}", "✗ No remediation available for this chain".red());
            }
        }
    }
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) if err.kind() == ErrorKind::MissingRequiredArgument => {
            eprintln!("error: missing package to analyze");
            eprintln!();
            eprintln!("Usage: pharos-cli <PACKAGE@VERSION> [OPTIONS]");
            eprintln!();
            eprintln!("Example:");
            eprintln!("  pharos-cli qs@6.13.0 --path .");
            eprintln!();
            eprintln!("For more information, try 'pharos-cli --help'.");
            std::process::exit(2);
        }
        Err(err) => err.exit(),
    };

    let spec = match parse_package(&cli.package) {
        Ok(spec) => spec,
        Err(e) => {
            match e {
                ParseError::Empty => eprintln!("No package provided."),
                ParseError::MissingVersion => {
                    eprintln!("Missing version. Use: pharos-cli pkg@1.2.3")
                }
                ParseError::InvalidVersion(v) => eprintln!(
                    "Invalid version '{}'. Please provide an exact semver version (e.g. 1.2.3)",
                    v
                ),
            }
            std::process::exit(1);
        }
    };
    let safe_range = match cli.fixed.as_deref() {
        Some(fixed) => match SafeRange::parse(fixed, spec.version) {
            Ok(range) => Some(range),
            Err(err) => {
                eprintln!("Invalid --fixed value: {}", err);
                std::process::exit(1);
            }
        },
        None => None,
    };

    let lockfiles = find_lockfiles(&cli.path, cli.recursive);
    if lockfiles.is_empty() {
        eprintln!("No lockfiles found in {}", cli.path);
        std::process::exit(2);
    }

    if cli.json {
        let mut registry_cache = RegistryCache::default();
        let lockfile_reports: Vec<LockfileReport> = lockfiles
            .iter()
            .map(|(lockfile_type, path)| {
                analyze_lockfile(
                    lockfile_type,
                    path,
                    spec.name,
                    spec.version,
                    safe_range.as_ref(),
                    cli.no_registry,
                    &mut registry_cache,
                )
            })
            .collect();
        let report = Report {
            schema_version: 1,
            package: ReportPackage {
                name: spec.name.to_string(),
                version: spec.version.to_string(),
                fixed_range: safe_range
                    .as_ref()
                    .map(|range| range.normalized().to_string()),
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

    let mut registry_cache = RegistryCache::default();
    for (lockfile_type, path) in &lockfiles {
        let report = analyze_lockfile(
            lockfile_type,
            path,
            spec.name,
            spec.version,
            safe_range.as_ref(),
            cli.no_registry,
            &mut registry_cache,
        );
        print_lockfile_report(&report, spec.name, spec.version);
    }
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
