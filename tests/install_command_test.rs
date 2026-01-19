//! Integration tests for Install Command (M3-E01-T05)
//!
//! These tests validate the install command behavior including:
//! - Installing a valid skill
//! - Installing to specific agents
//! - Installing to all agents
//! - Duplicate installation failures
//! - Installation over physical directories
//! - Invalid skill installation failures

mod common;

use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a valid skill directory with SKILL.md
fn create_valid_skill(base_dir: &Path, skill_name: &str) {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = format!(
        r#"---
name: "{}"
description: "A test skill for installation"
version: "1.0.0"
author: "Test Author"
---

# Test Skill

This is a test skill for installation testing.
"#,
        skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
    fs::write(skill_dir.join("script.sh"), "#!/bin/sh\necho test")
        .expect("Failed to write script.sh");
}

/// Helper to create an invalid skill (missing SKILL.md)
fn create_invalid_skill(base_dir: &Path, skill_name: &str) {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");
    fs::write(skill_dir.join("README.md"), "# Just a readme").expect("Failed to write README.md");
}

#[test]
fn test_install_valid_skill() {
    // M3-E01-T05-S01: Integration test: install valid skill
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent_skills = temp_dir.path().join("agents");
    fs::create_dir_all(&agent_skills).expect("Failed to create agent skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create a valid skill to install
    create_valid_skill(&skills_source, "test-install-skill");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("test-install-skill");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .success()
        .stdout(contains("Installing skill: test-install-skill"))
        .stdout(contains("Linked to claude-code"))
        .stdout(contains("Successfully installed test-install-skill"));

    // Verify skill was copied to repo
    assert!(repo_dir.join("test-install-skill").exists());
    assert!(repo_dir
        .join("test-install-skill")
        .join("SKILL.md")
        .exists());

    // Verify symlink was created
    let symlink_path = agent_skills.join("test-install-skill");
    #[cfg(unix)]
    {
        assert!(symlink_path.exists());
        assert!(symlink_path.is_symlink());
    }
}

#[test]
fn test_install_to_specific_agents() {
    // M3-E01-T05-S02: Integration test: install to specific agents
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent1_skills = temp_dir.path().join("claude").join("skills");
    let agent2_skills = temp_dir.path().join("windsurf").join("skills");
    fs::create_dir_all(&agent1_skills).expect("Failed to create agent1 skills dir");
    fs::create_dir_all(&agent2_skills).expect("Failed to create agent2 skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create a valid skill
    create_valid_skill(&skills_source, "multi-agent-skill");

    // Set up config with two agents
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"

[agents.windsurf]
enabled = true
global_path = "{}"
workspace_path = ".windsurf/skills"
"#,
        agent1_skills.display(),
        agent2_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("multi-agent-skill");

    // Install only to claude-code
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .success();

    // Verify symlink only in claude-code directory
    #[cfg(unix)]
    {
        assert!(agent1_skills.join("multi-agent-skill").exists());
        assert!(agent1_skills.join("multi-agent-skill").is_symlink());
        assert!(!agent2_skills.join("multi-agent-skill").exists());
    }

    // Verify skill in repo
    assert!(repo_dir.join("multi-agent-skill").exists());
}

#[test]
fn test_install_to_all_agents() {
    // M3-E01-T05-S03: Integration test: install to all
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent1_skills = temp_dir.path().join("claude").join("skills");
    let agent2_skills = temp_dir.path().join("windsurf").join("skills");
    fs::create_dir_all(&agent1_skills).expect("Failed to create agent1 skills dir");
    fs::create_dir_all(&agent2_skills).expect("Failed to create agent2 skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create a valid skill
    create_valid_skill(&skills_source, "all-agents-skill");

    // Set up config with two agents
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"

[agents.windsurf]
enabled = true
global_path = "{}"
workspace_path = ".windsurf/skills"
"#,
        agent1_skills.display(),
        agent2_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("all-agents-skill");

    // Install to all agents
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("all")
        .assert()
        .success();

    // Verify symlinks in both agent directories
    #[cfg(unix)]
    {
        assert!(agent1_skills.join("all-agents-skill").exists());
        assert!(agent1_skills.join("all-agents-skill").is_symlink());
        assert!(agent2_skills.join("all-agents-skill").exists());
        assert!(agent2_skills.join("all-agents-skill").is_symlink());
    }

    // Verify skill in repo
    assert!(repo_dir.join("all-agents-skill").exists());
}

#[test]
fn test_install_duplicate_fails() {
    // M3-E01-T05-S04: Integration test: install duplicate fails
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent_skills = temp_dir.path().join("agents");
    fs::create_dir_all(&agent_skills).expect("Failed to create agent skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create a valid skill
    create_valid_skill(&skills_source, "duplicate-skill");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("duplicate-skill");

    // First install should succeed
    let mut cmd1 = sikil_cmd!();
    cmd1.env("HOME", temp_dir.path());

    cmd1.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .success();

    // Second install should fail
    let mut cmd2 = sikil_cmd!();
    cmd2.env("HOME", temp_dir.path());

    cmd2.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .failure()
        .stderr(contains("Already exists"));
}

#[test]
fn test_install_over_physical_dir_fails() {
    // M3-E01-T05-S05: Integration test: install over physical dir fails
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent_skills = temp_dir.path().join("agents");
    fs::create_dir_all(&agent_skills).expect("Failed to create agent skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create a physical directory at the destination (unmanaged skill)
    // We manually create the skill content in place
    let existing_skill = agent_skills.join("physical-dir-skill");
    fs::create_dir(&existing_skill).expect("Failed to create existing skill");

    // Manually write SKILL.md for the existing skill
    let content = r#"---
name: "physical-dir-skill"
description: "An existing unmanaged skill"
version: "1.0.0"
---

# Physical Dir Skill
"#;
    fs::write(existing_skill.join("SKILL.md"), content).expect("Failed to write SKILL.md");

    // Create a valid skill to install with same name
    create_valid_skill(&skills_source, "physical-dir-skill");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("physical-dir-skill");

    // Install should fail with suggestion to use adopt
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .failure()
        .stderr(contains("adopt"));
}

#[test]
fn test_install_invalid_skill_fails() {
    // M3-E01-T05-S06: Integration test: install invalid skill fails
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent_skills = temp_dir.path().join("agents");
    fs::create_dir_all(&agent_skills).expect("Failed to create agent skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create an invalid skill (missing SKILL.md)
    create_invalid_skill(&skills_source, "invalid-skill");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("invalid-skill");

    // Install should fail with validation error
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .failure()
        .stderr(contains("SKILL.md").or(contains("invalid")));
}

#[test]
fn test_install_json_mode_uses_all_agents() {
    // Test that --json mode defaults to all agents when --to is not specified
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent1_skills = temp_dir.path().join("claude").join("skills");
    let agent2_skills = temp_dir.path().join("windsurf").join("skills");
    fs::create_dir_all(&agent1_skills).expect("Failed to create agent1 skills dir");
    fs::create_dir_all(&agent2_skills).expect("Failed to create agent2 skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create a valid skill
    create_valid_skill(&skills_source, "json-mode-skill");

    // Set up config with two agents
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"

[agents.windsurf]
enabled = true
global_path = "{}"
workspace_path = ".windsurf/skills"
"#,
        agent1_skills.display(),
        agent2_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("json-mode-skill");

    // Install without --to in JSON mode (should default to all)
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("install")
        .arg("--json")
        .arg(skill_path.to_str().unwrap())
        .assert()
        .success();

    // Verify symlinks in both agent directories
    #[cfg(unix)]
    {
        assert!(agent1_skills.join("json-mode-skill").exists());
        assert!(agent2_skills.join("json-mode-skill").exists());
    }
}

#[test]
fn test_install_creates_agent_directory_if_missing() {
    // Test that install creates agent directories if they don't exist
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    let agent_skills = temp_dir.path().join("agents");
    // Don't create the agent directory - let install create it

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create a valid skill
    create_valid_skill(&skills_source, "create-dir-skill");

    // Set up config
    let config_dir = temp_dir.path().join(".sikil");
    fs::create_dir_all(&config_dir).expect("Failed to create .sikil");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    let skill_path = skills_source.join("create-dir-skill");

    // Install should succeed and create the directory
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());

    cmd.arg("install")
        .arg(skill_path.to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .success();

    // Verify agent directory was created
    assert!(agent_skills.exists());

    #[cfg(unix)]
    {
        assert!(agent_skills.join("create-dir-skill").exists());
        assert!(agent_skills.join("create-dir-skill").is_symlink());
    }
}
