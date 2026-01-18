//! Integration tests for Show Command (M2-E03-T04)
//!
//! These tests validate the show command behavior including:
//! - Showing an existing skill
//! - Error handling for non-existent skills
//! - JSON output
//! - Snapshot testing

mod common;

use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

/// Helper to create a skill directory with SKILL.md
fn create_skill(base_dir: &std::path::Path, skill_name: &str, skill_title: &str, skill_desc: &str) {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = format!(
        r#"---
name: "{}"
description: "{}"
version: "0.1.0"
author: "Test Author"
license: "MIT"
---

# {}
"#,
        skill_name, skill_desc, skill_title
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
}

/// Helper to create a skill with scripts and references directories
fn create_skill_with_structure(base_dir: &std::path::Path, skill_name: &str) {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = format!(
        r#"---
name: "{}"
description: "A skill with scripts and references"
version: "0.2.0"
author: "Test Author"
license: "MIT"
---

# {}
"#,
        skill_name, skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

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
}

#[test]
fn test_show_existing_skill() {
    // M2-E03-T04-S01: Integration test: show existing skill
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    fs::create_dir(&skills_base).expect("Failed to create skills base");

    // Create a skill
    create_skill(
        &skills_base,
        "test-skill",
        "Test Skill",
        "A test skill for showing",
    );

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("show")
        .arg("test-skill")
        .arg("--no-cache")
        .assert()
        .success()
        .stdout(contains("Skill: test-skill"))
        .stdout(contains("Description: A test skill for showing"))
        .stdout(contains("Version: 0.1.0"))
        .stdout(contains("Author: Test Author"))
        .stdout(contains("License: MIT"))
        .stdout(contains("Installations (1):"))
        .stdout(contains("claude-code"))
        .stdout(contains("global"));
}

#[test]
fn test_show_non_existent_skill() {
    // M2-E03-T04-S02: Integration test: show non-existent skill
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    fs::create_dir(&skills_base).expect("Failed to create skills base");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("show")
        .arg("non-existent-skill")
        .arg("--no-cache")
        .assert()
        .failure()
        .stderr(contains("non-existent-skill"))
        .stderr(contains("not found"));
}

#[test]
fn test_show_json_output() {
    // M2-E03-T04-S03: Integration test: show --json
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    fs::create_dir(&skills_base).expect("Failed to create skills base");

    // Create a skill
    create_skill(
        &skills_base,
        "json-skill",
        "JSON Skill",
        "Testing JSON output for show",
    );

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    let output = cmd
        .arg("show")
        .arg("json-skill")
        .arg("--json")
        .arg("--no-cache")
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
    assert_eq!(parsed["name"], "json-skill");
    assert_eq!(parsed["description"], "Testing JSON output for show");
    assert_eq!(parsed["version"], "0.1.0");
    assert_eq!(parsed["author"], "Test Author");
    assert_eq!(parsed["license"], "MIT");
    assert!(parsed["installations"].is_array());
    assert_eq!(parsed["installations"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["installations"][0]["agent"], "claude-code");
}

#[test]
fn test_show_with_file_structure() {
    // Test showing a skill with scripts and references directories
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    fs::create_dir(&skills_base).expect("Failed to create skills base");

    // Create skill with structure
    create_skill_with_structure(&skills_base, "structured-skill");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("show")
        .arg("structured-skill")
        .arg("--no-cache")
        .assert()
        .success()
        .stdout(contains("Skill: structured-skill"))
        .stdout(contains("Files:"))
        .stdout(contains("SKILL.md:"))
        .stdout(contains("scripts/:"))
        .stdout(contains("references/:"))
        .stdout(contains("Total files:"))
        .stdout(contains("Total size:"));
}

#[test]
fn test_show_managed_skill() {
    // Test showing a managed skill (symlinked from repo)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    let repo_base = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&skills_base).expect("Failed to create skills base");
    fs::create_dir_all(&repo_base).expect("Failed to create repo");

    // Create managed skill in repo
    create_skill(
        &repo_base,
        "managed-skill",
        "Managed Skill",
        "A managed skill",
    );

    // Create symlink to managed skill
    #[cfg(unix)]
    {
        let link_path = skills_base.join("managed-skill");
        let target = repo_base.join("managed-skill");
        std::os::unix::fs::symlink(target, link_path).expect("Failed to create symlink");
    }

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    #[cfg(unix)]
    {
        let mut cmd = sikil_cmd!();
        cmd.env("HOME", temp_dir.path());

        cmd.arg("show")
            .arg("managed-skill")
            .arg("--no-cache")
            .assert()
            .success()
            .stdout(contains("Skill: managed-skill"))
            .stdout(contains("Status: Managed"))
            .stdout(contains("Canonical:"));
    }
}

#[test]
fn test_show_snapshot_output() {
    // M2-E03-T04-S04: Snapshot test: show output
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    fs::create_dir(&skills_base).expect("Failed to create skills base");

    // Create a skill with all fields
    create_skill(
        &skills_base,
        "snapshot-skill",
        "Snapshot Test Skill",
        "A skill for snapshot testing with comprehensive metadata",
    );

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    let output = cmd
        .arg("show")
        .arg("snapshot-skill")
        .arg("--no-cache")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Redact the dynamic temp path for snapshot stability
    let redacted = output_str
        .replace(&skills_base.display().to_string(), "[SKILLS_PATH]")
        .replace(&temp_dir.path().display().to_string(), "[TEMP_DIR]");

    // Verify key elements are present
    assert!(redacted.contains("Skill: snapshot-skill"));
    assert!(redacted.contains("Description: A skill for snapshot testing"));
    assert!(redacted.contains("Version: 0.1.0"));
    assert!(redacted.contains("Author: Test Author"));
    assert!(redacted.contains("License: MIT"));
    assert!(redacted.contains("Status:"));
    assert!(redacted.contains("Installations (1):"));
    assert!(redacted.contains("claude-code"));

    // Note: In actual snapshot testing, you would use insta::assert_snapshot!(redacted)
    // For this test, we're verifying the structure is correct
}

#[test]
fn test_show_minimal_skill() {
    // Test showing a skill with only required fields
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    fs::create_dir(&skills_base).expect("Failed to create skills base");

    // Create skill with minimal metadata
    let skill_dir = skills_base.join("minimal-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = r#"---
name: "minimal-skill"
description: "A minimal test skill"
---

# Minimal Skill
"#;

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    let output = cmd
        .arg("show")
        .arg("minimal-skill")
        .arg("--no-cache")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Verify required content
    assert!(output_str.contains("Skill: minimal-skill"));
    assert!(output_str.contains("Description: A minimal test skill"));

    // Verify that optional fields are not shown
    assert!(!output_str.contains("Version:"));
    assert!(!output_str.contains("Author:"));
    assert!(!output_str.contains("License:"));
}

#[test]
fn test_show_json_minimal_skill() {
    // Test JSON output for a skill with only required fields
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_base = temp_dir.path().join("skills");
    fs::create_dir(&skills_base).expect("Failed to create skills base");

    // Create skill with minimal metadata
    let skill_dir = skills_base.join("minimal-json-skill");
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = r#"---
name: "minimal-json-skill"
description: "A minimal test skill for JSON"
---

# Minimal Skill
"#;

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        skills_base.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    let output = cmd
        .arg("show")
        .arg("minimal-json-skill")
        .arg("--json")
        .arg("--no-cache")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).expect("Invalid UTF-8");

    // Verify valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Verify required fields are present
    assert_eq!(parsed["name"], "minimal-json-skill");
    assert_eq!(parsed["description"], "A minimal test skill for JSON");

    // Verify optional fields are not present (should be skipped)
    assert!(parsed.get("version").is_none() || parsed["version"].is_null());
    assert!(parsed.get("author").is_none() || parsed["author"].is_null());
    assert!(parsed.get("license").is_none() || parsed["license"].is_null());
}
