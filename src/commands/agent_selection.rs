//! Agent selection utilities for install command
//!
//! This module provides functionality for parsing and validating agent selections
//! from command-line arguments, including support for comma-separated lists,
//! "all" keyword, and interactive prompts.

use crate::core::config::Config;
use crate::core::errors::SikilError;
use crate::core::skill::Agent;
use std::io::{self, Write};

/// Parse agent selection from the --to flag
///
/// # Arguments
///
/// * `to_value` - The value from --to flag (comma-separated agents or "all")
/// * `config` - Configuration for getting enabled agents
///
/// # Returns
///
/// A vector of Agents to install to
///
/// # Errors
///
/// Returns an error if:
/// - The agent name is unknown
/// - The list is empty after parsing
pub fn parse_agent_selection(
    to_value: Option<&str>,
    config: &Config,
) -> Result<Vec<Agent>, SikilError> {
    match to_value {
        None => Ok(Vec::new()), // Empty means interactive prompt will be used
        Some(value) => {
            let value = value.trim();

            // Check for "all" keyword
            if value.eq_ignore_ascii_case("all") {
                return get_all_enabled_agents(config);
            }

            // Parse comma-separated list
            let agents: Result<Vec<Agent>, SikilError> = value
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|name| parse_and_validate_agent(name, config))
                .collect();

            let agents = agents?;

            if agents.is_empty() {
                return Err(SikilError::ValidationError {
                    reason: "no valid agents specified".to_string(),
                });
            }

            Ok(agents)
        }
    }
}

/// Get all enabled agents from config
fn get_all_enabled_agents(config: &Config) -> Result<Vec<Agent>, SikilError> {
    let agents: Vec<Agent> = config
        .agents
        .iter()
        .filter(|(_, cfg)| cfg.enabled)
        .filter_map(|(name, _)| Agent::from_cli_name(name))
        .collect();

    if agents.is_empty() {
        return Err(SikilError::ValidationError {
            reason: "no enabled agents in configuration".to_string(),
        });
    }

    Ok(agents)
}

/// Parse and validate a single agent name
fn parse_and_validate_agent(name: &str, config: &Config) -> Result<Agent, SikilError> {
    // First, try to parse as a known agent
    if let Some(agent) = Agent::from_cli_name(name) {
        // Check if agent is enabled in config
        if let Some(agent_config) = config.get_agent(name) {
            if !agent_config.enabled {
                return Err(SikilError::ValidationError {
                    reason: format!("agent '{}' is disabled in configuration", name),
                });
            }
        }
        return Ok(agent);
    }

    // Not a known agent
    Err(SikilError::ValidationError {
        reason: format!(
            "unknown agent '{}'. Valid agents: {}",
            name,
            list_valid_agents(config)
        ),
    })
}

