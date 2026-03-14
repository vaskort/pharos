use super::clean_version;

mod clean_version_tests {
    use super::*;

    #[test]
    fn strips_common_prefixes() {
        assert_eq!(clean_version("v1.2.3"), "1.2.3");
        assert_eq!(clean_version("^1.2.3"), "1.2.3");
        assert_eq!(clean_version("~1.2.3"), "1.2.3");
        assert_eq!(clean_version("1.2.3"), "1.2.3");
        assert_eq!(clean_version("v0.0.1-alpha"), "0.0.1-alpha");
    }
}
