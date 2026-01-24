//! List command implementation
//!
//! This module provides functionality for listing installed Agent Skills
//! across all configured agents.

use crate::cli::output::Output;
use crate::core::config::Config;
use crate::core::conflicts;
use crate::core::scanner::Scanner;
use crate::core::skill::{Agent, Scope, Skill};
use anyhow::Result;

/// Arguments for the list command
#[derive(Debug, Clone)]
pub struct ListArgs {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Whether to disable cache
    pub no_cache: bool,
    /// Filter by agent name
    pub agent_filter: Option<Agent>,
    /// Filter to show only managed skills
    pub managed_only: bool,
    /// Filter to show only unmanaged skills
    pub unmanaged_only: bool,
    /// Filter to show only conflicting skills
    pub conflicts_only: bool,
    /// Filter to show only duplicate skills
    pub duplicates_only: bool,
    /// Enable verbose output (show info-level conflicts)
    pub verbose: bool,
}

/// Output format for a single skill in the list
#[derive(Debug, Clone, serde::Serialize)]
pub struct ListSkillOutput {
    /// Skill name from metadata
    pub name: String,
    /// Directory name (if different from name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_name: Option<String>,
    /// Description
    pub description: String,
    /// Whether this skill is managed
    pub managed: bool,
    /// Agents where this skill is installed
    pub installations: Vec<ListInstallationOutput>,
}

/// Output format for a single installation
#[derive(Debug, Clone, serde::Serialize)]
pub struct ListInstallationOutput {
    /// Agent name
    pub agent: String,
    /// Scope (global or workspace)
    pub scope: String,
}

/// Gets information about disabled default agents
///
/// Returns a list of (agent_name, global_path) tuples for default agents
/// that are disabled in the current config.
fn get_disabled_default_agents(config: &Config) -> Vec<(String, String)> {
    let default_config = Config::default();
    let mut disabled = Vec::new();

    for (agent_name, default_agent) in &default_config.agents {
        // Only check default agents
        if let Some(current_agent) = config.get_agent(agent_name) {
            // If this agent is disabled but is enabled by default
            if !current_agent.enabled && default_agent.enabled {
                disabled.push((
                    agent_name.clone(),
                    current_agent.global_path.to_string_lossy().to_string(),
                ));
            }
        }
    }

    disabled.sort_by_key(|(name, _)| name.clone());
    disabled
}

/// Executes the list command
///
/// This function:
/// 1. Creates a scanner with the given configuration
/// 2. Scans all agent directories for skills
/// 3. Groups skills by managed/unmanaged status
/// 4. Formats output with skill name, agents, and scope
///
/// # Arguments
///
/// * `args` - List arguments including json_mode and cache settings
/// * `config` - Agent configuration
///
/// # Examples
///
/// ```no_run
/// use sikil::commands::list::{execute_list, ListArgs};
/// use sikil::core::config::Config;
///
/// let args = ListArgs {
///     json_mode: false,
///     no_cache: false,
///     agent_filter: None,
///     managed_only: false,
///     unmanaged_only: false,
///     conflicts_only: false,
///     duplicates_only: false,
///     verbose: false,
/// };
/// let config = Config::default();
/// execute_list(args, &config).unwrap();
/// ```
pub fn execute_list(args: ListArgs, config: &Config) -> Result<()> {
    let output = Output::new(args.json_mode);

    // Create scanner (with or without cache based on args)
    let scanner = if args.no_cache {
        Scanner::without_cache(config.clone())
    } else {
        Scanner::new(config.clone())
    };

    // Scan all agents
    let scan_result = scanner.scan_all_agents();

    // Detect conflicts
    let all_conflicts = conflicts::detect_conflicts(&scan_result);

    // Get all skills
    let skills = scan_result.all_skills();

    // Apply filters
    let filtered_skills = apply_filters(&skills, &args, &all_conflicts);

    // Check if any skills were found
    if filtered_skills.is_empty() {
        if args.json_mode {
            output.print_json(&Vec::<ListSkillOutput>::new())?;
        } else {
            output.print_info("No skills found. Install a skill with `sikil install`.");

            // Check if any default agents are disabled
            let disabled_agents = get_disabled_default_agents(config);
            if !disabled_agents.is_empty() {
                output.print_info("");
                output.print_warning("Some default agents are disabled in your config:");
                for (agent_name, path) in &disabled_agents {
                    output.print_info(&format!("  - {} (scans {})", agent_name, path));
                }
                output.print_info("");
                output.print_info(
                    "To enable an agent, run: sikil config set agents.<agent>.enabled true",
                );
            }
        }
        return Ok(());
    }

    // Group skills by managed status
    let mut managed_skills: Vec<&Skill> = Vec::new();
    let mut unmanaged_skills: Vec<&Skill> = Vec::new();

    for skill in &filtered_skills {
        if skill.is_managed {
            managed_skills.push(skill);
        } else {
            unmanaged_skills.push(skill);
        }
    }

    // Sort by name
    managed_skills.sort_by(|a, b| a.metadata.name.cmp(&b.metadata.name));
    unmanaged_skills.sort_by(|a, b| a.metadata.name.cmp(&b.metadata.name));

    // Convert to output format
    let mut output_skills = Vec::new();

    for skill in managed_skills.iter().chain(unmanaged_skills.iter()) {
        let installations: Vec<ListInstallationOutput> = skill
            .installations
            .iter()
            .map(|inst| ListInstallationOutput {
                agent: inst.agent.to_string(),
                scope: format_scope(inst.scope),
            })
            .collect();

        let directory_name = if skill.directory_name != skill.metadata.name {
            Some(skill.directory_name.clone())
        } else {
            None
        };

        output_skills.push(ListSkillOutput {
            name: skill.metadata.name.clone(),
            directory_name,
            description: skill.metadata.description.clone(),
            managed: skill.is_managed,
            installations,
        });
    }

    // Output results
    if args.json_mode {
        output.print_json(&output_skills)?;
    } else {
        print_human_readable(&output, &output_skills, &all_conflicts, args.verbose);
    }

    Ok(())
}

