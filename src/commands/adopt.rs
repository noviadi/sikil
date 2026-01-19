//! Adopt command implementation
//!
//! This module provides functionality for adopting existing unmanaged skills
//! into the Sikil management system. It moves the skill to the repository
//! and replaces the original with a symlink.

use crate::cli::output::Output;
use crate::core::config::Config;
use crate::core::errors::SikilError;
use crate::core::scanner::Scanner;
use crate::core::skill::Agent;
use crate::utils::atomic::atomic_move_dir;
use crate::utils::paths::{ensure_dir_exists, get_repo_path};
use crate::utils::symlink::create_symlink;
use anyhow::Result;
use fs_err as fs;

/// Arguments for the adopt command
#[derive(Debug, Clone)]
pub struct AdoptArgs {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Name of the skill to adopt
    pub name: String,
    /// Optional agent to adopt from (if multiple locations exist)
    pub from: Option<String>,
}

/// Executes the adopt command
///
/// This function:
/// 1. Finds unmanaged skill by name
/// 2. If multiple locations, requires `--from`
/// 3. Moves skill to `~/.sikil/repo/<name>/`
/// 4. Replaces original with symlink
///
/// # Arguments
///
/// * `args` - Adopt arguments including name and optional agent
/// * `config` - Configuration for resolving agent paths
///
/// # Errors
///
/// Returns an error if:
/// - The skill is not found
/// - The skill is already managed
/// - Multiple locations exist without `--from`
/// - The skill name already exists in repo
/// - The move operation fails
/// - Symlink creation fails
///
/// # Examples
///
/// ```no_run
/// use sikil::commands::adopt::{execute_adopt, AdoptArgs};
/// use sikil::core::config::Config;
///
/// let config = Config::default();
/// let args = AdoptArgs {
///     json_mode: false,
///     name: "my-skill".to_string(),
///     from: None,
/// };
/// execute_adopt(args, &config).unwrap();
/// ```
pub fn execute_adopt(args: AdoptArgs, config: &Config) -> Result<()> {
    let output = Output::new(args.json_mode);

    // M3-E03-T01-S02: Parse --from agent if provided
    let from_agent = if let Some(from_str) = &args.from {
        Some(
            Agent::from_cli_name(from_str).ok_or_else(|| SikilError::ValidationError {
                reason: format!("invalid agent name: {}", from_str),
            })?,
        )
    } else {
        None
    };

    // M3-E03-T01-S03: Find unmanaged skill by name across all agents
    let scanner = Scanner::new(config.clone());
    let scan_result = scanner.scan_all_agents();

    // Find all installations of the named skill
    let skill_installations: Vec<_> = scan_result
        .all_skills()
        .into_iter()
        .filter(|s| s.metadata.name == args.name)
        .flat_map(|s| s.installations)
        .collect();

    if skill_installations.is_empty() {
        return Err(SikilError::SkillNotFound {
            name: args.name.clone(),
        }
        .into());
    }

    // M3-E03-T01-S04: If multiple locations, require --from
    let target_installation = if skill_installations.len() > 1 {
        let from = from_agent.ok_or_else(|| {
            let locations: Vec<_> = skill_installations
                .iter()
                .map(|i| format!("{} ({})", i.agent, i.path.display()))
                .collect();
            SikilError::ValidationError {
                reason: format!(
                    "skill '{}' found in multiple locations. Use --from to specify:\n{}",
                    args.name,
                    locations.join("\n")
                ),
            }
        })?;

        skill_installations
            .into_iter()
            .find(|i| i.agent == from)
            .ok_or_else(|| SikilError::ValidationError {
                reason: format!("skill '{}' not found for agent {}", args.name, from),
            })?
    } else {
        skill_installations
            .into_iter()
            .next()
            .expect("should have one installation")
    };

    let source_path = &target_installation.path;

    // M3-E03-T02-S01: Check skill is unmanaged (not a symlink to repo)
    if source_path.is_symlink() {
        return Err(SikilError::ValidationError {
            reason: format!(
                "skill '{}' at {} is already managed (symlink)",
                args.name,
                source_path.display()
            ),
        }
        .into());
    }

    // M3-E03-T02-S02: Check skill name not in repo
    let repo_path = get_repo_path();
    ensure_dir_exists(&repo_path).map_err(|_e| SikilError::PermissionDenied {
        operation: "create repo directory".to_string(),
        path: repo_path.clone(),
    })?;

    let dest_path = repo_path.join(&args.name);

    if dest_path.exists() {
        return Err(SikilError::AlreadyExists {
            resource: format!("skill '{}' in repository", args.name),
        }
        .into());
    }

    // Start the adoption process
    if !args.json_mode {
        output.print_info(&format!("Adopting skill: {}", args.name));
        output.print_info(&format!("Source: {}", source_path.display()));
        output.print_info(&format!("Destination: {}", dest_path.display()));
        output.print_info(&format!("Agent: {}", target_installation.agent));
        output.print_info("");
    }

    // M3-E03-T01-S05: Move skill to ~/.sikil/repo/<name>/
    if !args.json_mode {
        output.print_info("Moving skill to repository...");
    }

    // M3-E03-T02-S03: Atomic move with rollback on failure
    atomic_move_dir(source_path, &dest_path)?;

    if !args.json_mode {
        output.print_success("Skill moved to repository");
    }

    // M3-E03-T01-S06: Replace original with symlink
    if !args.json_mode {
        output.print_info("Creating symlink...");
    }

    match create_symlink(&dest_path, source_path) {
        Ok(()) => {
            if !args.json_mode {
                output.print_success(&format!("Symlink created at {}", source_path.display()));
            }
        }
        Err(e) => {
            // Rollback: move the skill back
            let _ = fs::remove_dir_all(&dest_path);
            let _ = atomic_move_dir(&dest_path, source_path);
            return Err(e.into());
        }
    }

    if !args.json_mode {
        output.print_info("");
        output.print_success(&format!("Successfully adopted {}", args.name));
        output.print_info(&format!("Managed at: {}", dest_path.display()));
        output.print_info(&format!("Symlink at: {}", source_path.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    /// Helper to create a test skill with SKILL.md
    fn create_test_skill(dir: &Path, name: &str) {
        let content = format!(
            r#"---
name: {}
description: A test skill for adoption
version: 1.0.0
---

# Test Skill

This is a test skill."#,
            name
        );
        fs::write(dir.join("SKILL.md"), content).unwrap();
        fs::write(dir.join("script.sh"), "#!/bin/sh\necho test").unwrap();
    }

    /// Helper to create a test config with custom paths
    fn create_test_config_with_paths(agent_path: &Path) -> Config {
        let mut config = Config::new();
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                agent_path.to_path_buf(),
                PathBuf::from(".skills"),
            ),
        );
        config
    }

    #[test]
    fn test_adopt_args() {
        let args = AdoptArgs {
            json_mode: true,
            name: "my-skill".to_string(),
            from: Some("claude-code".to_string()),
        };

        assert!(args.json_mode);
        assert_eq!(args.name, "my-skill");
        assert_eq!(args.from, Some("claude-code".to_string()));
    }

    #[test]
    fn test_adopt_args_no_from() {
        let args = AdoptArgs {
            json_mode: false,
            name: "my-skill".to_string(),
            from: None,
        };

        assert!(!args.json_mode);
        assert_eq!(args.name, "my-skill");
        assert!(args.from.is_none());
    }

    // M3-E03-T04-S01: Integration test: adopt single unmanaged skill
    #[test]
    fn test_adopt_single_unmanaged_skill() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        // Get the actual repo path
        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        // Create unmanaged skill in agent directory
        let skill_path = agent_dir.join("adopt-me");
        fs::create_dir(&skill_path).unwrap();
        create_test_skill(&skill_path, "adopt-me");

        let config = create_test_config_with_paths(&agent_dir);
        let args = AdoptArgs {
            json_mode: false,
            name: "adopt-me".to_string(),
            from: None,
        };

        let result = execute_adopt(args, &config);
        assert!(result.is_ok(), "Adoption should succeed");

        // Verify skill was moved to repo
        let skill_in_repo = repo_dir.join("adopt-me");
        assert!(skill_in_repo.exists());
        assert!(skill_in_repo.join("SKILL.md").exists());

        // Verify symlink was created
        #[cfg(unix)]
        {
            assert!(skill_path.exists());
            assert!(skill_path.is_symlink());
        }

        // Cleanup
        let _ = fs::remove_dir_all(&skill_in_repo);
        #[cfg(unix)]
        let _ = fs::remove_file(&skill_path);
    }

    // M3-E03-T04-S02: Integration test: adopt with --from
    #[test]
    fn test_adopt_with_from_flag() {
        let temp_dir = TempDir::new().unwrap();
        let agent1_dir = temp_dir.path().join("agents1");
        let agent2_dir = temp_dir.path().join("agents2");
        fs::create_dir_all(&agent1_dir).unwrap();
        fs::create_dir_all(&agent2_dir).unwrap();

        // Get the actual repo path
        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        // Create same skill in both agent directories
        for agent_dir in [&agent1_dir, &agent2_dir] {
            let skill_path = agent_dir.join("multi-location");
            fs::create_dir(&skill_path).unwrap();
            create_test_skill(&skill_path, "multi-location");
        }

        let mut config = Config::new();
        config.insert_agent(
            "windsurf".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                agent1_dir.clone(),
                PathBuf::from(".skills"),
            ),
        );
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                agent2_dir.clone(),
                PathBuf::from(".skills"),
            ),
        );

        // Adopt from specific agent
        let args = AdoptArgs {
            json_mode: false,
            name: "multi-location".to_string(),
            from: Some("windsurf".to_string()),
        };

        let result = execute_adopt(args, &config);
        assert!(result.is_ok(), "Adoption with --from should succeed");

        // Verify only windsurf location was replaced with symlink
        #[cfg(unix)]
        {
            let skill1 = agent1_dir.join("multi-location");
            let skill2 = agent2_dir.join("multi-location");
            assert!(skill1.is_symlink());
            assert!(!skill2.is_symlink());
        }

        // Cleanup
        let _ = fs::remove_dir_all(repo_dir.join("multi-location"));
        #[cfg(unix)]
        {
            let _ = fs::remove_file(agent1_dir.join("multi-location"));
            let _ = fs::remove_dir_all(agent2_dir.join("multi-location"));
        }
    }

    // M3-E03-T04-S03: Integration test: adopt multiple locations without --from
    #[test]
    fn test_adopt_multiple_locations_without_from() {
        let temp_dir = TempDir::new().unwrap();
        let agent1_dir = temp_dir.path().join("agents1");
        let agent2_dir = temp_dir.path().join("agents2");
        fs::create_dir_all(&agent1_dir).unwrap();
        fs::create_dir_all(&agent2_dir).unwrap();

        // Create same skill in both agent directories
        for agent_dir in [&agent1_dir, &agent2_dir] {
            let skill_path = agent_dir.join("multi-skill");
            fs::create_dir(&skill_path).unwrap();
            create_test_skill(&skill_path, "multi-skill");
        }

        let mut config = Config::new();
        config.insert_agent(
            "windsurf".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                agent1_dir.clone(),
                PathBuf::from(".skills"),
            ),
        );
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                agent2_dir.clone(),
                PathBuf::from(".skills"),
            ),
        );

        // Try to adopt without --from
        let args = AdoptArgs {
            json_mode: false,
            name: "multi-skill".to_string(),
            from: None,
        };

        let result = execute_adopt(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("multiple locations") || err_msg.contains("--from"));

        // Cleanup
        let _ = fs::remove_dir_all(agent1_dir.join("multi-skill"));
        let _ = fs::remove_dir_all(agent2_dir.join("multi-skill"));
    }

    // M3-E03-T04-S04: Integration test: adopt already managed
    #[test]
    fn test_adopt_already_managed() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        // Get the actual repo path
        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        // Create managed skill in repo
        let managed_skill = repo_dir.join("managed-skill");
        fs::create_dir(&managed_skill).unwrap();
        create_test_skill(&managed_skill, "managed-skill");

        // Create symlink in agent directory (already managed)
        let symlink_path = agent_dir.join("managed-skill");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&managed_skill, &symlink_path).unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = AdoptArgs {
            json_mode: false,
            name: "managed-skill".to_string(),
            from: None,
        };

        let result = execute_adopt(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("already managed") || err_msg.contains("symlink"));

        // Cleanup
        #[cfg(unix)]
        let _ = fs::remove_file(&symlink_path);
        let _ = fs::remove_dir_all(&managed_skill);
    }

    // M3-E03-T04-S05: Integration test: adopt non-existent skill
    #[test]
    fn test_adopt_non_existent_skill() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = AdoptArgs {
            json_mode: false,
            name: "non-existent".to_string(),
            from: None,
        };

        let result = execute_adopt(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("Skill"));
    }

    // M3-E03-T02-S03: Test atomic move with rollback on failure
    #[test]
    fn test_adopt_rollback_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        // Get the actual repo path
        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        // Create unmanaged skill
        let skill_path = agent_dir.join("rollback-test");
        fs::create_dir(&skill_path).unwrap();
        create_test_skill(&skill_path, "rollback-test");

        // Create a file at the destination repo path to cause atomic_move_dir to fail
        let repo_skill_path = repo_dir.join("rollback-test");
        fs::write(&repo_skill_path, "this is a file, not a directory").unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = AdoptArgs {
            json_mode: false,
            name: "rollback-test".to_string(),
            from: None,
        };

        // The atomic_move_dir should fail because a file exists at the destination
        let result = execute_adopt(args, &config);
        assert!(result.is_err());

        // Verify the original skill directory still exists (rollback happened)
        assert!(skill_path.exists());
        assert!(skill_path.join("SKILL.md").exists());

        // Cleanup
        let _ = fs::remove_file(&repo_skill_path);
        let _ = fs::remove_dir_all(&skill_path);
    }

    // M3-E03-T02-S02: Test skill name already in repo
    #[test]
    fn test_adopt_skill_name_exists_in_repo() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        // Get the actual repo path
        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        // Use unique skill name to avoid test interference
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let skill_name = format!("existing-skill-{}", unique_id);

        // Clean up any leftover from previous failed test runs first
        let existing_skill = repo_dir.join(&skill_name);
        let _ = fs::remove_dir_all(&existing_skill);

        // Create skill already in repo
        fs::create_dir(&existing_skill).unwrap();
        create_test_skill(&existing_skill, &skill_name);

        // Create unmanaged skill with same name
        let skill_path = agent_dir.join(&skill_name);
        fs::create_dir(&skill_path).unwrap();
        create_test_skill(&skill_path, &skill_name);

        let config = create_test_config_with_paths(&agent_dir);
        let args = AdoptArgs {
            json_mode: false,
            name: skill_name.clone(),
            from: None,
        };

        let result = execute_adopt(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Already exists") || err_msg.contains("repository"));

        // Cleanup
        let _ = fs::remove_dir_all(&existing_skill);
        let _ = fs::remove_dir_all(&skill_path);
    }

    // M3-E03-T02-S04: Test preserves permissions and structure
    #[test]
    fn test_adopt_preserves_structure() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        // Get the actual repo path
        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        // Create skill with nested structure
        let skill_path = agent_dir.join("structured-skill");
        fs::create_dir_all(&skill_path).unwrap();
        create_test_skill(&skill_path, "structured-skill");

        // Create subdirectories
        fs::create_dir_all(skill_path.join("scripts")).unwrap();
        fs::create_dir_all(skill_path.join("references")).unwrap();
        fs::write(
            skill_path.join("scripts").join("run.sh"),
            "#!/bin/sh\necho run",
        )
        .unwrap();
        fs::write(
            skill_path.join("references").join("doc.md"),
            "# Documentation",
        )
        .unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = AdoptArgs {
            json_mode: false,
            name: "structured-skill".to_string(),
            from: None,
        };

        let result = execute_adopt(args, &config);
        assert!(result.is_ok());

        // Verify structure was preserved
        let skill_in_repo = repo_dir.join("structured-skill");
        assert!(skill_in_repo.join("SKILL.md").exists());
        assert!(skill_in_repo.join("scripts").exists());
        assert!(skill_in_repo.join("scripts").join("run.sh").exists());
        assert!(skill_in_repo.join("references").exists());
        assert!(skill_in_repo.join("references").join("doc.md").exists());

        // Cleanup
        let _ = fs::remove_dir_all(&skill_in_repo);
        #[cfg(unix)]
        let _ = fs::remove_file(&skill_path);
    }
}
