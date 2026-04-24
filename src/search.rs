/// A package reference inside a lockfile.
///
/// For dependencies this is the requested range from the parent package.
/// For descriptors this is the range that resolved to the entry's concrete version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencySpec {
    pub name: String,
    pub requested_as: String,
}

/// A resolved package entry from a lockfile, independent of the lockfile format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyEntry {
    pub name: String,
    pub version: String,
    pub descriptors: Vec<DependencySpec>,
    pub dependencies: Vec<DependencySpec>,
}

/// A single link in a dependency chain, representing one package
/// in the path from a direct dependency down to the vulnerable package.
///
/// For example, if `pkg-a` depends on `pkg-b` which depends on `pkg-c` (vulnerable),
/// the chain would be: `[ChainLink(pkg-a), ChainLink(pkg-b)]`.
#[derive(Clone, Debug)]
pub struct ChainLink {
    pub name: String,
    pub version: String,
    /// The version range the parent asked for (e.g. "^4.0.0"), not the resolved version.
    pub requested_as: String,
}

/// Checks if a specific package at a specific version exists in the lockfile entries.
///
/// # Arguments
/// * `entries` - The normalized lockfile entries
/// * `package_name` - The package name to search for
/// * `package_version` - The exact version to match
pub fn package_exists(
    entries: &[DependencyEntry],
    package_name: &str,
    package_version: &str,
) -> bool {
    for entry in entries.iter() {
        if entry.name == package_name && entry.version == package_version {
            return true;
        }
    }
    false
}

/// Finds all dependency chains that lead to a specific package version.
///
/// Starting from the target package, it walks *up* the dependency tree
/// to find every path from a root dependency down to the target.
/// Each chain is a `Vec<ChainLink>` representing one such path.
///
/// For example, if `pkg-c@1.0.0` is vulnerable and two packages depend on it:
/// - `pkg-a → pkg-b → pkg-c`
/// - `pkg-d → pkg-c`
///
/// This returns two chains: `[[pkg-a, pkg-b], [pkg-d]]`.
///
/// # Arguments
/// * `entries` - The normalized lockfile entries
/// * `package_name` - The target package name to trace chains for
/// * `package_version` - The target package version
///
/// # Returns
/// A `Vec<Vec<ChainLink>>` — e.g. `[[pkg-a, pkg-b], [pkg-d]]`
/// where each inner vec is one path leading to the target package.
/// Returns empty if the package is not found.
pub fn find_dependency_chains(
    entries: &[DependencyEntry],
    package_name: &str,
    package_version: &str,
) -> Vec<Vec<ChainLink>> {
    let mut chains = Vec::new();
    let initial_chain = Vec::new();
    let target_entry = entries
        .iter()
        .find(|e| e.name == package_name && e.version == package_version);
    let target_descriptors = match target_entry {
        Some(entry) => &entry.descriptors,
        None => return chains,
    };

    helper(entries, target_descriptors, initial_chain, &mut chains);

    fn helper(
        entries: &[DependencyEntry],
        descriptors: &[DependencySpec],
        current_chain: Vec<ChainLink>,
        chains: &mut Vec<Vec<ChainLink>>,
    ) {
        let mut found_parent = false;
        for entry in entries {
            for dependency in &entry.dependencies {
                if descriptors.contains(dependency) {
                    found_parent = true;
                    let mut branch = current_chain.clone();

                    branch.push(ChainLink {
                        name: entry.name.to_string(),
                        version: entry.version.to_string(),
                        requested_as: dependency.requested_as.to_string(),
                    });

                    helper(entries, &entry.descriptors, branch, chains);
                }
            }
        }
        if !found_parent {
            chains.push(current_chain);
        }
    }

    chains
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
