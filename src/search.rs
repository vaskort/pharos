use std::collections::HashMap;

/// A package reference inside a lockfile.
///
/// For dependencies this is the requested range from the parent package.
/// For descriptors this is the range that resolved to the entry's concrete version.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
    struct ParentMatch<'a> {
        order: usize,
        entry: &'a DependencyEntry,
        requested_as: &'a str,
    }

    type ParentIndex<'a> = HashMap<&'a DependencySpec, Vec<ParentMatch<'a>>>;

    fn build_parent_index(entries: &[DependencyEntry]) -> ParentIndex<'_> {
        let dependency_count = entries
            .iter()
            .map(|entry| entry.dependencies.len())
            .sum::<usize>();
        let mut index = HashMap::with_capacity(dependency_count);
        let mut order = 0;

        for entry in entries {
            for dependency in &entry.dependencies {
                index
                    .entry(dependency)
                    .or_insert_with(Vec::new)
                    .push(ParentMatch {
                        order,
                        entry,
                        requested_as: &dependency.requested_as,
                    });
                order += 1;
            }
        }

        index
    }

    fn matching_parents<'a>(
        parent_index: &'a ParentIndex<'a>,
        descriptors: &[DependencySpec],
    ) -> Vec<&'a ParentMatch<'a>> {
        let mut parents = descriptors
            .iter()
            .filter_map(|descriptor| parent_index.get(descriptor))
            .flatten()
            .collect::<Vec<_>>();

        parents.sort_by_key(|parent| parent.order);
        parents
    }

    fn helper(
        parent_index: &ParentIndex<'_>,
        descriptors: &[DependencySpec],
        current_chain: &mut Vec<ChainLink>,
        chains: &mut Vec<Vec<ChainLink>>,
    ) {
        let parents = matching_parents(parent_index, descriptors);

        if parents.is_empty() {
            chains.push(current_chain.clone());
            return;
        }

        for parent in parents {
            current_chain.push(ChainLink {
                name: parent.entry.name.to_string(),
                version: parent.entry.version.to_string(),
                requested_as: parent.requested_as.to_string(),
            });

            helper(
                parent_index,
                &parent.entry.descriptors,
                current_chain,
                chains,
            );
            current_chain.pop();
        }
    }

    let mut chains = Vec::new();
    let mut current_chain = Vec::new();
    let target_entry = entries
        .iter()
        .find(|e| e.name == package_name && e.version == package_version);
    let target_descriptors = match target_entry {
        Some(entry) => &entry.descriptors,
        None => return chains,
    };
    let parent_index = build_parent_index(entries);

    helper(
        &parent_index,
        target_descriptors,
        &mut current_chain,
        &mut chains,
    );

    chains
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
