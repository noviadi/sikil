//! Integration tests for Unmanage Command (M3-E04-T04)
//!
//! These tests validate the unmanage command behavior including:
//! - Unmanaging all agents (symlinks removed, copies created, repo deleted)
//! - Unmanaging specific agent only
//! - Unmanaging with --yes flag (no prompt)
//! - Error handling for unmanaged skills

mod common;

use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a valid skill in the repository
fn create_skill_in_repo(repo_dir: &Path, skill_name: &str) {
    let skill_dir = repo_dir.join(skill_name);
    fs::create_dir_all(&skill_dir).expect("Failed to create skill dir in repo");

    let content = format!(
        r#"---
name: "{}"
description: "A test skill for unmanage testing"
version: "1.0.0"
---

# Test Skill

This is a test skill.
"#,
        skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
    fs::write(skill_dir.join("script.sh"), "#!/bin/sh\necho test")
        .expect("Failed to write script.sh");
}

/// Helper to create a symlink from agent dir to repo
#[cfg(unix)]
fn create_managed_symlink(agent_dir: &Path, repo_dir: &Path, skill_name: &str) {
    let skill_repo_path = repo_dir.join(skill_name);
    let skill_link_path = agent_dir.join(skill_name);
    std::os::unix::fs::symlink(&skill_repo_path, &skill_link_path)
        .expect("Failed to create symlink");
}

/// Helper to create an unmanaged skill (physical directory, not symlink)
fn create_unmanaged_skill(agent_dir: &Path, skill_name: &str) {
    let skill_dir = agent_dir.join(skill_name);
    fs::create_dir_all(&skill_dir).expect("Failed to create unmanaged skill dir");

    let content = format!(
        r#"---
name: "{}"
description: "An unmanaged test skill"
version: "1.0.0"
---

# Unmanaged Skill
"#,
        skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
}

// M3-E04-T04-S01: Integration test: unmanage all agents
#[test]
#[cfg(unix)]
fn test_unmanage_all_agents() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Setup directories
    let agent1_dir = temp_dir.path().join("agent1");
    let agent2_dir = temp_dir.path().join("agent2");
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    let config_dir = temp_dir.path().join(".sikil");

    fs::create_dir_all(&agent1_dir).expect("Failed to create agent1 dir");
    fs::create_dir_all(&agent2_dir).expect("Failed to create agent2 dir");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    let skill_name = "unmanage-all-test";

    // Create skill in repo
    create_skill_in_repo(&repo_dir, skill_name);

    // Create symlinks in both agent directories
    create_managed_symlink(&agent1_dir, &repo_dir, skill_name);
    create_managed_symlink(&agent2_dir, &repo_dir, skill_name);

    // Create config
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
        agent1_dir.display(),
        agent2_dir.display()
    );
    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    // Verify initial state
    assert!(agent1_dir.join(skill_name).is_symlink());
    assert!(agent2_dir.join(skill_name).is_symlink());
    assert!(repo_dir.join(skill_name).exists());

    // Run unmanage command
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("unmanage")
        .arg(skill_name)
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("Unmanaging skill:"))
        .stdout(contains("Successfully unmanaged"));

    // Verify: symlinks replaced with physical directories
    let skill1_path = agent1_dir.join(skill_name);
    let skill2_path = agent2_dir.join(skill_name);

    assert!(skill1_path.exists(), "Skill should exist in agent1");
    assert!(!skill1_path.is_symlink(), "Skill in agent1 should be physical dir");
    assert!(skill1_path.join("SKILL.md").exists(), "SKILL.md should exist");

    assert!(skill2_path.exists(), "Skill should exist in agent2");
    assert!(!skill2_path.is_symlink(), "Skill in agent2 should be physical dir");

    // Verify: repo entry deleted
    assert!(
        !repo_dir.join(skill_name).exists(),
        "Skill should be removed from repo"
    );
}

