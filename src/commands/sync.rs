//! Sync command implementation
//!
//! This module provides functionality for syncing managed skills to agents
//! that don't have them yet.

use crate::cli::output::Output;
use crate::commands::parse_agent_selection;
use crate::core::config::Config;
use crate::core::errors::SikilError;
use crate::core::skill::Agent;
use crate::utils::paths::{ensure_dir_exists, get_repo_path};
use crate::utils::symlink::{create_symlink, is_symlink};
use anyhow::Result;
use fs_err as fs;
use std::path::{Path, PathBuf};

/// Arguments for the sync command
#[derive(Debug, Clone)]
pub struct SyncArgs {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Name of the skill to sync (optional, --all syncs all)
    pub name: Option<String>,
    /// Whether to sync all managed skills
    pub all: bool,
    /// Specific agents to sync to (optional)
    pub to: Option<String>,
}

/// Executes the sync command
///
/// This function:
/// 1. Finds managed skill in repo (error if not managed)
/// 2. Identifies agents missing the skill
/// 3. Creates symlinks to missing agents
/// 4. Skips agents that already have symlink
///
/// # Arguments
///
/// * `args` - Sync arguments including skill name, --all flag, and --to flag
/// * `config` - Configuration for resolving agent paths
/// * `repo_path` - Optional repo path override (for testing)
///
/// # Errors
///
/// Returns an error if:
/// - Neither --all nor a skill name is provided
/// - The skill is not found
/// - The skill is not managed (not in repo)
/// - Symlink creation fails
///
/// # Examples
///
/// ```no_run
/// use sikil::commands::sync::{execute_sync, SyncArgs};
/// use sikil::core::config::Config;
///
/// let config = Config::default();
/// let args = SyncArgs {
///     json_mode: false,
///     name: Some("my-skill".to_string()),
///     all: false,
///     to: Some("claude-code".to_string()),
/// };
/// execute_sync(args, &config, None).unwrap();
/// ```
pub fn execute_sync(args: SyncArgs, config: &Config, repo_path: Option<PathBuf>) -> Result<()> {
    let output = Output::new(args.json_mode);

    // Validate that either --all or a skill name is provided
    if !args.all && args.name.is_none() {
        return Err(SikilError::ValidationError {
            reason: "either --all or a skill name must be provided".to_string(),
        }
        .into());
    }

    // M4-E01-T01-S03: Find managed skill in repo
    let repo_path = repo_path.unwrap_or_else(get_repo_path);

    if args.all {
        // M4-E01-T02: Sync all managed skills
        sync_all_skills(args, config, &repo_path, &output)
    } else {
        // Sync a single skill
        let skill_name = args.name.as_ref().unwrap().clone();
        sync_single_skill(&skill_name, args, config, &repo_path, &output)
    }
}

