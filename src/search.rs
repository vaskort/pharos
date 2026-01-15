use yarn_lock_parser::Entry;

#[derive(Clone)]
pub struct ChainLink {
    pub name: String,
    pub version: String,
    pub requested_as: String,
}

pub fn package_exists(entries: &[Entry], package_name: &str) -> bool {
    for entry in entries.iter() {
        if entry.name == package_name {
            return true;
        }
    }
    false
}

pub fn find_dependency_chains(entries: &[Entry], package_name: &str) -> Vec<Vec<ChainLink>> {
    let mut chains = Vec::new();
    let initial_chain = Vec::new();

    fn helper(
        entries: &[Entry],
        current_package: &str,
        current_chain: Vec<ChainLink>,
        chains: &mut Vec<Vec<ChainLink>>,
    ) {
        let mut found_parent = false;
        for entry in entries {
            for (dep_name, dep_version) in &entry.dependencies {
                if *dep_name == current_package {
                    found_parent = true;
                    let mut branch = current_chain.clone();
                    branch.push(ChainLink {
                        name: entry.name.to_string(),
                        version: entry.version.to_string(),
                        requested_as: dep_version.to_string(),
                    });
                    helper(entries, entry.name, branch, chains);
                }
            }
        }
        if !found_parent {
            chains.push(current_chain);
        }
    }

    helper(entries, package_name, initial_chain, &mut chains);

    chains
}