// M3-E04-T04-S02: Integration test: unmanage specific agent
#[test]
#[cfg(unix)]
fn test_unmanage_specific_agent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Setup directories
    let agent1_dir = temp_dir.path().join("agent1");
    let agent2_dir = temp_dir.path().join("agent2");
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    let config_dir = temp_dir.path().join(".sikil");

    fs::create_dir_all(&agent1_dir).expect("Failed to create agent1 dir");
    fs::create_dir_all(&agent2_dir).expect("Failed to create agent2 dir");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    let skill_name = "unmanage-specific-test";

    // Create skill in repo
    create_skill_in_repo(&repo_dir, skill_name);

    // Create symlinks in both agent directories
    create_managed_symlink(&agent1_dir, &repo_dir, skill_name);
    create_managed_symlink(&agent2_dir, &repo_dir, skill_name);

    // Create config
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
        agent1_dir.display(),
        agent2_dir.display()
    );
    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    // Run unmanage command for specific agent only
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("unmanage")
        .arg(skill_name)
        .arg("--agent")
        .arg("claude-code")
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("Unmanaged from claude-code"))
        .stdout(contains("Still managed by"));

    // Verify: only agent1 (claude-code) converted to physical
    let skill1_path = agent1_dir.join(skill_name);
    assert!(skill1_path.exists(), "Skill should exist in agent1");
    assert!(
        !skill1_path.is_symlink(),
        "Skill in agent1 should be physical dir"
    );

    // Verify: agent2 (windsurf) still symlink
    let skill2_path = agent2_dir.join(skill_name);
    assert!(skill2_path.exists(), "Skill should exist in agent2");
    assert!(skill2_path.is_symlink(), "Skill in agent2 should still be symlink");

    // Verify: repo still exists (still managed by windsurf)
    assert!(
        repo_dir.join(skill_name).exists(),
        "Skill should still be in repo"
    );
}

// M3-E04-T04-S03: Integration test: unmanage with --yes
#[test]
#[cfg(unix)]
fn test_unmanage_with_yes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Setup directories
    let agent_dir = temp_dir.path().join("agents");
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    let config_dir = temp_dir.path().join(".sikil");

    fs::create_dir_all(&agent_dir).expect("Failed to create agent dir");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    let skill_name = "unmanage-yes-test";

    // Create skill in repo
    create_skill_in_repo(&repo_dir, skill_name);

    // Create symlink
    create_managed_symlink(&agent_dir, &repo_dir, skill_name);

    // Create config
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_dir.display()
    );
    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    // Run with --yes flag (should not prompt)
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("unmanage")
        .arg(skill_name)
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("Successfully unmanaged"));

    // Verify unmanage completed
    let skill_path = agent_dir.join(skill_name);
    assert!(skill_path.exists());
    assert!(!skill_path.is_symlink());
}

// M3-E04-T04-S04: Integration test: unmanage unmanaged skill
#[test]
fn test_unmanage_unmanaged_skill() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Setup directories
    let agent_dir = temp_dir.path().join("agents");
    let config_dir = temp_dir.path().join(".sikil");
    let repo_dir = temp_dir.path().join(".sikil").join("repo");

    fs::create_dir_all(&agent_dir).expect("Failed to create agent dir");
    fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    let skill_name = "unmanaged-skill-test";

    // Create unmanaged skill (physical directory, not in repo)
    create_unmanaged_skill(&agent_dir, skill_name);

    // Create config
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_dir.display()
    );
    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    // Run unmanage command - should fail
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("unmanage")
        .arg(skill_name)
        .arg("--yes")
        .assert()
        .failure()
        .stderr(contains("not managed").or(contains("not found in repository")));
}

// Additional test: unmanage non-existent skill
#[test]
fn test_unmanage_nonexistent_skill() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Setup directories
    let agent_dir = temp_dir.path().join("agents");
    let config_dir = temp_dir.path().join(".sikil");
    let repo_dir = temp_dir.path().join(".sikil").join("repo");

    fs::create_dir_all(&agent_dir).expect("Failed to create agent dir");
    fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    // Create config
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_dir.display()
    );
    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    // Run unmanage command for non-existent skill
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("unmanage")
        .arg("nonexistent-skill")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(contains("not found"));
}

// Additional test: unmanage with invalid agent name
#[test]
#[cfg(unix)]
fn test_unmanage_invalid_agent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Setup directories
    let agent_dir = temp_dir.path().join("agents");
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    let config_dir = temp_dir.path().join(".sikil");

    fs::create_dir_all(&agent_dir).expect("Failed to create agent dir");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    let skill_name = "invalid-agent-test";

    // Create skill in repo and symlink
    create_skill_in_repo(&repo_dir, skill_name);
    create_managed_symlink(&agent_dir, &repo_dir, skill_name);

    // Create config
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude/skills"
"#,
        agent_dir.display()
    );
    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    // Run unmanage with invalid agent
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path());
    cmd.arg("unmanage")
        .arg(skill_name)
        .arg("--agent")
        .arg("invalid-agent")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(contains("invalid"));
}

// Additional test: unmanage command help
#[test]
fn test_unmanage_help() {
    let mut cmd = sikil_cmd!();
    cmd.arg("unmanage")
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Unmanage a skill"))
        .stdout(contains("--agent"))
        .stdout(contains("--yes"));
}
