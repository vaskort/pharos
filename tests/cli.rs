use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

fn run_pharos(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pharos"))
        .args(args)
        .output()
        .expect("failed to run pharos binary")
}

fn copy_fixture_as_yarn_lock(dir: &Path, fixture_name: &str) {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata")
        .join(fixture_name);
    fs::copy(fixture_path, dir.join("yarn.lock")).expect("failed to copy yarn.lock fixture");
}

#[test]
fn exits_with_error_when_package_version_is_missing() {
    let output = run_pharos(&["pkg-a"]);

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Missing version"));
}

#[test]
fn exits_with_error_when_no_lockfiles_are_found() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path]);

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No lockfiles found"));
}

#[test]
fn reports_missing_package_in_yarn_lockfile() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");

    let output = run_pharos(&["pkg-z@1.0.0", "--path", path]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Package pkg-z@1.0.0 not found"));
}

#[test]
fn reports_direct_dependency_found_in_yarn_lockfile() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found pkg-a@1.0.0"));
    assert!(stdout.contains("pkg-a@1.0.0 (is a direct dependency)"));
}
