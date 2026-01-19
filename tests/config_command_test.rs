//! Integration tests for Config Command (M4-E02-T05)
//!
//! These tests validate the config command behavior including:
//! - Displaying current configuration
//! - Setting configuration values
//! - Handling missing config files (showing defaults)
//! - Creating config files when setting values

mod common;

use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_show() {
    // M4-E02-T05-S01: Integration test: config (show)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");

    cmd.assert()
        .success()
        .stdout(contains("Configuration File:"))
        .stdout(contains("Status: Using defaults"))
        .stdout(contains("Agents:"))
        .stdout(contains("claude-code:"))
        .stdout(contains("Enabled: true (default)"))
        .stdout(contains("Global Path:"))
        .stdout(contains("Workspace Path:"));
}

#[test]
fn test_config_show_with_json() {
    // Test JSON output for config show
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--json");

    cmd.assert()
        .success()
        .stdout(contains("config_file"))
        .stdout(contains("file_exists"))
        .stdout(contains("agents"))
        .stdout(contains("enabled"))
        .stdout(contains("global_path"))
        .stdout(contains("workspace_path"))
        .stdout(contains("is_default"));
}

#[test]
fn test_config_set() {
    // M4-E02-T05-S02: Integration test: config set
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--set");
    cmd.arg("agents.claude-code.enabled");
    cmd.arg("false");

    cmd.assert()
        .success()
        .stdout(contains("Set agents.claude-code.enabled = false"));

    // Verify the value was set by running config show
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");

    cmd.assert()
        .success()
        .stdout(contains("Enabled: false (custom)"));

    // Verify config file was created
    let config_file = config_dir.join("config.toml");
    assert!(config_file.exists());
    let content = fs::read_to_string(&config_file).expect("Failed to read config file");
    assert!(content.contains("enabled = false"));
}

#[test]
fn test_config_set_path() {
    // Test setting a path value
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--set");
    cmd.arg("agents.windsurf.global_path");
    cmd.arg("/custom/skills");

    cmd.assert()
        .success()
        .stdout(contains("Set agents.windsurf.global_path = /custom/skills"));

    // Verify the value was set
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");

    cmd.assert()
        .success()
        .stdout(contains("Global Path: /custom/skills"));
}

#[test]
fn test_config_with_no_file() {
    // M4-E02-T05-S03: Integration test: config with no file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    // Don't create .sikil directory or config file

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");

    cmd.assert()
        .success()
        .stdout(contains("Configuration File:"))
        .stdout(contains("Status: Using defaults (file does not exist)"))
        .stdout(contains("Agents:"))
        .stdout(contains("claude-code:"))
        .stdout(contains("Enabled: true (default)"));
}

#[test]
fn test_config_set_creates_file() {
    // M4-E02-T05-S04: Integration test: config set creates file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    // Don't create the directory or file beforehand

    let config_file = config_dir.join("config.toml");
    assert!(!config_file.exists());

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--set");
    cmd.arg("agents.claude-code.enabled");
    cmd.arg("false");

    cmd.assert()
        .success()
        .stdout(contains("Set agents.claude-code.enabled = false"));

    // Verify config file was created
    assert!(config_file.exists());
    let content = fs::read_to_string(&config_file).expect("Failed to read config file");
    assert!(content.contains("[agents.claude-code]"));
    assert!(content.contains("enabled = false"));
}

#[test]
fn test_config_set_invalid_key() {
    // Test error handling for invalid keys
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--set");
    cmd.arg("invalid.key");
    cmd.arg("value");

    cmd.assert()
        .failure()
        .stderr(contains("Key must be in format 'agents.<agent>.<field>'"));
}

#[test]
fn test_config_set_invalid_field() {
    // Test error handling for invalid fields
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--set");
    cmd.arg("agents.claude-code.invalid_field");
    cmd.arg("value");

    cmd.assert().failure().stderr(contains(
        "Field must be one of: enabled, global_path, workspace_path",
    ));
}

#[test]
fn test_config_set_missing_arguments() {
    // Test error handling when key or value is missing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--set");
    cmd.arg("agents.claude-code.enabled");
    // Missing value

    cmd.assert().failure().stderr(contains(
        "2 values required for '--set <KEY> <VALUE>' but 1 was provided",
    ));
}

#[test]
fn test_config_set_invalid_boolean() {
    // Test error handling for invalid boolean values
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil dir");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("config");
    cmd.arg("--set");
    cmd.arg("agents.claude-code.enabled");
    cmd.arg("maybe");

    cmd.assert()
        .failure()
        .stderr(contains("'enabled' field must be true or false"));
}
