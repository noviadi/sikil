//! Configuration command implementation
//!
//! This module implements the `sikil config` command for displaying and managing
//! the sikil configuration.

use crate::cli::output::Output;
use crate::core::config::{AgentConfig, Config};
use crate::utils::paths::get_config_path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Arguments for the config command
#[derive(Debug, Clone)]
pub struct ConfigArgs {
    /// Whether to edit the config file
    pub edit: bool,

    /// Whether to set a config value
    pub set: bool,

    /// Key for set operation (if provided)
    pub set_key: Option<String>,

    /// Value for set operation (if provided)
    pub set_value: Option<String>,

    /// JSON output mode
    pub json_mode: bool,
}

/// Display configuration showing which values are defaults vs overridden
#[derive(Debug, Serialize, Deserialize)]
struct ConfigDisplay {
    /// Path to the config file
    config_file: String,
    /// Whether config file exists
    file_exists: bool,
    /// Agent configurations
    agents: HashMap<String, AgentDisplay>,
}

/// Agent configuration with default/override information
#[derive(Debug, Serialize, Deserialize)]
struct AgentDisplay {
    /// Whether this agent is enabled
    enabled: bool,
    /// Global installation path
    global_path: String,
    /// Workspace installation path
    workspace_path: String,
    /// Whether this config uses default values
    is_default: bool,
}

/// Execute the config command
pub fn execute_config(args: ConfigArgs) -> Result<()> {
    let output = Output::new(args.json_mode);
    let config_path = get_config_path();

    if args.edit {
        return execute_config_edit(&output, &config_path);
    }

    if args.set {
        return execute_config_set(&output, &config_path, &args.set_key, &args.set_value);
    }

    // Default: show current config
    execute_config_show(&output, &config_path)
}

/// Show the current configuration
fn execute_config_show(output: &Output, config_path: &Path) -> Result<()> {
    // Load current config (with defaults if file doesn't exist)
    let current_config = Config::load(config_path)?;
    let file_exists = config_path.exists();

    // Get default config for comparison
    let default_config = Config::default();

    // Build display structure
    let mut agents = HashMap::new();

    for (agent_name, agent_config) in &current_config.agents {
        // Check if this agent config matches the defaults
        let default_agent = default_config.get_agent(agent_name);
        let is_default = match default_agent {
            Some(default) => {
                agent_config.enabled == default.enabled
                    && agent_config.global_path == default.global_path
                    && agent_config.workspace_path == default.workspace_path
            }
            None => false, // Not in defaults, so not default
        };

        agents.insert(
            agent_name.clone(),
            AgentDisplay {
                enabled: agent_config.enabled,
                global_path: agent_config.global_path.to_string_lossy().to_string(),
                workspace_path: agent_config.workspace_path.to_string_lossy().to_string(),
                is_default,
            },
        );
    }

    let display = ConfigDisplay {
        config_file: config_path.to_string_lossy().to_string(),
        file_exists,
        agents,
    };

    if output.json_mode {
        output.print_json(&display)?;
    } else {
        print_config_human_readable(&display, file_exists);
    }

    Ok(())
}

/// Print configuration in human-readable format
fn print_config_human_readable(config: &ConfigDisplay, file_exists: bool) {
    println!("Configuration File: {}", config.config_file);

    if file_exists {
        println!("Status: Loaded from file");
    } else {
        println!("Status: Using defaults (file does not exist)");
    }

    println!();
    println!("Agents:");

    if config.agents.is_empty() {
        println!("  No agents configured");
        return;
    }

    // Sort agents by name for consistent output
    let mut sorted_agents: Vec<_> = config.agents.iter().collect();
    sorted_agents.sort_by_key(|(name, _)| *name);

    for (agent_name, agent_display) in sorted_agents {
        println!("  {}:", agent_name);

        let status_indicator = if agent_display.is_default {
            " (default)"
        } else {
            " (custom)"
        };

        println!(
            "    Enabled: {}{}",
            if agent_display.enabled {
                "true"
            } else {
                "false"
            },
            status_indicator
        );
        println!("    Global Path: {}", agent_display.global_path);
        println!("    Workspace Path: {}", agent_display.workspace_path);
        println!();
    }
}

