pub fn clean_version(version: &str) -> &str {
    version.trim_start_matches(|c: char| !c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::clean_version;

    #[test]
    fn test_clean_version() {
        assert_eq!(clean_version("v1.2.3"), "1.2.3");
        assert_eq!(clean_version("^1.2.3"), "1.2.3");
        assert_eq!(clean_version("~1.2.3"), "1.2.3");
        assert_eq!(clean_version("1.2.3"), "1.2.3");
        assert_eq!(clean_version("v0.0.1-alpha"), "0.0.1-alpha");
    }
}
