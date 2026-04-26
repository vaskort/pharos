use serde_json::json;
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

fn write_package_lock(dir: &Path, content: &str) {
    fs::write(dir.join("package-lock.json"), content).expect("failed to write package-lock.json");
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

#[test]
fn reports_direct_dependency_as_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path, "--json"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(
        report,
        json!({
            "package": {
                "name": "pkg-a",
                "version": "1.0.0"
            },
            "lockfiles": [
                {
                    "path": dir.path().join("yarn.lock").display().to_string(),
                    "lockfile_type": "yarn",
                    "status": "found",
                    "chains": [
                        {
                            "links": [],
                            "fix_path": [],
                            "recommended": null,
                            "warnings": []
                        }
                    ]
                }
            ]
        })
    );
}

#[test]
fn reports_missing_package_as_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");

    let output = run_pharos(&["pkg-z@1.0.0", "--path", path, "--json"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(
        report,
        json!({
            "package": {
                "name": "pkg-z",
                "version": "1.0.0"
            },
            "lockfiles": [
                {
                    "path": dir.path().join("yarn.lock").display().to_string(),
                    "lockfile_type": "yarn",
                    "status": "not_found",
                    "chains": []
                }
            ]
        })
    );
}

#[test]
fn reports_direct_dependency_found_in_package_lockfile() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    write_package_lock(
        dir.path(),
        r#"{
            "name": "test-project",
            "version": "1.0.0",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "name": "test-project",
                    "version": "1.0.0",
                    "dependencies": {
                        "pkg-a": "^1.0.0"
                    }
                },
                "node_modules/pkg-a": {
                    "version": "1.0.0"
                }
            }
        }"#,
    );

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found pkg-a@1.0.0"));
    assert!(stdout.contains("pkg-a@1.0.0 (is a direct dependency)"));
}