/// List all valid (enabled) agents for error messages
fn list_valid_agents(config: &Config) -> String {
    config
        .agents
        .iter()
        .filter(|(_, cfg)| cfg.enabled)
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Interactive prompt for agent selection
///
/// Displays available enabled agents and prompts the user to select one or more.
///
/// # Arguments
///
/// * `config` - Configuration for getting enabled agents
///
/// # Returns
///
/// A vector of selected Agents
///
/// # Errors
///
/// Returns an error if:
/// - No enabled agents are available
/// - User input cannot be read
/// - User provides invalid input
pub fn prompt_agent_selection(config: &Config) -> Result<Vec<Agent>, SikilError> {
    let enabled_agents = get_all_enabled_agents(config)?;

    println!("\nSelect agents to install to:");
    for (i, agent) in enabled_agents.iter().enumerate() {
        println!("  {}. {}", i + 1, agent.cli_name());
    }
    println!("  a. All enabled agents");
    println!();

    print!("Enter selection (e.g., '1', '1,2', 'a'): ");
    io::stdout()
        .flush()
        .map_err(|_e| SikilError::PermissionDenied {
            operation: "flush stdout".to_string(),
            path: "stdout".into(),
        })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_e| SikilError::PermissionDenied {
            operation: "read stdin".to_string(),
            path: "stdin".into(),
        })?;

    let input = input.trim();

    if input.is_empty() {
        return Err(SikilError::ValidationError {
            reason: "no selection made".to_string(),
        });
    }

    // Handle "all" selection
    if input.eq_ignore_ascii_case("a") {
        return Ok(enabled_agents);
    }

    // Parse numbered selection
    let selections: Result<Vec<usize>, _> = input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>())
        .collect();

    let selections = selections.map_err(|_| SikilError::ValidationError {
        reason: "invalid selection format. Use numbers (e.g., '1', '1,2') or 'a' for all"
            .to_string(),
    })?;

    let mut selected_agents = Vec::new();
    for idx in selections {
        if idx == 0 || idx > enabled_agents.len() {
            return Err(SikilError::ValidationError {
                reason: format!(
                    "invalid selection {}. Must be between 1 and {}",
                    idx,
                    enabled_agents.len()
                ),
            });
        }
        selected_agents.push(enabled_agents[idx - 1]);
    }

    if selected_agents.is_empty() {
        return Err(SikilError::ValidationError {
            reason: "no valid selections made".to_string(),
        });
    }

    Ok(selected_agents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Helper to create a test config with all agents enabled
    fn create_test_config() -> Config {
        let mut config = Config::new();

        for agent_name in &["claude-code", "windsurf", "opencode", "kilocode", "amp"] {
            config.insert_agent(
                agent_name.to_string(),
                crate::core::config::AgentConfig::new(
                    true,
                    PathBuf::from(format!("/tmp/{}", agent_name)),
                    PathBuf::from(format!(".{}", agent_name)),
                ),
            );
        }

        config
    }

    /// Helper to create a test config with some agents disabled
    fn create_mixed_config() -> Config {
        let mut config = Config::new();

        // Enable some agents
        for (agent_name, enabled) in [
            ("claude-code", true),
            ("windsurf", true),
            ("opencode", false),
            ("kilocode", false),
            ("amp", true),
        ] {
            config.insert_agent(
                agent_name.to_string(),
                crate::core::config::AgentConfig::new(
                    enabled,
                    PathBuf::from(format!("/tmp/{}", agent_name)),
                    PathBuf::from(format!(".{}", agent_name)),
                ),
            );
        }

        config
    }

    // M3-E01-T03-S01: Parse --to flag: --to claude-code,windsurf
    #[test]
    fn test_parse_agent_selection_comma_separated() {
        let config = create_test_config();
        let result = parse_agent_selection(Some("claude-code,windsurf"), &config);

        assert!(result.is_ok());
        let agents = result.unwrap();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&Agent::ClaudeCode));
        assert!(agents.contains(&Agent::Windsurf));
    }

    #[test]
    fn test_parse_agent_selection_with_spaces() {
        let config = create_test_config();
        let result = parse_agent_selection(Some("claude-code, windsurf , amp"), &config);

        assert!(result.is_ok());
        let agents = result.unwrap();
        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&Agent::ClaudeCode));
        assert!(agents.contains(&Agent::Windsurf));
        assert!(agents.contains(&Agent::Amp));
    }

    #[test]
    fn test_parse_agent_selection_single_agent() {
        let config = create_test_config();
        let result = parse_agent_selection(Some("claude-code"), &config);

        assert!(result.is_ok());
        let agents = result.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0], Agent::ClaudeCode);
    }

    // M3-E01-T03-S02: Parse --to all for all enabled agents
    #[test]
    fn test_parse_agent_selection_all() {
        let config = create_test_config();
        let result = parse_agent_selection(Some("all"), &config);

        assert!(result.is_ok());
        let agents = result.unwrap();
        assert_eq!(agents.len(), 5);
    }

    #[test]
    fn test_parse_agent_selection_all_case_insensitive() {
        let config = create_test_config();
        let result = parse_agent_selection(Some("ALL"), &config);

        assert!(result.is_ok());
        let agents = result.unwrap();
        assert_eq!(agents.len(), 5);
    }

    #[test]
    fn test_parse_agent_selection_all_with_mixed_config() {
        let config = create_mixed_config();
        let result = parse_agent_selection(Some("all"), &config);

        assert!(result.is_ok());
        let agents = result.unwrap();
        // Should only include enabled agents: claude-code, windsurf, amp
        assert_eq!(agents.len(), 3);
        assert!(agents.contains(&Agent::ClaudeCode));
        assert!(agents.contains(&Agent::Windsurf));
        assert!(agents.contains(&Agent::Amp));
        assert!(!agents.contains(&Agent::OpenCode));
        assert!(!agents.contains(&Agent::KiloCode));
    }

    // M3-E01-T03-S04: Validate agent names
    #[test]
    fn test_parse_agent_selection_unknown_agent() {
        let config = create_test_config();
        let result = parse_agent_selection(Some("unknown-agent"), &config);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown agent"));
    }

    #[test]
    fn test_parse_agent_selection_mixed_unknown() {
        let config = create_test_config();
        let result = parse_agent_selection(Some("claude-code,unknown,windsurf"), &config);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown agent"));
    }

    #[test]
    fn test_parse_agent_selection_disabled_agent() {
        let config = create_mixed_config();
        let result = parse_agent_selection(Some("opencode"), &config);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("disabled"));
    }

    #[test]
    fn test_parse_agent_selection_empty_value() {
        let config = create_test_config();
        let result = parse_agent_selection(Some(""), &config);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_agent_selection_none_returns_empty() {
        let config = create_test_config();
        let result = parse_agent_selection(None, &config);

        assert!(result.is_ok());
        let agents = result.unwrap();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_list_valid_agents() {
        let config = create_mixed_config();
        let list = list_valid_agents(&config);

        // Should only list enabled agents
        assert!(list.contains("claude-code"));
        assert!(list.contains("windsurf"));
        assert!(list.contains("amp"));
        assert!(!list.contains("opencode"));
        assert!(!list.contains("kilocode"));
    }

    #[test]
    fn test_get_all_enabled_agents_with_no_enabled() {
        let mut config = Config::new();

        // Add all agents as disabled
        for agent_name in &["claude-code", "windsurf"] {
            config.insert_agent(
                agent_name.to_string(),
                crate::core::config::AgentConfig::new(
                    false,
                    PathBuf::from(format!("/tmp/{}", agent_name)),
                    PathBuf::from(format!(".{}", agent_name)),
                ),
            );
        }

        let result = get_all_enabled_agents(&config);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("no enabled agents"));
    }
}
