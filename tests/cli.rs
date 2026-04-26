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

fn write_package_json(dir: &Path, content: &str) {
    fs::write(dir.join("package.json"), content).expect("failed to write package.json");
}

fn write_yarn_lock(dir: &Path, content: &str) {
    fs::write(dir.join("yarn.lock"), content).expect("failed to write yarn.lock");
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
fn reports_direct_dependency_owner_from_package_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");
    write_package_json(
        dir.path(),
        r#"{
            "dependencies": {
                "pkg-a": "^1.0.0"
            }
        }"#,
    );

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Owner:"));
    assert!(stdout.contains("pkg-a from dependencies, requested as ^1.0.0"));
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
                            "owner": null,
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
fn reports_direct_dependency_owner_as_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");
    write_package_json(
        dir.path(),
        r#"{
            "dependencies": {
                "pkg-a": "^1.0.0"
            }
        }"#,
    );

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path, "--json"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(
        report["lockfiles"][0]["chains"][0]["owner"],
        json!({
            "name": "pkg-a",
            "dependency_type": "dependencies",
            "requested_as": "^1.0.0"
        })
    );
}

#[test]
fn reports_invalid_package_json_warning_as_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");
    write_package_json(dir.path(), "not json");

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path, "--json"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let warnings = report["lockfiles"][0]["chains"][0]["warnings"]
        .as_array()
        .unwrap();

    assert_eq!(report["lockfiles"][0]["chains"][0]["owner"], json!(null));
    assert_eq!(warnings.len(), 1);
    assert!(
        warnings[0]
            .as_str()
            .unwrap()
            .contains("Failed to parse package.json")
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
fn reports_lockfile_parse_error_as_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    write_yarn_lock(dir.path(), "not valid yarn lock content");

    let output = run_pharos(&["pkg-a@1.0.0", "--path", path, "--json"]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(report["lockfiles"][0]["status"], json!("error"));
    assert_eq!(report["lockfiles"][0]["chains"], json!([]));
    assert!(
        report["lockfiles"][0]["error"]
            .as_str()
            .unwrap()
            .contains("Failed to parse yarn.lock")
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