/// Edit the configuration file
fn execute_config_edit(output: &Output, config_path: &Path) -> Result<()> {
    use std::process::Command;

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        crate::utils::paths::ensure_dir_exists(parent)?;
    }

    // Create default config file if it doesn't exist
    if !config_path.exists() {
        let default_config = Config::default();
        let toml_content = toml::to_string_pretty(&default_config)?;
        std::fs::write(config_path, toml_content)?;
        output.print_success(&format!(
            "Created default config file: {}",
            config_path.display()
        ));
    }

    // Get editor from environment or default to vi
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    // Launch editor
    let mut child = Command::new(&editor)
        .arg(config_path)
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to launch editor '{}': {}", editor, e))?;

    let status = child.wait()?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status: {:?}", status);
    }

    // Validate the config after editing
    match Config::load(config_path) {
        Ok(_) => {
            output.print_success("Configuration is valid");
        }
        Err(e) => {
            output.print_error(&format!("Configuration is invalid: {}", e));
            anyhow::bail!("Invalid configuration after editing");
        }
    }

    Ok(())
}

/// Set a configuration value
fn execute_config_set(
    output: &Output,
    config_path: &Path,
    key: &Option<String>,
    value: &Option<String>,
) -> Result<()> {
    if key.is_none() || value.is_none() {
        anyhow::bail!("Both key and value must be provided for set operation");
    }

    let key = key.as_ref().unwrap();
    let value = value.as_ref().unwrap();

    // Parse dotted key (e.g., "agents.claude-code.enabled")
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Key must be in format 'agents.<agent>.<field>'");
    }

    if parts[0] != "agents" {
        anyhow::bail!("Only agent configurations can be set (key must start with 'agents')");
    }

    let agent_name = parts[1];
    let field = parts[2];

    // Validate field name
    if !matches!(field, "enabled" | "global_path" | "workspace_path") {
        anyhow::bail!("Field must be one of: enabled, global_path, workspace_path");
    }

    // Load current config
    let mut config = if config_path.exists() {
        Config::load(config_path)?
    } else {
        Config::default()
    };

    // Get or create agent config
    let agent_config = config
        .agents
        .entry(agent_name.to_string())
        .or_insert_with(|| AgentConfig {
            enabled: true,
            global_path: std::path::PathBuf::from("~/skills"),
            workspace_path: std::path::PathBuf::from("./skills"),
        });

    // Update the field
    match field {
        "enabled" => {
            let enabled_value = value
                .parse::<bool>()
                .map_err(|_| anyhow::anyhow!("'enabled' field must be true or false"))?;
            agent_config.enabled = enabled_value;
        }
        "global_path" => {
            agent_config.global_path = std::path::PathBuf::from(value);
        }
        "workspace_path" => {
            agent_config.workspace_path = std::path::PathBuf::from(value);
        }
        _ => unreachable!(), // Validated above
    }

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        crate::utils::paths::ensure_dir_exists(parent)?;
    }

    // Write updated config
    let toml_content = toml::to_string_pretty(&config)?;
    std::fs::write(config_path, toml_content)?;

    output.print_success(&format!("Set {} = {}", key, value));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_display_structure() {
        let mut agents = HashMap::new();
        agents.insert(
            "test-agent".to_string(),
            AgentDisplay {
                enabled: true,
                global_path: "/test/global".to_string(),
                workspace_path: "./test/workspace".to_string(),
                is_default: false,
            },
        );

        let display = ConfigDisplay {
            config_file: "/test/config.toml".to_string(),
            file_exists: true,
            agents,
        };

        // Verify structure can be serialized
        let json = serde_json::to_string(&display);
        assert!(json.is_ok());
    }

    #[test]
    fn test_execute_config_show_missing_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        // Delete the file to test missing file scenario
        fs::remove_file(temp_path).unwrap();

        let args = ConfigArgs {
            edit: false,
            set: false,
            set_key: None,
            set_value: None,
            json_mode: false,
        };

        // Should not panic and should show defaults
        let result = execute_config_show(&Output::new(false), temp_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_display_default_detection() {
        let mut agents = HashMap::new();
        agents.insert(
            "claude-code".to_string(),
            AgentDisplay {
                enabled: true,
                global_path: "~/.claude/skills".to_string(),
                workspace_path: ".claude/skills".to_string(),
                is_default: true,
            },
        );

        let display = ConfigDisplay {
            config_file: "/test/config.toml".to_string(),
            file_exists: false,
            agents,
        };

        assert_eq!(display.agents["claude-code"].is_default, true);
    }
}
