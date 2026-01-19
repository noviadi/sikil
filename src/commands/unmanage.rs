//! Unmanage command implementation
//!
//! This module provides functionality for unmanaging skills - converting
//! managed skills (symlinks to ~/.sikil/repo/) back to unmanaged skills
//! (physical directories in agent directories).

use crate::cli::output::Output;
use crate::core::config::Config;
use crate::core::errors::SikilError;
use crate::core::scanner::Scanner;
use crate::core::skill::Agent;
use crate::utils::atomic::copy_skill_dir;
use crate::utils::paths::get_repo_path;
use anyhow::Result;
use fs_err as fs;

/// Arguments for the unmanage command
#[derive(Debug, Clone)]
pub struct UnmanageArgs {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Name of the skill to unmanage
    pub name: String,
    /// Optional agent to unmanage from (for partial unmanage)
    pub agent: Option<String>,
    /// Skip confirmation prompt
    pub yes: bool,
}

/// Executes the unmanage command
///
/// This function:
/// 1. Finds managed skill by name
/// 2. If `--agent` specified, unmanages only that agent
/// 3. Removes symlink and copies content from repo to location
/// 4. If all symlinks removed, deletes from repo
///
/// # Arguments
///
/// * `args` - Unmanage arguments including name, optional agent, and yes flag
/// * `config` - Configuration for resolving agent paths
///
/// # Errors
///
/// Returns an error if:
/// - The skill is not found
/// - The skill is not managed
/// - No symlinks exist to remove
/// - The copy operation fails
/// - Symlink removal fails
///
/// # Examples
///
/// ```no_run
/// use sikil::commands::unmanage::{execute_unmanage, UnmanageArgs};
/// use sikil::core::config::Config;
///
/// let config = Config::default();
/// let args = UnmanageArgs {
///     json_mode: false,
///     name: "my-skill".to_string(),
///     agent: None,
///     yes: false,
/// };
/// execute_unmanage(args, &config).unwrap();
/// ```
pub fn execute_unmanage(args: UnmanageArgs, config: &Config) -> Result<()> {
    let output = Output::new(args.json_mode);

    // M3-E04-T01-S03: Parse --agent if provided
    let target_agent = if let Some(agent_str) = &args.agent {
        Some(
            Agent::from_cli_name(agent_str).ok_or_else(|| SikilError::ValidationError {
                reason: format!("invalid agent name: {}", agent_str),
            })?,
        )
    } else {
        None
    };

    // M3-E04-T01-S03: Find managed skill by name
    let scanner = Scanner::new(config.clone());
    let scan_result = scanner.scan_all_agents();

    // Find all installations of the named skill
    let skill_installations: Vec<_> = scan_result
        .all_skills()
        .into_iter()
        .filter(|s| s.metadata.name == args.name)
        .collect();

    if skill_installations.is_empty() {
        return Err(SikilError::SkillNotFound {
            name: args.name.clone(),
        }
        .into());
    }

    // Get the first skill to check if it's managed
    let skill = &skill_installations[0];

    // M3-E04-T01-S03: Check skill is managed (exists in repo)
    if !skill.is_managed {
        return Err(SikilError::ValidationError {
            reason: format!("skill '{}' is not managed", args.name),
        }
        .into());
    }

    let repo_path = get_repo_path();
    let skill_repo_path = repo_path.join(&args.name);

    if !skill_repo_path.exists() {
        return Err(SikilError::ValidationError {
            reason: format!("skill '{}' not found in repository", args.name),
        }
        .into());
    }

    // Filter installations to only managed symlinks
    let managed_installations: Vec<_> = skill
        .installations
        .iter()
        .filter(|i| i.is_symlink == Some(true) && i.symlink_target.is_some())
        .collect();

    if managed_installations.is_empty() {
        return Err(SikilError::ValidationError {
            reason: format!("no managed installations found for skill '{}'", args.name),
        }
        .into());
    }

    // M3-E04-T01-S04: If --agent specified, unmanage only that agent
    let (installations_to_unmanage, is_all_installations) = if let Some(agent) = target_agent {
        let found: Vec<_> = managed_installations
            .iter()
            .filter(|i| i.agent == agent)
            .copied()
            .collect();

        if found.is_empty() {
            return Err(SikilError::ValidationError {
                reason: format!("skill '{}' is not managed for agent {}", args.name, agent),
            }
            .into());
        }

        // Found specific agent installations, not all
        (found, false)
    } else {
        // No --agent specified, unmanage all managed installations
        (managed_installations.clone(), true)
    };

    // Display what will be unmanaged
    if !args.json_mode {
        output.print_info(&format!("Unmanaging skill: {}", args.name));
        output.print_info(&format!("Repository: {}", skill_repo_path.display()));
        output.print_info("");
        output.print_info("The following installations will be converted to unmanaged:");
        for installation in &installations_to_unmanage {
            output.print_info(&format!(
                "  - {} ({})",
                installation.agent,
                installation.path.display()
            ));
        }

        // Check if this will remove all installations
        if is_all_installations || installations_to_unmanage.len() == managed_installations.len() {
            output.print_warning(
                "This will remove all installations. The skill will be deleted from the repository.",
            );
        }
        output.print_info("");
    }

    // M3-E04-T02-S02: Prompt for confirmation (unless --yes)
    // Note: This is handled by M3-E04-T02, but we'll do a simple check here
    if !args.yes && !args.json_mode {
        // For now, we'll proceed - proper confirmation handling is in M3-E04-T02
        output.print_info("Proceeding with unmanage operation...");
    }

    let mut unmanaged_count = 0;
    let mut failed_installations = Vec::new();

    // M3-E04-T01-S05: Remove symlink and copy content from repo
    for installation in &installations_to_unmanage {
        let symlink_path = &installation.path;

        // Remove the symlink
        if let Err(e) = fs::remove_file(symlink_path) {
            let err_msg = format!(
                "failed to remove symlink at {}: {}",
                symlink_path.display(),
                e
            );
            output.print_error(&err_msg);
            failed_installations.push(symlink_path.clone());
            continue;
        }

        // Copy content from repo to the original location
        if let Err(e) = copy_skill_dir(&skill_repo_path, symlink_path) {
            let err_msg = format!("failed to copy skill to {}: {}", symlink_path.display(), e);
            output.print_error(&err_msg);
            failed_installations.push(symlink_path.clone());
            // Try to restore the symlink
            let _ = fs::remove_dir_all(symlink_path);
            if let Some(target) = &installation.symlink_target {
                let _ = fs_err::os::unix::fs::symlink(target, symlink_path);
            }
            continue;
        }

        unmanaged_count += 1;

        if !args.json_mode {
            output.print_success(&format!(
                "Unmanaged from {} ({})",
                installation.agent,
                symlink_path.display()
            ));
        }
    }

    if !failed_installations.is_empty() {
        return Err(SikilError::SymlinkError {
            reason: format!(
                "failed to unmanage {} installation(s)",
                failed_installations.len()
            ),
            source: None,
        }
        .into());
    }

    // M3-E04-T01-S06: If all symlinks removed, delete from repo
    if unmanaged_count == managed_installations.len() {
        if !args.json_mode {
            output.print_info("All installations unmanaged, removing from repository...");
        }

        if let Err(_e) = fs::remove_dir_all(&skill_repo_path) {
            return Err(SikilError::PermissionDenied {
                operation: "remove skill from repository".to_string(),
                path: skill_repo_path,
            }
            .into());
        }

        if !args.json_mode {
            output.print_success(&format!("Removed '{}' from repository", args.name));
        }
    } else if !args.json_mode {
        output.print_info(&format!(
            "Still managed by {} other agent(s)",
            managed_installations.len() - unmanaged_count
        ));
    }

    if !args.json_mode {
        output.print_info("");
        output.print_success(&format!(
            "Successfully unmanaged '{}' from {} agent(s)",
            args.name, unmanaged_count
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a test skill with SKILL.md
    fn create_test_skill(dir: &PathBuf, name: &str) {
        let content = format!(
            r#"---
name: {}
description: A test skill for unmanaging
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
    fn create_test_config_with_paths(agent_path: &PathBuf) -> Config {
        let mut config = Config::new();
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                agent_path.clone(),
                PathBuf::from(".skills"),
            ),
        );
        config
    }

    /// Helper to setup a managed skill for testing
    fn setup_managed_skill(repo_dir: &PathBuf, agent_dir: &PathBuf, skill_name: &str) -> PathBuf {
        // Create skill in repo
        let skill_repo_path = repo_dir.join(skill_name);
        fs::create_dir_all(&skill_repo_path).unwrap();
        create_test_skill(&skill_repo_path, skill_name);

        // Create symlink in agent directory
        let skill_link_path = agent_dir.join(skill_name);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&skill_repo_path, &skill_link_path).unwrap();

        skill_repo_path
    }

    #[test]
    fn test_unmanage_args() {
        let args = UnmanageArgs {
            json_mode: true,
            name: "my-skill".to_string(),
            agent: Some("claude-code".to_string()),
            yes: false,
        };

        assert!(args.json_mode);
        assert_eq!(args.name, "my-skill");
        assert_eq!(args.agent, Some("claude-code".to_string()));
        assert!(!args.yes);
    }

    #[test]
    fn test_unmanage_args_no_agent() {
        let args = UnmanageArgs {
            json_mode: false,
            name: "my-skill".to_string(),
            agent: None,
            yes: true,
        };

        assert!(!args.json_mode);
        assert_eq!(args.name, "my-skill");
        assert!(args.agent.is_none());
        assert!(args.yes);
    }

    // M3-E04-T04-S01: Integration test: unmanage all agents
    #[test]
    fn test_unmanage_all_agents() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        // Setup managed skill
        let skill_name = "unmanage-all";
        let _skill_repo_path = setup_managed_skill(&repo_dir, &agent_dir, skill_name);

        let config = create_test_config_with_paths(&agent_dir);
        let args = UnmanageArgs {
            json_mode: false,
            name: skill_name.to_string(),
            agent: None,
            yes: true,
        };

        let result = execute_unmanage(args, &config);
        assert!(result.is_ok(), "Unmanage should succeed");

        // Verify symlink was removed and replaced with physical directory
        let skill_path = agent_dir.join(skill_name);
        assert!(skill_path.exists());
        assert!(!skill_path.is_symlink());
        assert!(skill_path.join("SKILL.md").exists());

        // Verify skill was removed from repo
        let skill_in_repo = repo_dir.join(skill_name);
        assert!(!skill_in_repo.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&skill_path);
    }

    // M3-E04-T04-S02: Integration test: unmanage specific agent
    #[test]
    fn test_unmanage_specific_agent() {
        let temp_dir = TempDir::new().unwrap();
        let agent1_dir = temp_dir.path().join("agents1");
        let agent2_dir = temp_dir.path().join("agents2");
        fs::create_dir_all(&agent1_dir).unwrap();
        fs::create_dir_all(&agent2_dir).unwrap();

        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        let skill_name = "partial-unmanage";

        // Create skill in repo
        let skill_repo_path = repo_dir.join(skill_name);
        fs::create_dir_all(&skill_repo_path).unwrap();
        create_test_skill(&skill_repo_path, skill_name);

        // Create symlinks in both agent directories
        let link1 = agent1_dir.join(skill_name);
        let link2 = agent2_dir.join(skill_name);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&skill_repo_path, &link1).unwrap();
            std::os::unix::fs::symlink(&skill_repo_path, &link2).unwrap();
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

        // Unmanage from only windsurf
        let args = UnmanageArgs {
            json_mode: false,
            name: skill_name.to_string(),
            agent: Some("windsurf".to_string()),
            yes: true,
        };

        let result = execute_unmanage(args, &config);
        assert!(result.is_ok());

        // Verify windsurf link was replaced with physical directory
        #[cfg(unix)]
        {
            assert!(!link1.is_symlink());
            assert!(link1.join("SKILL.md").exists());
        }

        // Verify claude-code link is still a symlink
        #[cfg(unix)]
        {
            assert!(link2.is_symlink());
        }

        // Verify skill is still in repo
        assert!(skill_repo_path.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&link1);
        let _ = fs::remove_file(&link2);
        let _ = fs::remove_dir_all(&skill_repo_path);
    }

    // M3-E04-T04-S03: Integration test: unmanage with --yes
    #[test]
    fn test_unmanage_with_yes_flag() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        let skill_name = "yes-flag-test";
        let _skill_repo_path = setup_managed_skill(&repo_dir, &agent_dir, skill_name);

        let config = create_test_config_with_paths(&agent_dir);
        let args = UnmanageArgs {
            json_mode: false,
            name: skill_name.to_string(),
            agent: None,
            yes: true, // Skip confirmation
        };

        let result = execute_unmanage(args, &config);
        assert!(result.is_ok());

        // Verify unmanage completed
        let skill_path = agent_dir.join(skill_name);
        assert!(skill_path.exists());
        assert!(!skill_path.is_symlink());

        // Cleanup
        let _ = fs::remove_dir_all(&skill_path);
    }

    // M3-E04-T04-S04: Integration test: unmanage unmanaged skill
    #[test]
    fn test_unmanage_unmanaged_skill() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        // Create unmanaged skill (physical directory, not symlink)
        let skill_name = "unmanaged-skill";
        let skill_path = agent_dir.join(skill_name);
        fs::create_dir(&skill_path).unwrap();
        create_test_skill(&skill_path, skill_name);

        let config = create_test_config_with_paths(&agent_dir);
        let args = UnmanageArgs {
            json_mode: false,
            name: skill_name.to_string(),
            agent: None,
            yes: true,
        };

        let result = execute_unmanage(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not managed") || err_msg.contains("not found in repository"));

        // Cleanup
        let _ = fs::remove_dir_all(&skill_path);
    }

    #[test]
    fn test_unmanage_non_existent_skill() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = UnmanageArgs {
            json_mode: false,
            name: "non-existent".to_string(),
            agent: None,
            yes: true,
        };

        let result = execute_unmanage(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("Skill"));
    }

    #[test]
    fn test_unmanage_with_invalid_agent() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        let skill_name = "agent-test";
        let _skill_repo_path = setup_managed_skill(&repo_dir, &agent_dir, skill_name);

        let config = create_test_config_with_paths(&agent_dir);
        let args = UnmanageArgs {
            json_mode: false,
            name: skill_name.to_string(),
            agent: Some("invalid-agent".to_string()),
            yes: true,
        };

        let result = execute_unmanage(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid"));

        // Cleanup
        let _ = fs::remove_dir_all(&repo_dir.join(skill_name));
        let _ = fs::remove_file(&agent_dir.join(skill_name));
    }

    #[test]
    fn test_unmanage_skill_not_managed_for_agent() {
        let temp_dir = TempDir::new().unwrap();
        let agent1_dir = temp_dir.path().join("agents1");
        let agent2_dir = temp_dir.path().join("agents2");
        fs::create_dir_all(&agent1_dir).unwrap();
        fs::create_dir_all(&agent2_dir).unwrap();

        let repo_dir = get_repo_path();
        fs::create_dir_all(&repo_dir).unwrap();

        let skill_name = "single-agent";

        // Create skill in repo
        let skill_repo_path = repo_dir.join(skill_name);
        fs::create_dir_all(&skill_repo_path).unwrap();
        create_test_skill(&skill_repo_path, skill_name);

        // Create symlink only in agent1
        let link1 = agent1_dir.join(skill_name);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&skill_repo_path, &link1).unwrap();

        // Create physical directory in agent2 (unmanaged)
        let link2 = agent2_dir.join(skill_name);
        fs::create_dir(&link2).unwrap();
        create_test_skill(&link2, skill_name);

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

        // Try to unmanage from claude-code (which has unmanaged installation)
        let args = UnmanageArgs {
            json_mode: false,
            name: skill_name.to_string(),
            agent: Some("claude-code".to_string()),
            yes: true,
        };

        let result = execute_unmanage(args, &config);
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not managed"));

        // Cleanup
        let _ = fs::remove_file(&link1);
        let _ = fs::remove_dir_all(&link2);
        let _ = fs::remove_dir_all(&skill_repo_path);
    }
}