/// Syncs a single managed skill to agents
fn sync_single_skill(
    skill_name: &str,
    args: SyncArgs,
    config: &Config,
    repo_path: &Path,
    output: &Output,
) -> Result<()> {
    let skill_repo_path = repo_path.join(skill_name);

    // Check if skill exists in repo
    if !skill_repo_path.exists() {
        return Err(SikilError::SkillNotFound {
            name: skill_name.to_string(),
        }
        .into());
    }

    // Verify it's a managed skill (has SKILL.md)
    let skill_md_path = skill_repo_path.join("SKILL.md");
    if !skill_md_path.exists() {
        return Err(SikilError::ValidationError {
            reason: format!(
                "'{}' is not a valid managed skill (SKILL.md not found)",
                skill_name
            ),
        }
        .into());
    }

    // Determine target agents
    let target_agents = if let Some(to_value) = &args.to {
        parse_agent_selection(Some(to_value), config)?
    } else {
        // If no --to specified, sync to all enabled agents
        parse_agent_selection(Some("all"), config)?
    };

    if target_agents.is_empty() {
        return Err(SikilError::ValidationError {
            reason: "no agents selected for sync".to_string(),
        }
        .into());
    }

    // M4-E01-T01-S04: Identify agents missing the skill
    let mut missing_agents: Vec<Agent> = Vec::new();
    let mut already_synced: Vec<Agent> = Vec::new();

    for agent in &target_agents {
        if let Some(agent_config) = config.get_agent(&agent.to_string()) {
            let agent_skill_path = agent_config.global_path.join(skill_name);

            if agent_skill_path.exists() {
                // M4-E01-T01-S06: Skip agents that already have symlink
                if is_symlink(&agent_skill_path) {
                    already_synced.push(*agent);
                } else {
                    // Physical directory exists - this is a conflict
                    return Err(SikilError::AlreadyExists {
                        resource: format!(
                            "skill '{}' at {} (use `sikil adopt` to manage it)",
                            skill_name,
                            agent_skill_path.display()
                        ),
                    }
                    .into());
                }
            } else {
                // Agent doesn't have the skill
                missing_agents.push(*agent);
            }
        }
    }

    // Check if there's nothing to do
    if missing_agents.is_empty() {
        if !args.json_mode {
            output.print_info(&format!(
                "Skill '{}' is already synced to all specified agents",
                skill_name
            ));
        }
        return Ok(());
    }

    // Display what will be synced
    if !args.json_mode {
        output.print_info(&format!("Syncing skill: {}", skill_name));
        output.print_info(&format!("Managed at: {}", skill_repo_path.display()));
        output.print_info(&format!("Adding to {} agent(s):", missing_agents.len()));
        for agent in &missing_agents {
            output.print_info(&format!("  - {}", agent));
        }
        if !already_synced.is_empty() {
            output.print_info(&format!(
                "Already synced to {} agent(s):",
                already_synced.len()
            ));
            for agent in &already_synced {
                output.print_info(&format!("  - {}", agent));
            }
        }
        output.print_info("");
    }

    // M4-E01-T01-S05: Create symlinks to missing agents
    let mut synced_count = 0;
    for agent in &missing_agents {
        if let Some(agent_config) = config.get_agent(&agent.to_string()) {
            let agent_skill_dir = &agent_config.global_path;

            // Ensure agent directory exists
            if let Err(e) = ensure_dir_exists(agent_skill_dir) {
                output.print_warning(&format!(
                    "Failed to create agent directory for {}: {}",
                    agent, e
                ));
                continue;
            }

            let symlink_path = agent_skill_dir.join(skill_name);

            match create_symlink(&skill_repo_path, &symlink_path) {
                Ok(()) => {
                    synced_count += 1;
                    if !args.json_mode {
                        output.print_success(&format!(
                            "Linked to {} at {}",
                            agent,
                            symlink_path.display()
                        ));
                    }
                }
                Err(e) => {
                    output.print_warning(&format!("Failed to create symlink for {}: {}", agent, e));
                }
            }
        }
    }

    if !args.json_mode {
        output.print_info("");
        if synced_count > 0 {
            output.print_success(&format!(
                "Successfully synced {} to {} agent(s)",
                skill_name, synced_count
            ));
        } else {
            output.print_warning("No agents were synced");
        }
    }

    Ok(())
}

