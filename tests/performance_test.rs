//! Performance benchmark tests
//!
//! This module tests performance characteristics of key commands.
//! Target: `sikil list` with 50 skills should complete in <500ms
//! Target: `sikil show` should complete in <200ms

use std::fs;
use std::time::{Duration, Instant};
use tempfile::TempDir;

mod common;
use common::{create_complete_skill_md, create_skill_dir};

fn setup_large_skill_set(count: usize) -> (TempDir, TempDir) {
    let home_dir = TempDir::new().expect("Failed to create temp home");
    let sikil_dir = home_dir.path().join(".sikil");
    fs::create_dir(&sikil_dir).expect("Failed to create .sikil");

    let repo_dir = sikil_dir.join("repo");
    fs::create_dir(&repo_dir).expect("Failed to create repo");

    let agent_dir = TempDir::new().expect("Failed to create temp agent dir");

    for i in 0..count {
        let skill_name = format!("test-skill-{:03}", i);

        let repo_skill_path = create_skill_dir(&repo_dir, &skill_name);
        create_complete_skill_md(
            &repo_skill_path,
            &skill_name,
            &format!("Test skill number {}", i),
        );

        let agent_skill_path = agent_dir.path().join(&skill_name);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&repo_skill_path, &agent_skill_path)
            .expect("Failed to create symlink");
    }

    (home_dir, agent_dir)
}

#[test]
fn benchmark_list_50_skills() {
    let (home_dir, agent_dir) = setup_large_skill_set(50);

    let config_path = home_dir.path().join(".sikil/config.toml");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude-code/skills"
"#,
        agent_dir.path().display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    let start = Instant::now();
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home_dir.path())
        .env("NO_COLOR", "1")
        .arg("list")
        .arg("--no-cache");

    let output = cmd.assert().success();
    let duration = start.elapsed();

    println!("List 50 skills took: {:?}", duration);

    assert!(
        duration < Duration::from_millis(500),
        "List command took {:?}, expected <500ms",
        duration
    );

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("test-skill-000"));
    assert!(stdout.contains("test-skill-049"));
}

#[test]
fn benchmark_show_command() {
    let (home_dir, agent_dir) = setup_large_skill_set(10);

    let config_path = home_dir.path().join(".sikil/config.toml");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude-code/skills"
"#,
        agent_dir.path().display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    let start = Instant::now();
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home_dir.path())
        .env("NO_COLOR", "1")
        .arg("show")
        .arg("test-skill-005")
        .arg("--no-cache");

    let output = cmd.assert().success();
    let duration = start.elapsed();

    println!("Show command took: {:?}", duration);

    assert!(
        duration < Duration::from_millis(200),
        "Show command took {:?}, expected <200ms",
        duration
    );

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("test-skill-005"));
}

#[test]
fn benchmark_list_with_cache() {
    let (home_dir, agent_dir) = setup_large_skill_set(50);

    let config_path = home_dir.path().join(".sikil/config.toml");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude-code/skills"
"#,
        agent_dir.path().display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home_dir.path())
        .env("NO_COLOR", "1")
        .arg("list")
        .arg("--no-cache")
        .assert()
        .success();

    let start = Instant::now();
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home_dir.path())
        .env("NO_COLOR", "1")
        .arg("list");

    cmd.assert().success();
    let cached_duration = start.elapsed();

    println!("Cached list took: {:?}", cached_duration);

    assert!(
        cached_duration < Duration::from_millis(100),
        "Cached list took {:?}, expected <100ms",
        cached_duration
    );
}

#[test]
fn benchmark_validate_command() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let skill_path = create_skill_dir(temp_dir.path(), "validate-test");
    create_complete_skill_md(&skill_path, "validate-test", "Test skill for validation");

    let start = Instant::now();
    let mut cmd = sikil_cmd!();
    cmd.arg("validate")
        .arg(skill_path.to_str().unwrap())
        .assert()
        .success();

    let duration = start.elapsed();

    println!("Validate command took: {:?}", duration);

    assert!(
        duration < Duration::from_millis(100),
        "Validate command took {:?}, expected <100ms",
        duration
    );
}

#[test]
fn benchmark_list_json_output() {
    let (home_dir, agent_dir) = setup_large_skill_set(50);

    let config_path = home_dir.path().join(".sikil/config.toml");

    let config_content = format!(
        r#"[agents.claude-code]
enabled = true
global_path = "{}"
workspace_path = ".claude-code/skills"
"#,
        agent_dir.path().display()
    );
    fs::write(&config_path, config_content).expect("Failed to write config");

    let start = Instant::now();
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", home_dir.path())
        .env("NO_COLOR", "1")
        .arg("list")
        .arg("--json")
        .arg("--no-cache");

    let output = cmd.assert().success();
    let duration = start.elapsed();

    println!("List --json with 50 skills took: {:?}", duration);

    assert!(
        duration < Duration::from_millis(500),
        "List --json took {:?}, expected <500ms",
        duration
    );

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.starts_with('{') || stdout.starts_with('['));
}
