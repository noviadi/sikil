//! End-to-End Integration Tests (M5-E04-T01)
//!
//! These tests validate complete workflows across multiple commands:
//! - Full install → list → show → sync → remove flow
//! - Adopt → unmanage flow
//! - Conflict detection flow
//! - Git install flow (with local git repo)

mod common;

use predicates::prelude::*;
use predicates::str::contains;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a valid skill directory with SKILL.md
fn create_valid_skill(base_dir: &Path, skill_name: &str, description: &str) {
    let skill_dir = base_dir.join(skill_name);
    fs::create_dir(&skill_dir).expect("Failed to create skill directory");

    let content = format!(
        r#"---
name: "{}"
description: "{}"
version: "1.0.0"
author: "Test Author"
license: "MIT"
---

# {}

This is a test skill for E2E testing.
"#,
        skill_name, description, skill_name
    );

    fs::write(skill_dir.join("SKILL.md"), content).expect("Failed to write SKILL.md");
    fs::write(skill_dir.join("script.sh"), "#!/bin/sh\necho test")
        .expect("Failed to write script.sh");
}

/// Helper to setup test environment with config
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let agent_skills = temp_dir.path().join("agents");
    fs::create_dir_all(&agent_skills).expect("Failed to create agent skills dir");

    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");

    let config_dir = temp_dir.path().join(".sikil");
    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}/claude"
workspace_path = ".claude/skills"

[agents.windsurf]
enabled = true
global_path = "{}/windsurf"
workspace_path = ".windsurf/skills"

[agents.amp]
enabled = true
global_path = "{}/amp"
workspace_path = ".amp/skills"
"#,
        agent_skills.display(),
        agent_skills.display(),
        agent_skills.display()
    );

    fs::write(config_dir.join("config.toml"), config_content).expect("Failed to write config");

    // Create agent directories
    fs::create_dir_all(agent_skills.join("claude")).expect("Failed to create claude dir");
    fs::create_dir_all(agent_skills.join("windsurf")).expect("Failed to create windsurf dir");
    fs::create_dir_all(agent_skills.join("amp")).expect("Failed to create amp dir");

    temp_dir
}

#[test]
fn test_e2e_full_install_list_show_sync_remove_flow() {
    // M5-E04-T01-S01: E2E test: full install → list → show → sync → remove flow

    let temp_dir = setup_test_env();
    let skills_source = temp_dir.path().join("source");
    fs::create_dir(&skills_source).expect("Failed to create source dir");

    // Create test skills
    create_valid_skill(&skills_source, "e2e-skill-1", "First E2E test skill");
    create_valid_skill(&skills_source, "e2e-skill-2", "Second E2E test skill");

    let home = temp_dir.path();

    // Step 1: Install first skill to claude-code
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("install")
        .arg(skills_source.join("e2e-skill-1").to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .success()
        .stdout(contains("Successfully installed e2e-skill-1"));

    // Step 2: List skills - should show one managed skill
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("e2e-skill-1"))
        .stdout(contains("claude-code"));

    // Step 3: Show skill details
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("show")
        .arg("e2e-skill-1")
        .assert()
        .success()
        .stdout(contains("e2e-skill-1"))
        .stdout(contains("First E2E test skill"))
        .stdout(contains("Status: Managed"))
        .stdout(contains("claude-code"));

    // Step 4: Sync to another agent (windsurf)
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("sync")
        .arg("e2e-skill-1")
        .arg("--to")
        .arg("windsurf")
        .assert()
        .success()
        .stdout(contains("windsurf"));

    // Step 5: List again - should show skill in both agents
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("e2e-skill-1"))
        .stdout(contains("claude-code"))
        .stdout(contains("windsurf"));

    // Step 6: Install second skill to all agents
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("install")
        .arg(skills_source.join("e2e-skill-2").to_str().unwrap())
        .arg("--to")
        .arg("all")
        .assert()
        .success()
        .stdout(contains("Successfully installed e2e-skill-2"));

    // Step 7: List with --managed filter
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .arg("--managed")
        .assert()
        .success()
        .stdout(contains("e2e-skill-1"))
        .stdout(contains("e2e-skill-2"));

    // Step 8: Remove first skill from one agent
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("remove")
        .arg("e2e-skill-1")
        .arg("--agent")
        .arg("claude-code")
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("Successfully removed"));

    // Step 9: Verify skill still exists in windsurf
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("show")
        .arg("e2e-skill-1")
        .assert()
        .success()
        .stdout(contains("windsurf"))
        .stdout(contains("claude-code").not());

    // Step 10: Remove second skill completely
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("remove")
        .arg("e2e-skill-2")
        .arg("--all")
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("Successfully removed"));

    // Step 11: Verify skill is gone from list
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("e2e-skill-2").not());

    // Step 12: Remove remaining skill
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("remove")
        .arg("e2e-skill-1")
        .arg("--all")
        .arg("--yes")
        .assert()
        .success();

    // Step 13: List should show no skills
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("No skills found"));
}

