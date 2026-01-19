//! Error handling tests for Sikil
//!
//! This test suite verifies that all error paths produce clear, actionable error messages
//! and that the application handles edge cases gracefully.

mod common;

use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use common::{create_minimal_skill_md, create_skill_dir, setup_temp_skill_dir};

/// S01: Test all error paths have clear messages
#[test]
fn test_skill_not_found_error_message() {
    let mut cmd = sikil_cmd!();
    cmd.arg("show").arg("nonexistent-skill");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Skill not found: nonexistent-skill",
    ));
}

#[test]
fn test_invalid_skill_md_error_message() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "invalid-skill");

    // Create invalid SKILL.md (missing required fields)
    fs::write(skill_dir.join("SKILL.md"), "---\n---\n").unwrap();

    let mut cmd = sikil_cmd!();
    cmd.arg("validate").arg(skill_dir.to_str().unwrap());

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Invalid SKILL.md"))
        .stdout(predicate::str::contains("missing"));
}

#[test]
fn test_validation_error_clear_message() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "invalid-name-skill");

    // Create SKILL.md with invalid name (contains invalid characters)
    let content = r#"---
name: "my.invalid.skill"
description: "Test skill with invalid name"
---
# Test
"#;
    fs::write(skill_dir.join("SKILL.md"), content).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.arg("validate").arg(skill_dir.to_str().unwrap());

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Validation failed"))
        .stdout(predicate::str::contains("name"));
}

#[test]
fn test_already_exists_error_message() {
    let temp_dir = setup_temp_skill_dir();
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).unwrap();

    let skill_dir = create_skill_dir(temp_dir.path(), "test-skill");
    create_minimal_skill_md(&skill_dir, "test-skill", "Test skill");

    // Create skill in repo to simulate it already exists
    let repo_skill = repo_dir.join("test-skill");
    fs::create_dir_all(&repo_skill).unwrap();
    create_minimal_skill_md(&repo_skill, "test-skill", "Test skill");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("install")
        .arg(skill_dir.to_str().unwrap())
        .arg("--to")
        .arg("all");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Already exists"));
}

#[test]
fn test_git_error_clear_message() {
    let temp_dir = setup_temp_skill_dir();

    // Create a marker file to indicate this is a git URL test
    let marker = temp_dir.path().join(".git");
    fs::create_dir_all(&marker).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .env("PATH", "") // Remove git from PATH
        .arg("install")
        .arg("https://github.com/user/repo.git")
        .arg("--to")
        .arg("all");

    cmd.assert().failure().stderr(
        predicate::str::contains("git")
            .or(predicate::str::contains("Git"))
            .or(predicate::str::contains("command not found")),
    );
}

#[test]
fn test_invalid_git_url_error_message() {
    let temp_dir = setup_temp_skill_dir();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("install")
        .arg("https://example.com/malicious.git")
        .arg("--to")
        .arg("all");

    // Should fail with git error or URL validation error
    cmd.assert().failure().stderr(
        predicate::str::contains("git")
            .or(predicate::str::contains("Git"))
            .or(predicate::str::contains("Invalid")),
    );
}

#[test]
fn test_symlink_not_allowed_error_message() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "skill-with-symlink");
    create_minimal_skill_md(&skill_dir, "skill-with-symlink", "Test skill");

    // Create a symlink inside the skill directory
    let target = temp_dir.path().join("target.txt");
    fs::write(&target, "target content").unwrap();
    let link = skill_dir.join("link.txt");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("install")
        .arg(skill_dir.to_str().unwrap())
        .arg("--to")
        .arg("all");

    cmd.assert().failure().stderr(
        predicate::str::contains("Symlink not allowed").or(predicate::str::contains("symlink")),
    );
}

/// S02: Test permission denied scenarios
#[test]
fn test_permission_denied_read_skill_md() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "protected-skill");
    create_minimal_skill_md(&skill_dir, "protected-skill", "Test skill");

    let skill_md = skill_dir.join("SKILL.md");

    // Remove read permissions
    let mut perms = fs::metadata(&skill_md).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&skill_md, perms).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.arg("validate").arg(skill_dir.to_str().unwrap());

    let result = cmd.assert().failure();

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&skill_md).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&skill_md, perms).unwrap();

    result.stdout(
        predicate::str::contains("Failed to read file").or(predicate::str::contains("permission")),
    );
}

#[test]
fn test_permission_denied_write_to_directory() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "test-skill");
    create_minimal_skill_md(&skill_dir, "test-skill", "Test skill");

    // Create .sikil directory but make it read-only
    let sikil_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&sikil_dir).unwrap();

    // Remove write permissions from .sikil directory
    let mut perms = fs::metadata(&sikil_dir).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&sikil_dir, perms.clone()).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("install")
        .arg(skill_dir.to_str().unwrap())
        .arg("--to")
        .arg("all");

    let result = cmd.assert().failure();

    // Restore permissions for cleanup
    perms.set_mode(0o755);
    fs::set_permissions(&sikil_dir, perms).unwrap();

    result.stderr(
        predicate::str::contains("Permission denied").or(predicate::str::contains("permission")),
    );
}

