pub fn clean_version(version: &str) -> &str {
    version.trim_start_matches(|c: char| !c.is_ascii_digit())
}