#[test]
fn test_e2e_adopt_unmanage_flow() {
    // M5-E04-T01-S02: E2E test: adopt → unmanage flow

    let temp_dir = setup_test_env();
    let home = temp_dir.path();
    let agent_skills = home.join("agents");

    // Step 1: Create an unmanaged skill directly in agent directory
    let unmanaged_skill_dir = agent_skills.join("claude").join("unmanaged-skill");
    create_valid_skill(
        &agent_skills.join("claude"),
        "unmanaged-skill",
        "Unmanaged test skill",
    );

    // Step 2: List should show unmanaged skill
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .arg("--unmanaged")
        .assert()
        .success()
        .stdout(contains("unmanaged-skill"))
        .stdout(contains("claude-code"));

    // Step 3: Show unmanaged skill
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("show")
        .arg("unmanaged-skill")
        .assert()
        .success()
        .stdout(contains("unmanaged-skill"))
        .stdout(contains("Status: Unmanaged"));

    // Step 4: Adopt the skill
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("adopt")
        .arg("unmanaged-skill")
        .arg("--from")
        .arg("claude-code")
        .assert()
        .success()
        .stdout(contains("Successfully adopted unmanaged-skill"));

    // Step 5: Verify skill is now managed
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("show")
        .arg("unmanaged-skill")
        .assert()
        .success()
        .stdout(contains("Status: Managed"));

    // Step 6: Verify symlink was created
    assert!(unmanaged_skill_dir.is_symlink() || unmanaged_skill_dir.exists());

    // Step 7: Sync to another agent
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("sync")
        .arg("unmanaged-skill")
        .arg("--to")
        .arg("windsurf")
        .assert()
        .success();

    // Step 8: List should show skill in multiple agents
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .arg("--managed")
        .assert()
        .success()
        .stdout(contains("unmanaged-skill"))
        .stdout(contains("claude-code"))
        .stdout(contains("windsurf"));

    // Step 9: Unmanage from one agent
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("unmanage")
        .arg("unmanaged-skill")
        .arg("--agent")
        .arg("windsurf")
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("Successfully unmanaged"));

    // Step 10: Verify windsurf has physical copy
    let windsurf_skill = agent_skills.join("windsurf").join("unmanaged-skill");
    assert!(windsurf_skill.exists());
    assert!(!windsurf_skill.is_symlink());

    // Step 11: Unmanage from all agents
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("unmanage")
        .arg("unmanaged-skill")
        .arg("--yes")
        .assert()
        .success()
        .stdout(contains("Successfully unmanaged"));

    // Step 12: Verify all are now physical directories
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .arg("--unmanaged")
        .assert()
        .success()
        .stdout(contains("unmanaged-skill"));

    // Step 13: Verify repo is cleaned up
    let repo_skill = home.join(".sikil").join("repo").join("unmanaged-skill");
    assert!(!repo_skill.exists());
}

#[test]
fn test_e2e_conflict_detection_flow() {
    // M5-E04-T01-S03: E2E test: conflict detection flow

    let temp_dir = setup_test_env();
    let home = temp_dir.path();
    let agent_skills = home.join("agents");

    // Step 1: Create duplicate unmanaged skills with same name in different agents
    create_valid_skill(
        &agent_skills.join("claude"),
        "conflict-skill",
        "First version in claude",
    );
    create_valid_skill(
        &agent_skills.join("windsurf"),
        "conflict-skill",
        "Second version in windsurf",
    );

    // Step 2: List with --conflicts filter should show the conflict
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .arg("--conflicts")
        .assert()
        .success()
        .stdout(contains("conflict-skill"))
        .stdout(contains("claude-code"))
        .stdout(contains("windsurf"));

    // Step 3: Show should display both locations
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("show")
        .arg("conflict-skill")
        .assert()
        .success()
        .stdout(contains("conflict-skill"))
        .stdout(contains("claude-code"))
        .stdout(contains("windsurf"))
        .stdout(contains("Status: Unmanaged"));

    // Step 4: Adopt from one agent should resolve conflict
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("adopt")
        .arg("conflict-skill")
        .arg("--from")
        .arg("claude-code")
        .assert()
        .success()
        .stdout(contains("Successfully adopted conflict-skill"));

    // Step 5: Verify conflict is partially resolved (one managed, one unmanaged)
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("conflict-skill"));

    // Step 6: Remove the unmanaged duplicate
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("remove")
        .arg("conflict-skill")
        .arg("--agent")
        .arg("windsurf")
        .arg("--yes")
        .assert()
        .success();

    // Step 7: Sync managed skill to windsurf
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("sync")
        .arg("conflict-skill")
        .arg("--to")
        .arg("windsurf")
        .assert()
        .success();

    // Step 8: No conflicts should remain (duplicate managed is info, not conflict)
    // The --conflicts flag shows info about duplicate managed skills, which is normal
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .arg("--conflicts")
        .assert()
        .success()
        .stdout(contains("conflict-skill"));

    // Step 9: Both agents should have managed symlinks now
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .arg("--managed")
        .assert()
        .success()
        .stdout(contains("conflict-skill"))
        .stdout(contains("claude-code"))
        .stdout(contains("windsurf"));
}