#[test]
fn test_permission_denied_create_symlink() {
    let temp_dir = setup_temp_skill_dir();
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).unwrap();

    let skill_dir = create_skill_dir(temp_dir.path(), "test-skill");
    create_minimal_skill_md(&skill_dir, "test-skill", "Test skill");

    // Create agent directory without write permissions
    let agent_dir = temp_dir.path().join("agent-skills");
    fs::create_dir_all(&agent_dir).unwrap();
    let mut perms = fs::metadata(&agent_dir).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&agent_dir, perms.clone()).unwrap();

    // Create config that points to protected directory
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).unwrap();
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".sikil/skills"
"#,
        agent_dir.to_str().unwrap()
    );
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("install")
        .arg(skill_dir.to_str().unwrap())
        .arg("--to")
        .arg("claude-code");

    let result = cmd.assert().failure();

    // Restore permissions for cleanup
    perms.set_mode(0o755);
    fs::set_permissions(&agent_dir, perms).unwrap();

    result.stderr(
        predicate::str::contains("Symlink error").or(predicate::str::contains("permission")),
    );
}

/// S03: Test missing directory scenarios
#[test]
fn test_missing_skill_directory() {
    let mut cmd = sikil_cmd!();
    cmd.arg("validate").arg("/nonexistent/path/to/skill");

    cmd.assert().failure().stderr(
        predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
    );
}

#[test]
fn test_missing_skill_md_file() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "no-skill-md");
    // Don't create SKILL.md

    let mut cmd = sikil_cmd!();
    cmd.arg("validate").arg(skill_dir.to_str().unwrap());

    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("SKILL.md").and(predicate::str::contains("not found")));
}

#[test]
fn test_missing_agent_directory() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "test-skill");
    create_minimal_skill_md(&skill_dir, "test-skill", "Test skill");

    // Create config with non-existent agent directory
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).unwrap();
    let config_content = r#"
[agents.claude-code]
enabled = true
global_path = "/nonexistent/agent/path"
workspace_path = ".sikil/skills"
"#;
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("install")
        .arg(skill_dir.to_str().unwrap())
        .arg("--to")
        .arg("claude-code");

    // The command succeeds but reports failure to create agent directory
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Failed to create agent directory"));
}

#[test]
fn test_missing_repo_directory_auto_created() {
    let temp_dir = setup_temp_skill_dir();
    let skill_dir = create_skill_dir(temp_dir.path(), "test-skill");
    create_minimal_skill_md(&skill_dir, "test-skill", "Test skill");

    // Don't create .sikil/repo - it should be auto-created
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    assert!(!repo_dir.exists());

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("install")
        .arg(skill_dir.to_str().unwrap())
        .arg("--to")
        .arg("all");

    // Should succeed and create repo directory
    cmd.assert().success();
    assert!(repo_dir.exists());
}

/// S04: Test broken symlink scenarios
#[test]
fn test_broken_symlink_detection() {
    let temp_dir = setup_temp_skill_dir();
    let agent_dir = temp_dir.path().join("agent-skills");
    fs::create_dir_all(&agent_dir).unwrap();

    // Create a symlink to a non-existent target
    let broken_link = agent_dir.join("broken-skill");
    let nonexistent_target = temp_dir.path().join("nonexistent");
    std::os::unix::fs::symlink(&nonexistent_target, &broken_link).unwrap();

    // Create config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).unwrap();
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".sikil/skills"
"#,
        agent_dir.to_str().unwrap()
    );
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path()).arg("list");

    // Scanner skips broken symlinks, so no skills are found
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No skills found"));
}

#[test]
fn test_show_broken_symlink() {
    let temp_dir = setup_temp_skill_dir();
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).unwrap();

    let agent_dir = temp_dir.path().join("agent-skills");
    fs::create_dir_all(&agent_dir).unwrap();

    // Create skill in repo
    let skill_dir = repo_dir.join("test-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    create_minimal_skill_md(&skill_dir, "test-skill", "Test skill");

    // Create symlink
    let link = agent_dir.join("test-skill");
    std::os::unix::fs::symlink(&skill_dir, &link).unwrap();

    // Now delete the repo skill to break the symlink
    fs::remove_dir_all(&skill_dir).unwrap();

    // Create config
    let config_dir = temp_dir.path().join(".sikil");
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".sikil/skills"
"#,
        agent_dir.to_str().unwrap()
    );
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("show")
        .arg("test-skill");

    // Broken symlinks are skipped by scanner, so skill not found
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Skill not found"));
}

#[test]
fn test_remove_broken_symlink() {
    let temp_dir = setup_temp_skill_dir();
    let agent_dir = temp_dir.path().join("agent-skills");
    fs::create_dir_all(&agent_dir).unwrap();

    // Create a broken symlink
    let broken_link = agent_dir.join("broken-skill");
    let nonexistent_target = temp_dir.path().join("nonexistent");
    std::os::unix::fs::symlink(&nonexistent_target, &broken_link).unwrap();

    // Create config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).unwrap();
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".sikil/skills"
"#,
        agent_dir.to_str().unwrap()
    );
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("remove")
        .arg("broken-skill")
        .arg("--all")
        .arg("--yes");

    // Broken symlinks are not detected by scanner, so skill not found
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Skill not found"));
}

#[test]
fn test_sync_with_broken_symlink() {
    let temp_dir = setup_temp_skill_dir();
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).unwrap();

    let agent_dir = temp_dir.path().join("agent-skills");
    fs::create_dir_all(&agent_dir).unwrap();

    // Create skill in repo
    let skill_dir = repo_dir.join("test-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    create_minimal_skill_md(&skill_dir, "test-skill", "Test skill");

    // Create broken symlink
    let link = agent_dir.join("test-skill");
    let nonexistent = temp_dir.path().join("nonexistent");
    std::os::unix::fs::symlink(&nonexistent, &link).unwrap();

    // Create config
    let config_dir = temp_dir.path().join(".sikil");
    let config_content = format!(
        r#"
[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".sikil/skills"
"#,
        agent_dir.to_str().unwrap()
    );
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("sync")
        .arg("test-skill");

    // Should fix broken symlink or report it
    cmd.assert().success();
}
