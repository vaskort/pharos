use super::*;
use crate::registry::{RegistryCache, RegistryResponse, VersionInfo};
use crate::search::{ChainLink, DependencyChain, DependencyKind};
use std::collections::HashMap;

fn version_info(dependency: &str, requested_as: &str) -> VersionInfo {
    VersionInfo {
        dependencies: Some(HashMap::from([(
            dependency.to_string(),
            requested_as.to_string(),
        )])),
        optional_dependencies: None,
    }
}

fn package(versions: &[(&str, VersionInfo)]) -> RegistryResponse {
    RegistryResponse {
        versions: versions
            .iter()
            .map(|(version, info)| ((*version).to_string(), info.clone()))
            .collect(),
    }
}

fn link(name: &str, version: &str, requested_as: &str) -> ChainLink {
    ChainLink {
        node_id: 0,
        name: name.to_string(),
        version: version.to_string(),
        locator: format!("{}@{}", name, version),
        requested_as: requested_as.to_string(),
        dependency_kind: DependencyKind::Normal,
    }
}

fn chain(links: Vec<ChainLink>) -> DependencyChain {
    DependencyChain {
        target_node_id: 0,
        target_locator: "target@1.0.0".to_string(),
        links,
        warnings: Vec::new(),
    }
}

fn owner(name: &str, requested_as: &str) -> DependencyOwner {
    DependencyOwner {
        name: name.to_string(),
        dependency_type: "dependencies".to_string(),
        requested_as: requested_as.to_string(),
    }
}

#[test]
fn exact_fixed_version_becomes_minimum_safe_range() {
    let safe = SafeRange::parse("2.0.0", "1.0.0").unwrap();

    assert_eq!(safe.normalized(), ">=2.0.0");
}

#[test]
fn accepts_an_explicit_safe_range() {
    let safe = SafeRange::parse(">=2.0.0 <3", "1.0.0").unwrap();

    assert_eq!(safe.normalized(), ">=2.0.0 <3");
}

#[test]
fn rejects_a_safe_range_that_contains_the_vulnerable_version() {
    let error = SafeRange::parse(">=1.0.0", "1.0.0").unwrap_err();

    assert!(error.contains("contains vulnerable version"));
}

#[test]
fn verified_path_never_recommends_an_older_parent() {
    let safe = SafeRange::parse("2.0.0", "1.0.0").unwrap();
    let cache = RegistryCache::from([
        (
            "target".to_string(),
            package(&[("2.0.0", VersionInfo::default())]),
        ),
        (
            "parent".to_string(),
            package(&[
                ("1.0.0", version_info("target", "^2.0.0")),
                ("2.0.0", version_info("target", "^1.0.0")),
                ("3.0.0", version_info("target", "^2.0.0")),
            ]),
        ),
    ]);
    let dependency_chain = chain(vec![link("parent", "2.0.0", "^1.0.0")]);

    let result = build_remediation(
        &dependency_chain,
        "target",
        "1.0.0",
        Some(&safe),
        Some(&owner("parent", "^2.0.0")),
        PackageManager::Npm,
        &cache,
    );

    assert_eq!(result.status, RemediationStatus::SemverVerified);
    assert_eq!(result.fix_path[0].minimum_version, "3.0.0");
}

#[test]
fn overlapping_but_not_safe_parent_range_is_not_verified() {
    let safe = SafeRange::parse(">=2.0.0 <3", "1.0.0").unwrap();
    let cache = RegistryCache::from([
        (
            "target".to_string(),
            package(&[("2.0.0", VersionInfo::default())]),
        ),
        (
            "parent".to_string(),
            package(&[("2.0.0", version_info("target", ">=1.0.0 <3"))]),
        ),
    ]);

    let result = build_remediation(
        &chain(vec![link("parent", "1.0.0", ">=1.0.0 <3")]),
        "target",
        "1.0.0",
        Some(&safe),
        Some(&owner("parent", "^1.0.0")),
        PackageManager::Npm,
        &cache,
    );

    assert_eq!(result.primary_action.unwrap().kind, ActionKind::Override);
    assert!(
        result
            .alternatives
            .iter()
            .any(|action| action.kind == ActionKind::LockfileRefresh)
    );
}

#[test]
fn an_or_range_is_verified_only_when_every_branch_is_safe() {
    let safe = SafeRange::parse(">=2.0.0 <3", "1.0.0").unwrap();
    let cache = RegistryCache::from([
        (
            "target".to_string(),
            package(&[("2.0.0", VersionInfo::default())]),
        ),
        (
            "parent".to_string(),
            package(&[("2.0.0", version_info("target", "^2.0.0 || ^1.0.0"))]),
        ),
    ]);

    let result = build_remediation(
        &chain(vec![link("parent", "1.0.0", "^1.0.0")]),
        "target",
        "1.0.0",
        Some(&safe),
        Some(&owner("parent", "^1.0.0")),
        PackageManager::Npm,
        &cache,
    );

    assert_eq!(result.primary_action.unwrap().kind, ActionKind::Override);
}

#[test]
fn no_fixed_range_produces_an_unverified_candidate() {
    let cache = RegistryCache::from([(
        "parent".to_string(),
        package(&[
            ("1.0.0", version_info("target", "^1.0.0")),
            ("2.0.0", version_info("target", "^2.0.0")),
        ]),
    )]);

    let result = build_remediation(
        &chain(vec![link("parent", "1.0.0", "^1.0.0")]),
        "target",
        "1.0.0",
        None,
        Some(&owner("parent", "^1.0.0")),
        PackageManager::Npm,
        &cache,
    );

    assert_eq!(result.status, RemediationStatus::Candidate);
    assert_eq!(result.fix_path[0].minimum_version, "2.0.0");
}

#[test]
fn verifies_every_step_in_a_multi_hop_chain() {
    let safe = SafeRange::parse("2.0.0", "1.0.0").unwrap();
    let cache = RegistryCache::from([
        (
            "target".to_string(),
            package(&[("2.0.0", VersionInfo::default())]),
        ),
        (
            "intermediate".to_string(),
            package(&[("2.0.0", version_info("target", "^2.0.0"))]),
        ),
        (
            "root".to_string(),
            package(&[("2.0.0", version_info("intermediate", "^2.0.0"))]),
        ),
    ]);
    let dependency_chain = chain(vec![
        link("intermediate", "1.0.0", "^1.0.0"),
        link("root", "1.0.0", "^1.0.0"),
    ]);

    let result = build_remediation(
        &dependency_chain,
        "target",
        "1.0.0",
        Some(&safe),
        Some(&owner("root", "^1.0.0")),
        PackageManager::Npm,
        &cache,
    );

    assert_eq!(
        result.fix_path,
        vec![
            FixStep {
                package: "intermediate".to_string(),
                minimum_version: "2.0.0".to_string(),
            },
            FixStep {
                package: "root".to_string(),
                minimum_version: "2.0.0".to_string(),
            },
        ]
    );
    assert_eq!(result.primary_action.unwrap().kind, ActionKind::OwnerUpdate);
}

#[test]
fn does_not_recommend_a_prerelease_as_the_safe_target() {
    let safe = SafeRange::parse("2.0.0", "1.0.0").unwrap();
    let cache = RegistryCache::from([(
        "target".to_string(),
        package(&[("2.0.0-beta.1", VersionInfo::default())]),
    )]);

    let result = build_remediation(
        &chain(Vec::new()),
        "target",
        "1.0.0",
        Some(&safe),
        Some(&owner("target", "1.0.0")),
        PackageManager::Npm,
        &cache,
    );

    assert_eq!(result.status, RemediationStatus::Unavailable);
}
