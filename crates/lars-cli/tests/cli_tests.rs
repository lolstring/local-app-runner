//! Integration tests for the LARS CLI

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn lars_cmd(temp: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("lars").unwrap();
    cmd.env("LARS_CONFIG_HOME", temp.path());
    cmd
}

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("lars").unwrap();
    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("Local App Runner")
            .and(predicate::str::contains("add"))
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("start")),
    );
}

#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("lars").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("lar"));
}

#[test]
fn test_add_and_list() {
    let temp = TempDir::new().unwrap();

    // Add a service
    lars_cmd(&temp)
        .args(["add", "echo hello", "--name", "test-service"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added service 'test-service'"));

    // List services
    lars_cmd(&temp)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-service"));

    // List with JSON
    lars_cmd(&temp)
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-service"));
}

#[test]
fn test_add_auto_name() {
    let temp = TempDir::new().unwrap();

    // Add without explicit name
    lars_cmd(&temp)
        .args(["add", "python -m http.server"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added service 'python'"));

    // Verify it was added
    lars_cmd(&temp)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("python"));
}

#[test]
fn test_add_duplicate_fails() {
    let temp = TempDir::new().unwrap();

    // Add first service
    lars_cmd(&temp)
        .args(["add", "echo hello", "--name", "test"])
        .assert()
        .success();

    // Try to add duplicate
    lars_cmd(&temp)
        .args(["add", "echo world", "--name", "test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_add_invalid_name_fails() {
    let temp = TempDir::new().unwrap();

    // Invalid name with shell characters
    lars_cmd(&temp)
        .args(["add", "echo hello", "--name", "test;evil"])
        .assert()
        .failure();
}

#[test]
fn test_remove_service() {
    let temp = TempDir::new().unwrap();

    // Add a service
    lars_cmd(&temp)
        .args(["add", "echo hello", "--name", "test"])
        .assert()
        .success();

    // Remove it
    lars_cmd(&temp)
        .args(["remove", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed service 'test'"));

    // Verify it's gone
    lars_cmd(&temp)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No services"));
}

#[test]
fn test_remove_nonexistent_fails() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args(["remove", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_enable_disable() {
    let temp = TempDir::new().unwrap();

    // Add a service
    lars_cmd(&temp)
        .args(["add", "echo hello", "--name", "test"])
        .assert()
        .success();

    // Disable it
    lars_cmd(&temp)
        .args(["disable", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("disabled"));

    // Verify it's disabled (won't show in list without --all)
    lars_cmd(&temp)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No services").or(predicate::str::contains("disabled")));

    // Show with --all
    lars_cmd(&temp)
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));

    // Enable it
    lars_cmd(&temp)
        .args(["enable", "test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("enabled"));
}

#[test]
fn test_inspect() {
    let temp = TempDir::new().unwrap();

    // Add a service
    lars_cmd(&temp)
        .args(["add", "echo hello", "--name", "test"])
        .assert()
        .success();

    // Inspect it
    lars_cmd(&temp)
        .args(["inspect", "test"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Service: test")
                .and(predicate::str::contains("Command: echo hello"))
                .and(predicate::str::contains("Runner:")),
        );

    // Inspect with JSON
    lars_cmd(&temp)
        .args(["inspect", "test", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("test")
                .and(predicate::str::contains("echo hello")),
        );
}

#[test]
fn test_config_show() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Configuration")
                .and(predicate::str::contains("default_runner")),
        );
}

#[test]
fn test_config_set() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args(["config", "set", "shutdown_behavior", "leave_running"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Set shutdown_behavior"));

    // Verify it was set
    lars_cmd(&temp)
        .args(["config", "show", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("leave_running"));
}

#[test]
fn test_config_set_invalid_key() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args(["config", "set", "invalid_key", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown config key"));
}

#[test]
fn test_doctor() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("System Diagnostics"));
}

#[test]
fn test_export_import() {
    let temp = TempDir::new().unwrap();
    let export_path = temp.path().join("export.json");

    // Add a service
    lars_cmd(&temp)
        .args(["add", "echo hello", "--name", "test"])
        .assert()
        .success();

    // Export
    lars_cmd(&temp)
        .args(["export", "--output", export_path.to_str().unwrap()])
        .assert()
        .success();

    // Verify export file exists
    assert!(export_path.exists());

    // Create new temp dir for import
    let temp2 = TempDir::new().unwrap();

    // Import
    lars_cmd(&temp2)
        .args(["import", export_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported"));

    // Verify service exists in new config
    lars_cmd(&temp2)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_completions() {
    let mut cmd = Command::cargo_bin("lars").unwrap();
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_start_nonexistent() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args(["start", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_add_with_env() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args([
            "add",
            "echo hello",
            "--name",
            "test",
            "-e",
            "FOO=bar",
            "-e",
            "BAZ=qux",
        ])
        .assert()
        .success();

    // Verify env vars were stored
    lars_cmd(&temp)
        .args(["inspect", "test", "--json"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("FOO").and(predicate::str::contains("BAZ")),
        );
}

#[test]
fn test_add_with_workdir() {
    let temp = TempDir::new().unwrap();

    lars_cmd(&temp)
        .args([
            "add",
            "echo hello",
            "--name",
            "test",
            "-d",
            "/tmp",
        ])
        .assert()
        .success();

    // Verify workdir was stored
    lars_cmd(&temp)
        .args(["inspect", "test", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"/tmp\""));
}
