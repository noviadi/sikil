//! List command implementation
//!
//! This module provides functionality for listing installed Agent Skills
//! across all configured agents.

use crate::cli::output::Output;
use crate::core::config::Config;
use crate::core::scanner::Scanner;
use crate::core::skill::{Scope, Skill};
use anyhow::Result;

/// Arguments for the list command
#[derive(Debug, Clone)]
pub struct ListArgs {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Whether to disable cache
    pub no_cache: bool,
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

    // Get all skills
    let skills = scan_result.all_skills();

    // Check if any skills were found
    if skills.is_empty() {
        if args.json_mode {
            output.print_json(&Vec::<ListSkillOutput>::new())?;
        } else {
            output.print_info("No skills found. Install a skill with `sikil install`.");
        }
        return Ok(());
    }

    // Group skills by managed status
    let mut managed_skills: Vec<&Skill> = Vec::new();
    let mut unmanaged_skills: Vec<&Skill> = Vec::new();

    for skill in &skills {
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
        print_human_readable(&output, &output_skills);
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

/// Prints human-readable output for the list command
fn print_human_readable(output: &Output, skills: &[ListSkillOutput]) {
    let managed_count = skills.iter().filter(|s| s.managed).count();
    let unmanaged_count = skills.len() - managed_count;

    // Print header
    output.print_info(&format!(
        "Found {} skill{} ({} managed, {} unmanaged)",
        skills.len(),
        if skills.len() == 1 { "" } else { "s" },
        managed_count,
        unmanaged_count
    ));
    output.print_info("");

    for skill in skills {
        // Print status indicator
        if skill.managed {
            output.print_success(&format!("{} ✓", skill.name));
        } else {
            output.print_warning(&format!("{} ?", skill.name));
        };

        // Print description (truncated if too long)
        let desc = if skill.description.len() > 60 {
            format!("{}...", &skill.description[..57])
        } else {
            skill.description.clone()
        };
        output.print_info(&format!("  {}", desc));

        // Print directory name if different
        if let Some(ref dir_name) = skill.directory_name {
            output.print_info(&format!("  dir: {}", dir_name));
        }

        // Print installations
        if !skill.installations.is_empty() {
            let mut agent_str = String::new();
            for (i, inst) in skill.installations.iter().enumerate() {
                if i > 0 {
                    agent_str.push_str(", ");
                }
                agent_str.push_str(&format!("{} ({})", inst.agent, inst.scope));
            }
            output.print_info(&format!("  agents: {}", agent_str));
        }

        output.print_info("");
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
        };

        assert!(args.json_mode);
        assert!(!args.no_cache);
    }
}