/// Syncs all managed skills to agents
fn sync_all_skills(
    args: SyncArgs,
    config: &Config,
    repo_path: &Path,
    output: &Output,
) -> Result<()> {
    // Read all skills in repo
    let entries = match fs::read_dir(repo_path) {
        Ok(entries) => entries,
        Err(_) => {
            if !args.json_mode {
                output.print_info("No managed skills found in repository");
            }
            return Ok(());
        }
    };

    let mut skill_names: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let entry_path = entry.path();

        // Skip non-directories
        if !entry_path.is_dir() {
            continue;
        }

        // Skip hidden directories
        let dir_name = match entry_path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };
        if dir_name.starts_with('.') {
            continue;
        }

        // Check if it has SKILL.md (valid skill)
        let skill_md_path = entry_path.join("SKILL.md");
        if skill_md_path.exists() {
            skill_names.push(dir_name);
        }
    }

    if skill_names.is_empty() {
        if !args.json_mode {
            output.print_info("No managed skills found in repository");
        }
        return Ok(());
    }

    // Sync each skill
    let mut total_synced = 0;
    for skill_name in &skill_names {
        // Create sync args for this skill (without --all)
        let skill_args = SyncArgs {
            json_mode: args.json_mode,
            name: Some(skill_name.clone()),
            all: false,
            to: args.to.clone(),
        };

        if let Err(e) = sync_single_skill(skill_name, skill_args, config, repo_path, output) {
            output.print_warning(&format!("Failed to sync '{}': {}", skill_name, e));
        } else {
            total_synced += 1;
        }

        if !args.json_mode {
            output.print_info("");
        }
    }

    // M4-E01-T02-S03: Display summary of synced skills
    if !args.json_mode {
        output.print_success(&format!(
            "Synced {} of {} managed skill(s)",
            total_synced,
            skill_names.len()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test skill with SKILL.md
    fn create_test_skill(dir: &PathBuf, name: &str) {
        let content = format!(
            r#"---
name: {}
description: A test skill for syncing
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
                agent_path.to_path_buf(),
                PathBuf::from(".skills"),
            ),
        );
        config
    }

    #[test]
    fn test_sync_args() {
        let args = SyncArgs {
            json_mode: true,
            name: Some("my-skill".to_string()),
            all: false,
            to: Some("claude-code".to_string()),
        };

        assert!(args.json_mode);
        assert_eq!(args.name, Some("my-skill".to_string()));
        assert!(!args.all);
        assert_eq!(args.to, Some("claude-code".to_string()));
    }

    #[test]
    fn test_sync_args_all() {
        let args = SyncArgs {
            json_mode: false,
            name: None,
            all: true,
            to: None,
        };

        assert!(!args.json_mode);
        assert!(args.name.is_none());
        assert!(args.all);
        assert!(args.to.is_none());
    }

    // M4-E01-T01-S03: Find managed skill in repo (error if not managed)
    #[test]
    fn test_sync_fails_if_skill_not_in_repo() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = SyncArgs {
            json_mode: false,
            name: Some("non-existent-skill".to_string()),
            all: false,
            to: Some("claude-code".to_string()),
        };

        let result = sync_single_skill(
            "non-existent-skill",
            args,
            &config,
            &repo_dir,
            &Output::new(false),
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("SkillNotFound"));
    }

    // M4-E01-T01-S04: Identify agents missing the skill
    #[test]
    fn test_sync_identifies_missing_agents() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();

        // Create a managed skill in repo
        let skill_repo_path = repo_dir.join("test-sync-skill");
        fs::create_dir(&skill_repo_path).unwrap();
        create_test_skill(&skill_repo_path, "test-sync-skill");

        let config = create_test_config_with_paths(&agent_dir);
        let args = SyncArgs {
            json_mode: false,
            name: Some("test-sync-skill".to_string()),
            all: false,
            to: Some("claude-code".to_string()),
        };

        // Agent doesn't have the skill yet, so it should be identified as missing
        let result = sync_single_skill(
            "test-sync-skill",
            args,
            &config,
            &repo_dir,
            &Output::new(false),
        );
        assert!(result.is_ok());

        // Verify symlink was created
        let symlink_path = agent_dir.join("test-sync-skill");
        #[cfg(unix)]
        {
            assert!(symlink_path.exists());
            assert!(symlink_path.is_symlink());
        }

        // Cleanup
        let _ = fs::remove_file(&symlink_path);
        let _ = fs::remove_dir_all(&skill_repo_path);
    }

    // M4-E01-T01-S05: Create symlinks to missing agents
    #[test]
    fn test_sync_creates_symlinks() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();

        // Create a managed skill in repo
        let skill_repo_path = repo_dir.join("symlink-test-skill");
        fs::create_dir(&skill_repo_path).unwrap();
        create_test_skill(&skill_repo_path, "symlink-test-skill");

        let config = create_test_config_with_paths(&agent_dir);
        let args = SyncArgs {
            json_mode: false,
            name: Some("symlink-test-skill".to_string()),
            all: false,
            to: Some("claude-code".to_string()),
        };

        let result = sync_single_skill(
            "symlink-test-skill",
            args,
            &config,
            &repo_dir,
            &Output::new(false),
        );
        assert!(result.is_ok());

        // Verify symlink was created and points to correct location
        let symlink_path = agent_dir.join("symlink-test-skill");
        #[cfg(unix)]
        {
            assert!(symlink_path.exists());
            assert!(symlink_path.is_symlink());
            let target = fs::read_link(&symlink_path).unwrap();
            assert_eq!(target, skill_repo_path);
        }

        // Cleanup
        let _ = fs::remove_file(&symlink_path);
        let _ = fs::remove_dir_all(&skill_repo_path);
    }

    // M4-E01-T01-S06: Skip agents that already have symlink
    #[test]
    fn test_sync_skips_existing_symlinks() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();

        // Create a managed skill in repo
        let skill_repo_path = repo_dir.join("skip-test-skill");
        fs::create_dir(&skill_repo_path).unwrap();
        create_test_skill(&skill_repo_path, "skip-test-skill");

        // Create an existing symlink
        let symlink_path = agent_dir.join("skip-test-skill");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&skill_repo_path, &symlink_path).unwrap();
        }

        let config = create_test_config_with_paths(&agent_dir);
        let args = SyncArgs {
            json_mode: false,
            name: Some("skip-test-skill".to_string()),
            all: false,
            to: Some("claude-code".to_string()),
        };

        let result = sync_single_skill(
            "skip-test-skill",
            args,
            &config,
            &repo_dir,
            &Output::new(false),
        );
        assert!(result.is_ok());

        // Verify existing symlink is still there and valid
        #[cfg(unix)]
        {
            assert!(symlink_path.exists());
            assert!(symlink_path.is_symlink());
            let target = fs::read_link(&symlink_path).unwrap();
            assert_eq!(target, skill_repo_path);
        }

        // Cleanup
        let _ = fs::remove_file(&symlink_path);
        let _ = fs::remove_dir_all(&skill_repo_path);
    }

    #[test]
    fn test_sync_fails_without_name_or_all() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = SyncArgs {
            json_mode: false,
            name: None,
            all: false,
            to: None,
        };

        let result = execute_sync(args, &config, None);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("either --all or a skill name"));
    }

    #[test]
    fn test_sync_all_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();

        let config = create_test_config_with_paths(&agent_dir);
        let args = SyncArgs {
            json_mode: false,
            name: None,
            all: true,
            to: None,
        };

        let result = execute_sync(args, &config, Some(repo_dir.clone()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_all_multiple_skills() {
        let temp_dir = TempDir::new().unwrap();
        let agent_dir = temp_dir.path().join("agents");
        fs::create_dir_all(&agent_dir).unwrap();

        let repo_dir = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();

        // Create multiple managed skills in repo
        for i in 1..=3 {
            let skill_repo_path = repo_dir.join(format!("multi-skill-{}", i));
            fs::create_dir(&skill_repo_path).unwrap();
            create_test_skill(&skill_repo_path, &format!("multi-skill-{}", i));
        }

        let config = create_test_config_with_paths(&agent_dir);
        let args = SyncArgs {
            json_mode: false,
            name: None,
            all: true,
            to: Some("claude-code".to_string()),
        };

        let result = execute_sync(args, &config, Some(repo_dir.clone()));
        assert!(result.is_ok());

        // Verify all symlinks were created
        for i in 1..=3 {
            let symlink_path = agent_dir.join(format!("multi-skill-{}", i));
            #[cfg(unix)]
            {
                assert!(symlink_path.exists());
                assert!(symlink_path.is_symlink());
            }
            // Cleanup
            let _ = fs::remove_file(&symlink_path);
            let _ = fs::remove_dir_all(repo_dir.join(format!("multi-skill-{}", i)));
        }
    }
}
