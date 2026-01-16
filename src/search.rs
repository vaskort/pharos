use yarn_lock_parser::Entry;

#[derive(Clone, Debug)]
pub struct ChainLink {
    pub name: String,
    pub version: String,
    pub requested_as: String,
}

pub fn package_exists(entries: &[Entry], package_name: &str, package_version: &str) -> bool {
    for entry in entries.iter() {
        if entry.name == package_name && entry.version == package_version {
            return true;
        }
    }
    false
}

pub fn find_dependency_chains(
    entries: &[Entry],
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
        entries: &[Entry],
        descriptors: &Vec<(&str, &str)>,
        current_chain: Vec<ChainLink>,
        chains: &mut Vec<Vec<ChainLink>>,
    ) {
        let mut found_parent = false;
        for entry in entries {
            for (dep_name, dep_version) in &entry.dependencies {
                if (descriptors).contains(&(*dep_name, *dep_version)) {
                    found_parent = true;
                    let mut branch = current_chain.clone();

                    branch.push(ChainLink {
                        name: entry.name.to_string(),
                        version: entry.version.to_string(),
                        requested_as: dep_version.to_string(),
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