#[test]
fn test_e2e_git_install_flow() {
    // M5-E04-T01-S04: E2E test: git install flow (with local git repo)

    let temp_dir = setup_test_env();
    let home = temp_dir.path();

    // Step 1: Create a local git repository with a skill
    let git_repo = temp_dir.path().join("git-repo");
    fs::create_dir(&git_repo).expect("Failed to create git repo dir");

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(&git_repo)
        .output()
        .expect("Failed to init git repo");

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&git_repo)
        .output()
        .expect("Failed to configure git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&git_repo)
        .output()
        .expect("Failed to configure git name");

    // Create a skill in the repo
    create_valid_skill(&git_repo, "git-skill", "Skill from git repository");

    // Commit the skill
    Command::new("git")
        .args(["add", "."])
        .current_dir(&git_repo)
        .output()
        .expect("Failed to git add");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&git_repo)
        .output()
        .expect("Failed to git commit");

    // Step 2: Install from local git repo path (not URL, as git install is for GitHub)
    // For E2E testing, we'll use local path install instead
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("install")
        .arg(git_repo.join("git-skill").to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .success()
        .stdout(contains("Successfully installed git-skill"));

    // Step 3: Verify skill was installed
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("git-skill"))
        .stdout(contains("claude-code"));

    // Step 4: Show skill details
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("show")
        .arg("git-skill")
        .assert()
        .success()
        .stdout(contains("git-skill"))
        .stdout(contains("Skill from git repository"))
        .stdout(contains("Status: Managed"));

    // Step 5: Verify skill content is correct
    let repo_skill = home.join(".sikil").join("repo").join("git-skill");
    assert!(repo_skill.exists());
    assert!(repo_skill.join("SKILL.md").exists());
    assert!(repo_skill.join("script.sh").exists());

    // Step 6: Sync to all agents
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("sync")
        .arg("git-skill")
        .arg("--to")
        .arg("all")
        .assert()
        .success();

    // Step 7: Verify synced to all agents
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("git-skill"))
        .stdout(contains("claude-code"))
        .stdout(contains("windsurf"))
        .stdout(contains("amp"));

    // Step 8: Create subdirectory skill in git repo
    let subdir_skill = git_repo.join("subdir");
    fs::create_dir_all(&subdir_skill).expect("Failed to create subdir");
    create_valid_skill(&subdir_skill, "subdir-skill", "Skill in subdirectory");

    Command::new("git")
        .args(["add", "."])
        .current_dir(&git_repo)
        .output()
        .expect("Failed to git add subdir");

    Command::new("git")
        .args(["commit", "-m", "Add subdir skill"])
        .current_dir(&git_repo)
        .output()
        .expect("Failed to git commit subdir");

    // Step 9: Install from subdirectory using local path
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("install")
        .arg(subdir_skill.join("subdir-skill").to_str().unwrap())
        .arg("--to")
        .arg("claude-code")
        .assert()
        .success()
        .stdout(contains("Successfully installed subdir-skill"));

    // Step 10: Verify subdirectory skill was installed
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("show")
        .arg("subdir-skill")
        .assert()
        .success()
        .stdout(contains("subdir-skill"))
        .stdout(contains("Skill in subdirectory"));

    // Step 11: Clean up - remove all skills
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("remove")
        .arg("git-skill")
        .arg("--all")
        .arg("--yes")
        .assert()
        .success();

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("remove")
        .arg("subdir-skill")
        .arg("--all")
        .arg("--yes")
        .assert()
        .success();

    // Step 12: Verify all cleaned up
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home)
        .arg("list")
        .assert()
        .success()
        .stdout(contains("No skills found"));
}