/// Formats the scope enum as a display string
fn format_scope(scope: Scope) -> String {
    match scope {
        Scope::Global => "global".to_string(),
        Scope::Workspace => "workspace".to_string(),
    }
}

/// Applies filters to a list of skills based on the provided arguments
fn apply_filters<'a>(
    skills: &'a [Skill],
    args: &ListArgs,
    all_conflicts: &[crate::core::conflicts::Conflict],
) -> Vec<&'a Skill> {
    let mut filtered: Vec<&Skill> = skills.iter().collect();

    // Apply --agent filter
    if let Some(agent) = args.agent_filter {
        filtered.retain(|skill| skill.installations.iter().any(|inst| inst.agent == agent));
    }

    // Apply --managed filter
    if args.managed_only {
        filtered.retain(|skill| skill.is_managed);
    }

    // Apply --unmanaged filter
    if args.unmanaged_only {
        filtered.retain(|skill| !skill.is_managed);
    }

    // Apply --conflicts filter using the conflicts module
    if args.conflicts_only {
        // Use filter_displayable_conflicts to respect verbose mode
        let displayable_conflicts =
            conflicts::filter_displayable_conflicts(all_conflicts, args.verbose);
        let conflict_skill_names: std::collections::HashSet<_> = displayable_conflicts
            .iter()
            .map(|c| &c.skill_name)
            .collect();

        filtered.retain(|skill| conflict_skill_names.contains(&skill.metadata.name));
    }

    // Apply --duplicates filter (alias for conflicts)
    if args.duplicates_only {
        // Use filter_displayable_conflicts to respect verbose mode
        let displayable_conflicts =
            conflicts::filter_displayable_conflicts(all_conflicts, args.verbose);
        let conflict_skill_names: std::collections::HashSet<_> = displayable_conflicts
            .iter()
            .map(|c| &c.skill_name)
            .collect();

        filtered.retain(|skill| conflict_skill_names.contains(&skill.metadata.name));
    }

    filtered
}

