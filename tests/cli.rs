use serde_json::json;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::tempdir;

fn run_pharos(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pharos"))
        .args(args)
        .output()
        .expect("failed to run pharos binary")
}

fn run_pharos_with_registry(args: &[&str], response_body: &'static str) -> Output {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0; 4096];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });

    Command::new(env!("CARGO_BIN_EXE_pharos"))
        .args(args)
        .env("PHAROS_REGISTRY_URL", format!("http://{}", address))
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
fn help_uses_user_facing_command_name() {
    let output = run_pharos(&["--help"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: pharos-cli [OPTIONS] <PACKAGE@VERSION>"));
    assert!(stdout.contains("Examples:"));
    assert!(stdout.contains("pharos-cli qs@6.13.0 --path ."));
}

#[test]
fn missing_package_error_uses_user_facing_command_name() {
    let output = run_pharos(&[]);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error: missing package to analyze"));
    assert!(stderr.contains("Usage: pharos-cli <PACKAGE@VERSION> [OPTIONS]"));
    assert!(stderr.contains("Example:"));
    assert!(stderr.contains("pharos-cli qs@6.13.0 --path ."));
    assert!(stderr.contains("For more information, try 'pharos-cli --help'."));
}

#[test]
fn exits_with_error_when_package_version_is_missing() {
    let output = run_pharos(&["pkg-a"]);

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Missing version. Use: pharos-cli pkg@1.2.3"));
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

    assert_eq!(report["schema_version"], json!(1));
    assert_eq!(report["package"]["name"], json!("pkg-a"));
    assert_eq!(report["lockfiles"][0]["status"], json!("found"));
    assert_eq!(report["lockfiles"][0]["chains"][0]["links"], json!([]));
    assert_eq!(
        report["lockfiles"][0]["chains"][0]["remediation"]["status"],
        json!("unavailable")
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
    assert!(warnings.iter().any(|warning| {
        warning
            .as_str()
            .unwrap()
            .contains("Failed to parse package.json")
    }));
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

    assert_eq!(report["schema_version"], json!(1));
    assert_eq!(report["package"]["name"], json!("pkg-z"));
    assert_eq!(report["lockfiles"][0]["status"], json!("not_found"));
    assert_eq!(report["lockfiles"][0]["chains"], json!([]));
}

#[test]
fn rejects_invalid_or_contradictory_fixed_ranges() {
    let dir = tempdir().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");
    let path = dir.path().to_str().unwrap();

    let invalid = run_pharos(&["pkg-a@1.0.0", "--path", path, "--fixed", "not-semver"]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("Invalid --fixed value"));

    let contradictory = run_pharos(&["pkg-a@1.0.0", "--path", path, "--fixed", ">=1"]);
    assert_eq!(contradictory.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&contradictory.stderr).contains("contains vulnerable version"));
}

#[test]
fn reports_verified_direct_remediation_as_additive_json() {
    let dir = tempdir().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");
    write_package_json(dir.path(), r#"{"dependencies":{"pkg-a":"^1.0.0"}}"#);
    let path = dir.path().to_str().unwrap();

    let output = run_pharos_with_registry(
        &["pkg-a@1.0.0", "--path", path, "--fixed", "2.0.0", "--json"],
        r#"{"versions":{"2.0.0":{}}}"#,
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let chain = &report["lockfiles"][0]["chains"][0];
    assert_eq!(report["schema_version"], json!(1));
    assert_eq!(report["package"]["fixed_range"], json!(">=2.0.0"));
    assert_eq!(chain["remediation"]["status"], json!("semver_verified"));
    assert_eq!(
        chain["remediation"]["primary_action"]["kind"],
        json!("direct_update")
    );
    assert_eq!(
        chain["remediation"]["primary_action"]["requested_as"],
        json!("^2.0.0")
    );
}

#[test]
fn no_registry_returns_chains_without_network_diagnostics() {
    let dir = tempdir().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "single_package.lock");
    let path = dir.path().to_str().unwrap();

    let output = run_pharos(&[
        "pkg-a@1.0.0",
        "--path",
        path,
        "--fixed",
        "2.0.0",
        "--no-registry",
        "--json",
    ]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["lockfiles"][0]["chains"][0]["remediation"]["status"],
        json!("unavailable")
    );
    assert!(
        report["lockfiles"][0]["chains"][0]["warnings"][0]
            .as_str()
            .unwrap()
            .contains("--no-registry")
    );
}

#[test]
fn text_labels_unfixed_registry_advice_as_a_candidate() {
    let dir = tempdir().unwrap();
    copy_fixture_as_yarn_lock(dir.path(), "simple_chain.lock");
    write_package_json(dir.path(), r#"{"dependencies":{"pkg-a":"^1.0.0"}}"#);
    let path = dir.path().to_str().unwrap();

    let output = run_pharos_with_registry(
        &["pkg-b@2.0.0", "--path", path],
        r#"{"versions":{"1.0.0":{"dependencies":{"pkg-b":"^2.0.0"}},"2.0.0":{"dependencies":{"pkg-b":"^3.0.0"}}}}"#,
    );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Candidate path (not verified; pass --fixed to verify)"));
    assert!(!stdout.contains("Verified remediation"));
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
