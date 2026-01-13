use yarn_lock_parser::Entry;

pub fn package_exists(entries: &[Entry], package_name: &str) -> bool {
    for entry in entries.iter() {
        if entry.name == package_name {
            return true;
        }
    }
    false
}

pub fn find_parents(entries: &[Entry], package_name: &str) -> Vec<String> {
    let mut parents = Vec::new();

    for entry in entries.iter() {
        for dep in entry.dependencies.iter() {
            if dep.0 == package_name {
                parents.push(entry.name.to_string());
            }
        }
    }

    parents
}