/// Prints human-readable output for the list command
fn print_human_readable(
    output: &Output,
    skills: &[ListSkillOutput],
    all_conflicts: &[crate::core::conflicts::Conflict],
    verbose: bool,
) {
    let managed_count = skills.iter().filter(|s| s.managed).count();
    let unmanaged_count = skills.len() - managed_count;

    // Print header with summary including conflicts
    let conflicts_summary =
        crate::core::conflicts::format_conflicts_summary(all_conflicts, verbose);
    output.print_info(&format!(
        "Found {} skill{} ({} managed, {} unmanaged) - {}",
        skills.len(),
        if skills.len() == 1 { "" } else { "s" },
        managed_count,
        unmanaged_count,
        conflicts_summary
    ));

    if skills.is_empty() {
        return;
    }

    output.print_info("");

    // Calculate column widths
    let max_name_len = skills
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(20)
        .min(30);
    let name_width = max_name_len.max(15);
    let desc_width = 50;

    // Print table header
    let header = format!(
        "{:<width_name$}  {:<width_desc$}  {}",
        "NAME",
        "DESCRIPTION",
        "AGENTS",
        width_name = name_width,
        width_desc = desc_width
    );
    let separator = format!(
        "{:-<width_name$}  {:-<width_desc$}  {:-<20}",
        "",
        "",
        "",
        width_name = name_width,
        width_desc = desc_width
    );

    output.print_info(&header);
    output.print_info(&separator);

    // Print each skill as a table row
    for skill in skills {
        // Format skill name with status indicator
        let status_char = if skill.managed { "✓" } else { "?" };

        let name_with_status = format!("{} {}", status_char, skill.name);

        // Truncate description if needed
        let desc = if skill.description.len() > desc_width {
            format!("{}...", &skill.description[..desc_width.saturating_sub(3)])
        } else {
            skill.description.clone()
        };

        // Format agents list
        let agents_str = if skill.installations.is_empty() {
            "-".to_string()
        } else {
            skill
                .installations
                .iter()
                .map(|inst| format!("{}({})", inst.agent, inst.scope))
                .collect::<Vec<_>>()
                .join(", ")
        };

        // Format the row
        let row = format!(
            "{:<width_name$}  {:<width_desc$}  {}",
            name_with_status,
            desc,
            agents_str,
            width_name = name_width,
            width_desc = desc_width
        );

        // Print with appropriate color for the status
        if skill.managed {
            output.print_success(&row);
        } else {
            output.print_warning(&row);
        }

        // Print directory name note if different
        if let Some(ref dir_name) = skill.directory_name {
            output.print_info(&format!(
                "{:<width_name$}  (directory: {})",
                "",
                dir_name,
                width_name = name_width
            ));
        }
    }

    // Print conflict details and recommendations if any conflicts exist
    // Filter based on verbose mode - don't print DuplicateManaged conflicts unless verbose
    let displayable_conflicts = conflicts::filter_displayable_conflicts(all_conflicts, verbose);
    if !displayable_conflicts.is_empty() {
        output.print_info("");
        for conflict in displayable_conflicts {
            output.print_info(&conflicts::format_conflict(conflict));
            output.print_info("  Recommendations:");
            for (i, rec) in conflict.recommendations().iter().enumerate() {
                output.print_info(&format!("    {}. {}", i + 1, rec));
            }
            output.print_info("");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_scope() {
        assert_eq!(format_scope(Scope::Global), "global");
        assert_eq!(format_scope(Scope::Workspace), "workspace");
    }

    #[test]
    fn test_list_installation_output_serialization() {
        let output = ListInstallationOutput {
            agent: "claude-code".to_string(),
            scope: "global".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"agent\":\"claude-code\""));
        assert!(json.contains("\"scope\":\"global\""));
    }

    #[test]
    fn test_list_skill_output_serialization() {
        let output = ListSkillOutput {
            name: "test-skill".to_string(),
            directory_name: None,
            description: "A test skill".to_string(),
            managed: true,
            installations: vec![ListInstallationOutput {
                agent: "claude-code".to_string(),
                scope: "global".to_string(),
            }],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"name\":\"test-skill\""));
        assert!(json.contains("\"managed\":true"));
        assert!(json.contains("\"agent\":\"claude-code\""));
    }

    #[test]
    fn test_list_skill_output_with_different_directory_name() {
        let output = ListSkillOutput {
            name: "my-skill".to_string(),
            directory_name: Some("my-skill-v2".to_string()),
            description: "A test skill".to_string(),
            managed: false,
            installations: vec![],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"directory_name\":\"my-skill-v2\""));
    }

    #[test]
    fn test_list_skill_output_without_different_directory_name_skips_field() {
        let output = ListSkillOutput {
            name: "my-skill".to_string(),
            directory_name: None,
            description: "A test skill".to_string(),
            managed: false,
            installations: vec![],
        };

        let json = serde_json::to_string(&output).unwrap();
        // directory_name should be skipped when None
        assert!(!json.contains("directory_name"));
    }

    #[test]
    fn test_execute_list_empty_results() {
        let args = ListArgs {
            json_mode: false,
            no_cache: true,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: false,
            verbose: false,
        };

        let config = Config::new(); // Empty config

        // Should not error, just return Ok with message about no skills
        let result = execute_list(args, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_args_new() {
        let args = ListArgs {
            json_mode: true,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: false,
            verbose: false,
        };

        assert!(args.json_mode);
        assert!(!args.no_cache);
        assert!(!args.verbose);
    }

    #[test]
    fn test_list_args_with_verbose() {
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: false,
            verbose: true,
        };

        assert!(args.verbose);
    }

    #[test]
    fn test_apply_filters_agent_filter() {
        use std::path::PathBuf;

        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new("skill1".to_string(), "A skill".to_string()),
            "skill1".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill1"),
            Scope::Global,
        ));

        let skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "skill2".to_string(),
                "Another skill".to_string(),
            ),
            "skill2".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skill2"),
            Scope::Global,
        ));

        let skill3 = Skill::new(
            crate::core::skill::SkillMetadata::new("skill3".to_string(), "Third skill".to_string()),
            "skill3".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill3"),
            Scope::Global,
        ))
        .with_installation(crate::core::skill::Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skill3"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2, skill3];

        // Filter by Claude Code
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: Some(Agent::ClaudeCode),
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: false,
            verbose: false,
        };

        let conflicts = vec![];
        let filtered = apply_filters(&skills, &args, &conflicts);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].metadata.name, "skill1");
        assert_eq!(filtered[1].metadata.name, "skill3");
    }

    #[test]
    fn test_apply_filters_managed_only() {
        use std::path::PathBuf;

        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new("skill1".to_string(), "A skill".to_string()),
            "skill1".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill1"),
            Scope::Global,
        ));

        let mut skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "skill2".to_string(),
                "Another skill".to_string(),
            ),
            "skill2".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill2"),
            Scope::Global,
        ));
        skill2.is_managed = true;

        let skills = vec![skill1, skill2];

        // Filter by managed only
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: true,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: false,
            verbose: false,
        };

        let conflicts = vec![];
        let filtered = apply_filters(&skills, &args, &conflicts);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata.name, "skill2");
        assert!(filtered[0].is_managed);
    }

    #[test]
    fn test_apply_filters_unmanaged_only() {
        use std::path::PathBuf;

        let mut skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new("skill1".to_string(), "A skill".to_string()),
            "skill1".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill1"),
            Scope::Global,
        ));
        skill1.is_managed = true;

        let skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "skill2".to_string(),
                "Another skill".to_string(),
            ),
            "skill2".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill2"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2];

        // Filter by unmanaged only
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: true,
            conflicts_only: false,
            duplicates_only: false,
            verbose: false,
        };

        let conflicts = vec![];
        let filtered = apply_filters(&skills, &args, &conflicts);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata.name, "skill2");
        assert!(!filtered[0].is_managed);
    }

    #[test]
    fn test_apply_filters_duplicates() {
        use crate::core::conflicts::{Conflict, ConflictLocation, ConflictType};
        use std::path::PathBuf;

        // Skill with duplicate paths (same name, different physical locations)
        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "duplicate-skill".to_string(),
                "A duplicate".to_string(),
            ),
            "duplicate-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skills/duplicate-skill"),
            Scope::Global,
        ))
        .with_installation(crate::core::skill::Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skills/duplicate-skill"),
            Scope::Global,
        ));

        // Skill with single installation
        let skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "unique-skill".to_string(),
                "A unique skill".to_string(),
            ),
            "unique-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skills/unique-skill"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2];

        // Create a conflict for duplicate-skill (it has duplicates)
        let conflicts = vec![Conflict::new(
            "duplicate-skill".to_string(),
            vec![
                ConflictLocation::new(
                    "claude-code".to_string(),
                    PathBuf::from("/claude/skills/duplicate-skill"),
                    false,
                    None,
                ),
                ConflictLocation::new(
                    "windsurf".to_string(),
                    PathBuf::from("/windsurf/skills/duplicate-skill"),
                    false,
                    None,
                ),
            ],
            ConflictType::DuplicateUnmanaged,
        )];

        // Filter by duplicates only
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: true,
            verbose: false,
        };

        let filtered = apply_filters(&skills, &args, &conflicts);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata.name, "duplicate-skill");
    }

    #[test]
    fn test_apply_filters_no_filters() {
        use std::path::PathBuf;

        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new("skill1".to_string(), "A skill".to_string()),
            "skill1".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill1"),
            Scope::Global,
        ));

        let skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "skill2".to_string(),
                "Another skill".to_string(),
            ),
            "skill2".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skill2"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2];

        // No filters applied
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: false,
            verbose: false,
        };

        let conflicts = vec![];
        let filtered = apply_filters(&skills, &args, &conflicts);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_apply_filters_agent_and_managed() {
        use std::path::PathBuf;

        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new("skill1".to_string(), "A skill".to_string()),
            "skill1".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill1"),
            Scope::Global,
        ));

        let mut skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "skill2".to_string(),
                "Another skill".to_string(),
            ),
            "skill2".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skill2"),
            Scope::Global,
        ));
        skill2.is_managed = true;

        let skill3 = Skill::new(
            crate::core::skill::SkillMetadata::new("skill3".to_string(), "Third skill".to_string()),
            "skill3".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skill3"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2, skill3];

        // Filter by Claude Code AND managed only
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: Some(Agent::ClaudeCode),
            managed_only: true,
            unmanaged_only: false,
            conflicts_only: false,
            duplicates_only: false,
            verbose: false,
        };

        let conflicts = vec![];
        let filtered = apply_filters(&skills, &args, &conflicts);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata.name, "skill2");
        assert!(filtered[0].is_managed);
    }

    #[test]
    fn test_apply_filters_with_conflicts() {
        use crate::core::conflicts::{Conflict, ConflictLocation, ConflictType};
        use std::path::PathBuf;

        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "conflict-skill".to_string(),
                "A conflict".to_string(),
            ),
            "conflict-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/conflict-skill"),
            Scope::Global,
        ));

        let skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "normal-skill".to_string(),
                "Normal".to_string(),
            ),
            "normal-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/normal-skill"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2];

        // Create a conflict for conflict-skill
        let conflicts = vec![Conflict::new(
            "conflict-skill".to_string(),
            vec![ConflictLocation::new(
                "claude-code".to_string(),
                PathBuf::from("/claude/conflict-skill"),
                false,
                None,
            )],
            ConflictType::DuplicateUnmanaged,
        )];

        // Filter by conflicts only
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: true,
            duplicates_only: false,
            verbose: false,
        };

        let filtered = apply_filters(&skills, &args, &conflicts);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata.name, "conflict-skill");
    }

    #[test]
    fn test_get_disabled_default_agents_all_enabled() {
        let config = Config::default(); // All agents enabled
        let disabled = get_disabled_default_agents(&config);
        assert_eq!(disabled.len(), 0);
    }

    #[test]
    fn test_get_disabled_default_agents_one_disabled() {
        let mut config = Config::default();
        // Disable claude-code
        if let Some(agent) = config.agents.get_mut("claude-code") {
            agent.enabled = false;
        }

        let disabled = get_disabled_default_agents(&config);
        assert_eq!(disabled.len(), 1);
        assert_eq!(disabled[0].0, "claude-code");
        assert!(disabled[0].1.contains(".claude"));
    }

    #[test]
    fn test_get_disabled_default_agents_multiple_disabled() {
        let mut config = Config::default();
        // Disable claude-code and windsurf
        if let Some(agent) = config.agents.get_mut("claude-code") {
            agent.enabled = false;
        }
        if let Some(agent) = config.agents.get_mut("windsurf") {
            agent.enabled = false;
        }

        let disabled = get_disabled_default_agents(&config);
        assert_eq!(disabled.len(), 2);
        // Check they're sorted alphabetically
        assert_eq!(disabled[0].0, "claude-code");
        assert_eq!(disabled[1].0, "windsurf");
    }

    #[test]
    fn test_get_disabled_default_agents_custom_agent_ignored() {
        let mut config = Config::default();
        // Add a custom agent that's disabled (should not be in result)
        config.insert_agent(
            "custom-agent".to_string(),
            crate::core::config::AgentConfig::new(
                false,
                std::path::PathBuf::from("~/custom/skills"),
                std::path::PathBuf::from(".custom/skills"),
            ),
        );

        let disabled = get_disabled_default_agents(&config);
        // Custom agent should not be in the disabled list since it's not a default agent
        assert_eq!(disabled.len(), 0);
    }

    #[test]
    fn test_apply_filters_conflicts_only_verbose_false_filters_info_conflicts() {
        use crate::core::conflicts::{Conflict, ConflictLocation, ConflictType};
        use std::path::PathBuf;

        // Skill with DuplicateManaged conflict (info-level, not error)
        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "managed-skill".to_string(),
                "A managed skill".to_string(),
            ),
            "managed-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skills/managed-skill"),
            Scope::Global,
        ))
        .with_installation(crate::core::skill::Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skills/managed-skill"),
            Scope::Global,
        ));

        // Skill with DuplicateUnmanaged conflict (error-level)
        let skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "unmanaged-skill".to_string(),
                "An unmanaged skill".to_string(),
            ),
            "unmanaged-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skills/unmanaged-skill"),
            Scope::Global,
        ))
        .with_installation(crate::core::skill::Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skills/unmanaged-skill"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2];

        // Create conflicts: one managed (info) and one unmanaged (error)
        let conflicts = vec![
            Conflict::new(
                "managed-skill".to_string(),
                vec![ConflictLocation::new(
                    "claude-code".to_string(),
                    PathBuf::from("/claude/skills/managed-skill"),
                    true,
                    None,
                )],
                ConflictType::DuplicateManaged,
            ),
            Conflict::new(
                "unmanaged-skill".to_string(),
                vec![ConflictLocation::new(
                    "claude-code".to_string(),
                    PathBuf::from("/claude/skills/unmanaged-skill"),
                    false,
                    None,
                )],
                ConflictType::DuplicateUnmanaged,
            ),
        ];

        // Filter by conflicts only with verbose=false should only show error conflicts
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: true,
            duplicates_only: false,
            verbose: false,
        };

        let filtered = apply_filters(&skills, &args, &conflicts);
        // Should only include the error-level conflict skill
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].metadata.name, "unmanaged-skill");
    }

    #[test]
    fn test_apply_filters_conflicts_only_verbose_true_includes_info_conflicts() {
        use crate::core::conflicts::{Conflict, ConflictLocation, ConflictType};
        use std::path::PathBuf;

        // Skill with DuplicateManaged conflict (info-level)
        let skill1 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "managed-skill".to_string(),
                "A managed skill".to_string(),
            ),
            "managed-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skills/managed-skill"),
            Scope::Global,
        ));

        // Skill with DuplicateUnmanaged conflict (error-level)
        let skill2 = Skill::new(
            crate::core::skill::SkillMetadata::new(
                "unmanaged-skill".to_string(),
                "An unmanaged skill".to_string(),
            ),
            "unmanaged-skill".to_string(),
        )
        .with_installation(crate::core::skill::Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skills/unmanaged-skill"),
            Scope::Global,
        ));

        let skills = vec![skill1, skill2];

        // Create conflicts: one managed (info) and one unmanaged (error)
        let conflicts = vec![
            Conflict::new(
                "managed-skill".to_string(),
                vec![ConflictLocation::new(
                    "claude-code".to_string(),
                    PathBuf::from("/claude/skills/managed-skill"),
                    true,
                    None,
                )],
                ConflictType::DuplicateManaged,
            ),
            Conflict::new(
                "unmanaged-skill".to_string(),
                vec![ConflictLocation::new(
                    "claude-code".to_string(),
                    PathBuf::from("/claude/skills/unmanaged-skill"),
                    false,
                    None,
                )],
                ConflictType::DuplicateUnmanaged,
            ),
        ];

        // Filter by conflicts only with verbose=true should show both error and info conflicts
        let args = ListArgs {
            json_mode: false,
            no_cache: false,
            agent_filter: None,
            managed_only: false,
            unmanaged_only: false,
            conflicts_only: true,
            duplicates_only: false,
            verbose: true,
        };

        let filtered = apply_filters(&skills, &args, &conflicts);
        // Should include both skills
        assert_eq!(filtered.len(), 2);
        let skill_names: Vec<_> = filtered.iter().map(|s| &s.metadata.name).collect();
        assert!(skill_names.contains(&&"managed-skill".to_string()));
        assert!(skill_names.contains(&&"unmanaged-skill".to_string()));
    }
}
