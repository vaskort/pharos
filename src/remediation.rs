use crate::registry::{RegistryCache, VersionInfo};
use crate::search::{DependencyChain, DependencyKind};
use node_semver::{Range, Version};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct SafeRange {
    raw: String,
    normalized: String,
    range: Range,
}

impl SafeRange {
    pub fn parse(input: &str, vulnerable_version: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("fixed version or range cannot be empty".to_string());
        }

        let (normalized, range) = match Version::parse(input) {
            Ok(version) => {
                let normalized = format!(">={}", version);
                let range = Range::parse(&normalized).map_err(|err| err.to_string())?;
                (normalized, range)
            }
            Err(_) => (
                input.to_string(),
                Range::parse(input).map_err(|err| format!("invalid fixed range: {}", err))?,
            ),
        };
        let vulnerable = Version::parse(vulnerable_version)
            .map_err(|err| format!("invalid vulnerable version: {}", err))?;
        if range.satisfies(&vulnerable) {
            return Err(format!(
                "fixed range '{}' contains vulnerable version {}",
                normalized, vulnerable_version
            ));
        }

        Ok(Self {
            raw: input.to_string(),
            normalized,
            range,
        })
    }

    pub fn normalized(&self) -> &str {
        &self.normalized
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManager {
    Npm,
    YarnClassic,
    YarnModern,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DependencyOwner {
    pub name: String,
    pub dependency_type: String,
    pub requested_as: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStatus {
    SemverVerified,
    Candidate,
    Unavailable,
}

impl RemediationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::SemverVerified => "semver verified",
            Self::Candidate => "candidate",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    DirectUpdate,
    OwnerUpdate,
    LockfileRefresh,
    Override,
}

impl ActionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::DirectUpdate => "direct update",
            Self::OwnerUpdate => "owner update",
            Self::LockfileRefresh => "lockfile refresh",
            Self::Override => "override",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RemediationAction {
    pub kind: ActionKind,
    pub verification: RemediationStatus,
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_version: Option<String>,
    pub target_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_as: Option<String>,
    pub instructions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FixStep {
    pub package: String,
    pub minimum_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RemediationPlan {
    pub status: RemediationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_action: Option<RemediationAction>,
    pub alternatives: Vec<RemediationAction>,
    #[serde(skip)]
    pub fix_path: Vec<FixStep>,
    #[serde(skip)]
    pub warnings: Vec<String>,
}

pub fn build_remediation(
    chain: &DependencyChain,
    package_name: &str,
    package_version: &str,
    safe_range: Option<&SafeRange>,
    owner: Option<&DependencyOwner>,
    package_manager: PackageManager,
    registry_cache: &RegistryCache,
) -> RemediationPlan {
    match safe_range {
        Some(safe_range) => build_verified_remediation(
            chain,
            package_name,
            package_version,
            safe_range,
            owner,
            package_manager,
            registry_cache,
        ),
        None => build_candidate_remediation(
            chain,
            package_name,
            package_version,
            owner,
            package_manager,
            registry_cache,
        ),
    }
}

fn build_verified_remediation(
    chain: &DependencyChain,
    package_name: &str,
    package_version: &str,
    safe_range: &SafeRange,
    owner: Option<&DependencyOwner>,
    package_manager: PackageManager,
    registry_cache: &RegistryCache,
) -> RemediationPlan {
    let Some(safe_version) =
        smallest_published_version(registry_cache, package_name, &safe_range.range)
    else {
        return unavailable(format!(
            "No stable published {} version satisfies {}",
            package_name, safe_range.normalized
        ));
    };

    let mut alternatives = lockfile_refresh_action(
        chain,
        package_name,
        package_version,
        &safe_version,
        safe_range,
        owner,
        package_manager,
    )
    .into_iter()
    .collect::<Vec<_>>();

    if chain.links.is_empty() {
        if let Some(owner) = owner {
            let requested_as =
                proposed_range(&owner.requested_as, &safe_version, &safe_range.range);
            let action = manifest_action(
                ActionKind::DirectUpdate,
                RemediationStatus::SemverVerified,
                package_name,
                package_version,
                &safe_version,
                owner,
                requested_as,
                package_manager,
                Some(safe_range),
            );
            return RemediationPlan {
                status: RemediationStatus::SemverVerified,
                primary_action: Some(action),
                alternatives,
                fix_path: vec![FixStep {
                    package: package_name.to_string(),
                    minimum_version: safe_version,
                }],
                warnings: Vec::new(),
            };
        }

        let action = override_action(
            package_name,
            package_version,
            &safe_version,
            package_manager,
            safe_range,
        );
        return RemediationPlan {
            status: RemediationStatus::SemverVerified,
            primary_action: Some(action),
            alternatives,
            fix_path: Vec::new(),
            warnings: vec!["Target is not declared in the sibling package.json".to_string()],
        };
    }

    let mut child_name = package_name.to_string();
    let mut required_range = safe_range.range.clone();
    let mut fix_path = Vec::new();
    let mut failure = None;

    for link in &chain.links {
        let Some(candidate) = smallest_safe_parent_version(
            registry_cache,
            &link.name,
            &link.version,
            &child_name,
            link.dependency_kind,
            &required_range,
        ) else {
            failure = Some(format!(
                "No newer stable {} version constrains {} to {}",
                link.name, child_name, required_range
            ));
            break;
        };

        fix_path.push(FixStep {
            package: link.name.clone(),
            minimum_version: candidate.clone(),
        });
        child_name = link.name.clone();
        required_range = Range::parse(format!(">={}", candidate))
            .expect("a parsed registry version must form a valid minimum range");
    }

    if failure.is_none()
        && fix_path.len() == chain.links.len()
        && let (Some(owner), Some(last_step)) = (owner, fix_path.last())
    {
        let owner_required = Range::parse(format!(">={}", last_step.minimum_version))
            .expect("a parsed registry version must form a valid owner range");
        let requested_as = proposed_range(
            &owner.requested_as,
            &last_step.minimum_version,
            &owner_required,
        );
        let action = manifest_action(
            ActionKind::OwnerUpdate,
            RemediationStatus::SemverVerified,
            &owner.name,
            &chain.links.last().expect("non-empty chain").version,
            &last_step.minimum_version,
            owner,
            requested_as,
            package_manager,
            Some(safe_range),
        );
        return RemediationPlan {
            status: RemediationStatus::SemverVerified,
            primary_action: Some(action),
            alternatives,
            fix_path,
            warnings: Vec::new(),
        };
    }

    let override_action = override_action(
        package_name,
        package_version,
        &safe_version,
        package_manager,
        safe_range,
    );
    if alternatives
        .iter()
        .all(|action| action.kind != ActionKind::Override)
    {
        // Override is the verified fallback and therefore becomes the primary action.
    }
    RemediationPlan {
        status: RemediationStatus::SemverVerified,
        primary_action: Some(override_action),
        alternatives: std::mem::take(&mut alternatives),
        fix_path,
        warnings: failure.into_iter().collect(),
    }
}

fn build_candidate_remediation(
    chain: &DependencyChain,
    package_name: &str,
    package_version: &str,
    owner: Option<&DependencyOwner>,
    package_manager: PackageManager,
    registry_cache: &RegistryCache,
) -> RemediationPlan {
    if chain.links.is_empty() {
        return unavailable("Pass --fixed to verify a direct dependency update".to_string());
    }

    let mut child_name = package_name.to_string();
    let mut child_version = package_version.to_string();
    let mut fix_path = Vec::new();
    let mut warnings = Vec::new();

    for link in &chain.links {
        let Some(candidate) = smallest_candidate_parent_version(
            registry_cache,
            &link.name,
            &link.version,
            &child_name,
            &child_version,
            link.dependency_kind,
        ) else {
            warnings.push(format!(
                "No newer {} version excludes {}@{}",
                link.name, child_name, child_version
            ));
            break;
        };
        fix_path.push(FixStep {
            package: link.name.clone(),
            minimum_version: candidate.clone(),
        });
        child_name = link.name.clone();
        child_version = candidate;
    }

    let Some(last_step) = fix_path.last() else {
        let mut plan = unavailable("No candidate parent upgrade was found".to_string());
        plan.warnings.extend(warnings);
        return plan;
    };
    let action_owner = owner.cloned().unwrap_or_else(|| DependencyOwner {
        name: last_step.package.clone(),
        dependency_type: "unknown".to_string(),
        requested_as: last_step.minimum_version.clone(),
    });
    let required_range = Range::parse(format!(">={}", last_step.minimum_version))
        .expect("a parsed registry version must form a valid candidate range");
    let requested_as = proposed_range(
        &action_owner.requested_as,
        &last_step.minimum_version,
        &required_range,
    );
    let current_version = chain
        .links
        .last()
        .map(|link| link.version.as_str())
        .unwrap_or(package_version);
    let action = manifest_action(
        ActionKind::OwnerUpdate,
        RemediationStatus::Candidate,
        &last_step.package,
        current_version,
        &last_step.minimum_version,
        &action_owner,
        requested_as,
        package_manager,
        None,
    );

    RemediationPlan {
        status: RemediationStatus::Candidate,
        primary_action: Some(action),
        alternatives: Vec::new(),
        fix_path,
        warnings,
    }
}

fn smallest_published_version(
    cache: &RegistryCache,
    package_name: &str,
    safe_range: &Range,
) -> Option<String> {
    sorted_stable_versions(cache.get(package_name)?)
        .into_iter()
        .find(|(_, version)| safe_range.satisfies(version))
        .map(|(raw, _)| raw)
}

fn smallest_safe_parent_version(
    cache: &RegistryCache,
    parent_name: &str,
    installed_parent_version: &str,
    child_name: &str,
    dependency_kind: DependencyKind,
    required_range: &Range,
) -> Option<String> {
    let installed = Version::parse(installed_parent_version).ok()?;
    sorted_stable_versions(cache.get(parent_name)?)
        .into_iter()
        .filter(|(_, version)| version > &installed)
        .find_map(|(raw, _)| {
            let info = cache.get(parent_name)?.versions.get(&raw)?;
            let requested = dependency_requirement(info, child_name, dependency_kind)?;
            range_is_subset(required_range, requested).then_some(raw)
        })
}

fn range_is_subset(required_range: &Range, requested: &str) -> bool {
    // `node-semver` represents OR ranges as multiple bound sets. Its `allows_all`
    // answers whether any bound-set pair is contained, so check every requested
    // OR branch separately to avoid accepting `safe || unsafe` dependencies.
    requested.split("||").all(|branch| {
        Range::parse(branch.trim())
            .is_ok_and(|branch_range| required_range.allows_all(&branch_range))
    })
}

fn smallest_candidate_parent_version(
    cache: &RegistryCache,
    parent_name: &str,
    installed_parent_version: &str,
    child_name: &str,
    vulnerable_child_version: &str,
    dependency_kind: DependencyKind,
) -> Option<String> {
    let installed = Version::parse(installed_parent_version).ok()?;
    let vulnerable = Version::parse(vulnerable_child_version).ok()?;
    sorted_stable_versions(cache.get(parent_name)?)
        .into_iter()
        .filter(|(_, version)| version > &installed)
        .find_map(|(raw, _)| {
            let info = cache.get(parent_name)?.versions.get(&raw)?;
            let requested = dependency_requirement(info, child_name, dependency_kind)?;
            let requested_range = Range::parse(requested).ok()?;
            (!requested_range.satisfies(&vulnerable)).then_some(raw)
        })
}

fn sorted_stable_versions(response: &crate::registry::RegistryResponse) -> Vec<(String, Version)> {
    let mut versions = response
        .versions
        .keys()
        .filter_map(|raw| {
            Version::parse(raw)
                .ok()
                .map(|version| (raw.clone(), version))
        })
        .filter(|(_, version)| !version.is_prerelease())
        .collect::<Vec<_>>();
    versions.sort_by(|(_, left), (_, right)| left.cmp(right));
    versions
}

fn dependency_requirement<'a>(
    info: &'a VersionInfo,
    dependency_name: &str,
    kind: DependencyKind,
) -> Option<&'a str> {
    let dependencies = match kind {
        DependencyKind::Normal => info.dependencies.as_ref(),
        DependencyKind::Optional => info.optional_dependencies.as_ref(),
    }?;
    dependencies.get(dependency_name).map(String::as_str)
}

fn proposed_range(current: &str, target: &str, required: &Range) -> String {
    for prefix in ["^", "~"] {
        if current.starts_with(prefix) {
            let proposal = format!("{}{}", prefix, target);
            if Range::parse(&proposal)
                .is_ok_and(|proposal_range| required.allows_all(&proposal_range))
            {
                return proposal;
            }
        }
    }
    target.to_string()
}

#[allow(clippy::too_many_arguments)]
fn manifest_action(
    kind: ActionKind,
    verification: RemediationStatus,
    package: &str,
    current_version: &str,
    target_version: &str,
    owner: &DependencyOwner,
    requested_as: String,
    package_manager: PackageManager,
    safe_range: Option<&SafeRange>,
) -> RemediationAction {
    let install_command = match package_manager {
        PackageManager::Npm => "npm install",
        PackageManager::YarnClassic | PackageManager::YarnModern => "yarn install",
    };
    let mut instructions = vec![
        format!(
            "Change package.json {}.{} from \"{}\" to \"{}\"",
            owner.dependency_type, owner.name, owner.requested_as, requested_as
        ),
        format!("Run {}", install_command),
    ];
    if let Some(safe_range) = safe_range {
        instructions.push(format!(
            "Rerun pharos {}@{} --fixed '{}'",
            package, current_version, safe_range.raw
        ));
    } else {
        instructions.push("Rerun Pharos with --fixed to verify the result".to_string());
    }

    RemediationAction {
        kind,
        verification,
        package: package.to_string(),
        current_version: Some(current_version.to_string()),
        target_version: target_version.to_string(),
        manifest_section: Some(owner.dependency_type.clone()),
        requested_as: Some(requested_as),
        instructions,
    }
}

fn override_action(
    package_name: &str,
    package_version: &str,
    safe_version: &str,
    package_manager: PackageManager,
    safe_range: &SafeRange,
) -> RemediationAction {
    let (field, install) = match package_manager {
        PackageManager::Npm => ("overrides", "npm install"),
        PackageManager::YarnClassic | PackageManager::YarnModern => ("resolutions", "yarn install"),
    };
    RemediationAction {
        kind: ActionKind::Override,
        verification: RemediationStatus::SemverVerified,
        package: package_name.to_string(),
        current_version: Some(package_version.to_string()),
        target_version: safe_version.to_string(),
        manifest_section: Some(field.to_string()),
        requested_as: Some(safe_version.to_string()),
        instructions: vec![
            format!(
                "Add to package.json: \"{}\": {{ \"{}\": \"{}\" }}",
                field, package_name, safe_version
            ),
            format!("Run {}", install),
            format!(
                "Rerun pharos {}@{} --fixed '{}' and run the project test suite",
                package_name, package_version, safe_range.raw
            ),
        ],
    }
}

fn lockfile_refresh_action(
    chain: &DependencyChain,
    package_name: &str,
    package_version: &str,
    safe_version: &str,
    safe_range: &SafeRange,
    owner: Option<&DependencyOwner>,
    package_manager: PackageManager,
) -> Option<RemediationAction> {
    let requested_as = chain
        .links
        .first()
        .map(|link| link.requested_as.as_str())
        .or_else(|| owner.map(|owner| owner.requested_as.as_str()))?;
    let requested_range = Range::parse(requested_as).ok()?;
    if !safe_range.range.allows_any(&requested_range) {
        return None;
    }
    let command = match package_manager {
        PackageManager::Npm => format!("npm update {} --package-lock-only", package_name),
        PackageManager::YarnClassic => format!("yarn upgrade {}", package_name),
        PackageManager::YarnModern => format!("yarn up -R {}@{}", package_name, safe_version),
    };
    Some(RemediationAction {
        kind: ActionKind::LockfileRefresh,
        verification: RemediationStatus::Candidate,
        package: package_name.to_string(),
        current_version: Some(package_version.to_string()),
        target_version: safe_version.to_string(),
        manifest_section: None,
        requested_as: Some(requested_as.to_string()),
        instructions: vec![
            format!("Run {}", command),
            format!(
                "Rerun pharos {}@{} --fixed '{}' to verify the resolved lockfile",
                package_name, package_version, safe_range.raw
            ),
        ],
    })
}

fn unavailable(warning: String) -> RemediationPlan {
    RemediationPlan {
        status: RemediationStatus::Unavailable,
        primary_action: None,
        alternatives: Vec::new(),
        fix_path: Vec::new(),
        warnings: vec![warning],
    }
}

#[cfg(test)]
#[path = "remediation_tests.rs"]
mod tests;
