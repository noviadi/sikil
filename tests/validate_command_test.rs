//! Integration tests for Validate Command (M2-E04-T04)
//!
//! These tests validate the validate command behavior including:
//! - Validating a valid skill (exit 0)
//! - Error handling for missing SKILL.md (exit non-zero)
//! - Error handling for invalid name format (exit non-zero)
//! - Error handling for missing required fields (exit non-zero)
//! - Snapshot testing for validation output

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

const COMMAND_NAME: &str = "sikil";

/// Helper to create a valid skill directory with SKILL.md
fn create_valid_skill(base_dir: &std::path::Path, skill_name: &str) -> std::path::PathBuf {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = format!(
        r#"---
name: "{}"
description: "A valid test skill for validation"
version: "0.1.0"
author: "Test Author"
license: "MIT"
---

# {}
"#,
        skill_name, skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
    skill_dir
}

/// Helper to create a skill with minimal valid metadata
fn create_minimal_skill(base_dir: &std::path::Path, skill_name: &str) -> std::path::PathBuf {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = format!(
        r#"---
name: "{}"
description: "A minimal test skill"
---

# {}
"#,
        skill_name, skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
    skill_dir
}

/// Helper to create a skill directory without SKILL.md
fn create_skill_without_md(base_dir: &std::path::Path, skill_name: &str) -> std::path::PathBuf {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");
    skill_dir
}

/// Helper to create a skill with invalid name
fn create_skill_with_invalid_name(
    base_dir: &std::path::Path,
    skill_name: &str,
) -> std::path::PathBuf {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    // Use the directory name as the skill name in YAML (which may be invalid)
    let content = format!(
        r#"---
name: "{}"
description: "A skill with invalid name"
---

# {}
"#,
        skill_name, skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
    skill_dir
}

/// Helper to create a skill with missing required field
fn create_skill_missing_field(
    base_dir: &std::path::Path,
    skill_name: &str,
    missing_field: &str,
) -> std::path::PathBuf {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = match missing_field {
        "name" => {
            r#"---
description: "A skill without name"
---

# Skill
"#
        }
        "description" => {
            r#"---
name: "test-skill"
---

# Skill
"#
        }
        _ => {
            r#"---
name: "test-skill"
description: "A test skill"
---

# Skill
"#
        }
    };

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
    skill_dir
}

#[test]
fn test_validate_valid_skill() {
    // M2-E04-T04-S01: Integration test: validate valid skill
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_valid_skill(temp_dir.path(), "valid-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .success()
        .stdout(contains("Validating skill at:"))
        .stdout(contains("SKILL.md exists"))
        .stdout(contains("YAML frontmatter is valid"))
        .stdout(contains("Required fields present"))
        .stdout(contains("Name format is valid"))
        .stdout(contains("Description length is valid"))
        .stdout(contains("PASSED"));
}

#[test]
fn test_validate_valid_skill_by_path() {
    // Test validating a skill by providing the SKILL.md file path directly
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_valid_skill(temp_dir.path(), "path-skill");
    let skill_md = skill_dir.join("SKILL.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(skill_md)
        .assert()
        .success()
        .stdout(contains("PASSED"));
}

#[test]
fn test_validate_missing_skill_md() {
    // M2-E04-T04-S02: Integration test: validate missing SKILL.md
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_skill_without_md(temp_dir.path(), "no-md-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Validating skill at:"))
        .stdout(contains("SKILL.md exists"))
        .stdout(contains("SKILL.md file not found"))
        .stdout(contains("FAILED"));
}

#[test]
fn test_validate_invalid_name_starts_with_hyphen() {
    // M2-E04-T04-S03: Integration test: validate invalid name
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_skill_with_invalid_name(temp_dir.path(), "-invalid-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Validating skill at:"))
        .stdout(contains("Name format is valid"))
        .stdout(contains("FAILED"))
        .stdout(contains("must start with"));
}

#[test]
fn test_validate_invalid_name_with_dot() {
    // Test name validation rejects path traversal attempts
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    // Use a safe directory name
    let skill_dir = create_skill_with_invalid_name(temp_dir.path(), "dot-skill");

    // Now manually write the SKILL.md with an invalid name containing dot
    let content = r#"---
name: "../malicious"
description: "A skill with invalid name"
---

# Skill
"#;

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Name format is valid"))
        .stdout(contains("FAILED"));
}

#[test]
fn test_validate_invalid_name_too_long() {
    // Test name validation rejects names > 64 characters
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let long_name = "a".repeat(65);
    let skill_dir = temp_dir.path().join(&long_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = format!(
        r#"---
name: "{}"
description: "A skill with a very long name"
---

# Skill
"#,
        long_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Name format is valid"))
        .stdout(contains("FAILED"))
        .stdout(contains("64 characters"));
}

#[test]
fn test_validate_missing_required_field_name() {
    // M2-E04-T04-S04: Integration test: validate missing required field
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_skill_missing_field(temp_dir.path(), "no-name", "name");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Validating skill at:"))
        .stdout(contains("Required fields present"))
        .stdout(contains("FAILED"))
        .stdout(contains("name"));
}

#[test]
fn test_validate_missing_required_field_description() {
    // Test validation with missing description
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_skill_missing_field(temp_dir.path(), "no-desc", "description");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Validating skill at:"))
        .stdout(contains("Required fields present"))
        .stdout(contains("FAILED"))
        .stdout(contains("description"));
}

#[test]
fn test_validate_invalid_frontmatter() {
    // Test validation with malformed YAML frontmatter
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = temp_dir.path().join("bad-frontmatter");
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    // Missing closing ---
    let content = r#"---
name: "test-skill"
description: "A test skill"

# Skill
"#;

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Validating skill at:"))
        .stdout(contains("YAML frontmatter is valid"))
        .stdout(contains("FAILED"));
}

#[test]
fn test_validate_no_frontmatter() {
    // Test validation with no frontmatter at all
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = temp_dir.path().join("no-frontmatter");
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = r#"# Test Skill

This is a test skill without frontmatter.
"#;

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("YAML frontmatter is valid"))
        .stdout(contains("FAILED"));
}

#[test]
fn test_validate_description_too_long() {
    // Test validation rejects description > 1024 characters
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = temp_dir.path().join("long-desc");
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let long_desc = "a".repeat(1025);
    let content = format!(
        r#"---
name: "test-skill"
description: "{}"
---

# Skill
"#,
        long_desc
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Description length is valid"))
        .stdout(contains("FAILED"))
        .stdout(contains("too long"));
}

#[test]
fn test_validate_description_empty() {
    // Test validation rejects empty description
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = temp_dir.path().join("empty-desc");
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = r#"---
name: "test-skill"
description: ""
---

# Skill
"#;

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .stdout(contains("Description length is valid"))
        .stdout(contains("FAILED"))
        .stdout(contains("empty"));
}

#[test]
fn test_validate_with_warnings() {
    // Test validation shows warnings for missing optional fields
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_minimal_skill(temp_dir.path(), "minimal-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .success()
        .stdout(contains("PASSED"))
        .stdout(contains("Warnings:"))
        .stdout(contains("Optional field 'version' is missing"))
        .stdout(contains("Optional field 'author' is missing"))
        .stdout(contains("Optional field 'license' is missing"));
}

#[test]
fn test_validate_with_directories() {
    // Test validation detects scripts/ and references/ directories
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_valid_skill(temp_dir.path(), "dir-skill");

    // Create scripts directory
    let scripts_dir = skill_dir.join("scripts");
    fs::create_dir(&scripts_dir).expect("Failed to create scripts directory");
    fs::write(
        scripts_dir.join("install.sh"),
        "#!/bin/bash\necho 'Installing'",
    )
    .expect("Failed to write install.sh");

    // Create references directory
    let refs_dir = skill_dir.join("references");
    fs::create_dir(&refs_dir).expect("Failed to create references directory");
    fs::write(refs_dir.join("doc.md"), "# Reference documentation")
        .expect("Failed to write doc.md");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    cmd.arg("validate")
        .arg(&skill_dir)
        .assert()
        .success()
        .stdout(contains("PASSED"))
        .stdout(contains("Detected directories:"))
        .stdout(contains("scripts/:"))
        .stdout(contains("references/:"));
}

#[test]
fn test_validate_json_output_valid_skill() {
    // Test JSON output for a valid skill
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_valid_skill(temp_dir.path(), "json-valid-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    let output = cmd
        .arg("validate")
        .arg(&skill_dir)
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Verify valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Verify JSON structure
    assert_eq!(parsed["passed"], true);
    assert!(parsed["skill_path"].is_string());
    assert!(parsed["checks"].is_array());
    assert!(parsed["warnings"].is_null() || parsed["warnings"].is_array());
    assert!(parsed["detected_directories"].is_object());

    // Verify all checks passed
    let checks = parsed["checks"].as_array().unwrap();
    assert!(checks.len() >= 5); // At least 5 checks
    for check in checks {
        assert_eq!(check["passed"], true);
    }
}

#[test]
fn test_validate_json_output_invalid_skill() {
    // Test JSON output for an invalid skill (missing SKILL.md)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_skill_without_md(temp_dir.path(), "json-invalid-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    let output = cmd
        .arg("validate")
        .arg(&skill_dir)
        .arg("--json")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Verify valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Verify JSON structure shows failure
    assert_eq!(parsed["passed"], false);
    assert!(parsed["checks"].is_array());

    // Verify SKILL.md check failed
    let checks = parsed["checks"].as_array().unwrap();
    let skill_md_check = checks
        .iter()
        .find(|c| c["name"] == "SKILL.md exists")
        .expect("Should have SKILL.md exists check");
    assert_eq!(skill_md_check["passed"], false);
}

#[test]
fn test_validate_snapshot_output() {
    // M2-E04-T04-S05: Snapshot test: validation output
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_valid_skill(temp_dir.path(), "snapshot-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    let skill_dir_str = skill_dir.display().to_string();
    let output = cmd
        .arg("validate")
        .arg(&skill_dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Redact the dynamic temp path for snapshot stability
    let redacted = output_str
        .replace(&skill_dir_str, "[SKILL_DIR]")
        .replace(&temp_dir.path().display().to_string(), "[TEMP_DIR]");

    // Verify key elements are present
    assert!(redacted.contains("Validating skill at: [SKILL_DIR]"));
    assert!(redacted.contains("SKILL.md exists"));
    assert!(redacted.contains("YAML frontmatter is valid"));
    assert!(redacted.contains("Required fields present"));
    assert!(redacted.contains("Name format is valid"));
    assert!(redacted.contains("Description length is valid (1-1024)"));
    assert!(redacted.contains("PASSED"));

    // Verify check marks are present
    assert!(redacted.contains(""));

    // Verify optional fields warnings are not present (skill has all fields)
    assert!(!redacted.contains("Warnings:"));
}

#[test]
fn test_validate_snapshot_output_with_warnings() {
    // Snapshot test for validation output with warnings
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_minimal_skill(temp_dir.path(), "snapshot-warn-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    let output = cmd
        .arg("validate")
        .arg(&skill_dir)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Verify warnings are present
    assert!(output_str.contains("Warnings:"));
    assert!(output_str.contains("Optional field 'version' is missing"));
    assert!(output_str.contains("Optional field 'author' is missing"));
    assert!(output_str.contains("Optional field 'license' is missing"));
}

#[test]
fn test_validate_snapshot_output_failure() {
    // Snapshot test for validation output showing failure
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_dir = create_skill_without_md(temp_dir.path(), "snapshot-fail-skill");

    let mut cmd = Command::cargo_bin(COMMAND_NAME).unwrap();

    let output = cmd
        .arg("validate")
        .arg(&skill_dir)
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Verify failure output
    assert!(output_str.contains("Validating skill at:"));
    assert!(output_str.contains("SKILL.md exists"));
    assert!(output_str.contains("SKILL.md file not found"));
    assert!(output_str.contains("FAILED"));
}
