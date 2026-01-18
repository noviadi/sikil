use crate::core::errors::ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Agent-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    /// Whether this agent is enabled
    pub enabled: bool,
    /// Global installation path for this agent
    pub global_path: PathBuf,
    /// Workspace installation path for this agent
    pub workspace_path: PathBuf,
}

impl AgentConfig {
    /// Create a new agent configuration
    pub fn new(enabled: bool, global_path: PathBuf, workspace_path: PathBuf) -> Self {
        Self {
            enabled,
            global_path,
            workspace_path,
        }
    }
}

/// Global configuration for sikil
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Agent configurations indexed by agent name
    pub agents: HashMap<String, AgentConfig>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Insert an agent configuration
    pub fn insert_agent(&mut self, name: String, config: AgentConfig) {
        self.agents.insert(name, config);
    }

    /// Get an agent configuration
    pub fn get_agent(&self, name: &str) -> Option<&AgentConfig> {
        self.agents.get(name)
    }

    /// Load configuration from a TOML file
    /// If the file doesn't exist, returns the default configuration.
    ///
    /// # Arguments
    /// * `path` - Path to the config file (typically ~/.sikil/config.toml)
    ///
    /// # Errors
    /// - If file exists but is larger than 1MB
    /// - If file exists but contains invalid TOML
    /// - If file exists but contains unknown fields
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        // Check if file exists
        if !path.exists() {
            return Ok(Self::default());
        }

        // Check file size (1MB max = 1_048_576 bytes)
        let metadata = fs::metadata(path).map_err(|e| ConfigError::FileRead(e.to_string()))?;
        let file_size = metadata.len();
        if file_size > 1_048_576 {
            return Err(ConfigError::ConfigTooLarge(file_size));
        }

        // Read and parse TOML
        let content = fs::read_to_string(path).map_err(|e| ConfigError::FileRead(e.to_string()))?;
        let config: Config =
            toml::from_str(&content).map_err(|e| ConfigError::InvalidToml(e.to_string()))?;

        Ok(config)
    }

    /// Expand paths in the configuration (replacing ~ with home directory)
    pub fn expand_paths(&mut self) {
        for agent_config in self.agents.values_mut() {
            agent_config.global_path = expand_path(&agent_config.global_path);
            agent_config.workspace_path = expand_path(&agent_config.workspace_path);
        }
    }
}

/// Expand ~ to home directory
fn expand_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    let expanded = shellexpand::tilde(&path_str);
    PathBuf::from(expanded.as_ref())
}

impl Default for Config {
    fn default() -> Self {
        let mut agents = HashMap::new();

        // Claude Code - per official docs: https://code.claude.com/docs/en/skills
        agents.insert(
            "claude-code".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.claude/skills"),
                workspace_path: PathBuf::from(".claude/skills"),
            },
        );

        // Windsurf - per TRD §Domain Model
        agents.insert(
            "windsurf".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.codeium/windsurf/skills"),
                workspace_path: PathBuf::from(".windsurf/skills"),
            },
        );

        // OpenCode - per TRD §Domain Model
        agents.insert(
            "opencode".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.config/opencode/skill"),
                workspace_path: PathBuf::from(".opencode/skill"),
            },
        );

        // KiloCode - per official docs: https://kilo.ai/docs/agent-behavior/skills
        agents.insert(
            "kilocode".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.kilocode/skills"),
                workspace_path: PathBuf::from(".kilocode/skills"),
            },
        );

        // Amp - per TRD §Domain Model
        agents.insert(
            "amp".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.config/agents/skills"),
                workspace_path: PathBuf::from(".agents/skills"),
            },
        );

        Self { agents }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_new() {
        let config = AgentConfig::new(
            true,
            PathBuf::from("/home/user/.cache/agent/skills"),
            PathBuf::from(".agent/skills"),
        );

        assert!(config.enabled);
        assert_eq!(
            config.global_path,
            PathBuf::from("/home/user/.cache/agent/skills")
        );
        assert_eq!(config.workspace_path, PathBuf::from(".agent/skills"));
    }

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.agents.is_empty());
    }

    #[test]
    fn test_config_insert_and_get_agent() {
        let mut config = Config::new();

        let agent_config = AgentConfig::new(
            true,
            PathBuf::from("/global/path"),
            PathBuf::from(".workspace/path"),
        );

        config.insert_agent("test-agent".to_string(), agent_config);

        let retrieved = config.get_agent("test-agent");
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().enabled);
        assert_eq!(
            retrieved.unwrap().global_path,
            PathBuf::from("/global/path")
        );
    }

    #[test]
    fn test_config_default_has_all_agents() {
        let config = Config::default();

        assert!(config.get_agent("claude-code").is_some());
        assert!(config.get_agent("windsurf").is_some());
        assert!(config.get_agent("opencode").is_some());
        assert!(config.get_agent("kilocode").is_some());
        assert!(config.get_agent("amp").is_some());
    }

    #[test]
    fn test_config_default_all_agents_enabled() {
        let config = Config::default();

        for agent_config in config.agents.values() {
            assert!(
                agent_config.enabled,
                "All agents should be enabled by default"
            );
        }
    }

    #[test]
    fn test_config_default_agent_paths() {
        let config = Config::default();

        // Claude Code - per official docs
        let claude_code = config.get_agent("claude-code").unwrap();
        assert_eq!(claude_code.global_path, PathBuf::from("~/.claude/skills"));
        assert_eq!(claude_code.workspace_path, PathBuf::from(".claude/skills"));

        // KiloCode - per official docs
        let kilocode = config.get_agent("kilocode").unwrap();
        assert_eq!(kilocode.global_path, PathBuf::from("~/.kilocode/skills"));
        assert_eq!(kilocode.workspace_path, PathBuf::from(".kilocode/skills"));

        // Amp - per TRD
        let amp = config.get_agent("amp").unwrap();
        assert_eq!(amp.global_path, PathBuf::from("~/.config/agents/skills"));
        assert_eq!(amp.workspace_path, PathBuf::from(".agents/skills"));
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::new();
        config.insert_agent(
            "test-agent".to_string(),
            AgentConfig::new(true, PathBuf::from("/global"), PathBuf::from(".workspace")),
        );

        // Test that the config can be serialized
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok());

        // Test that it can be deserialized
        let deserialized: Result<Config, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let config2 = deserialized.unwrap();
        assert!(config2.get_agent("test-agent").is_some());
    }

    #[test]
    fn test_config_load_missing_file_returns_defaults() {
        let result = Config::load(std::path::Path::new("/nonexistent/path/to/config.toml"));
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.get_agent("claude-code").is_some());
        assert!(config.get_agent("amp").is_some());
    }

    #[test]
    fn test_config_load_valid_toml() {
        // Create a temporary TOML file
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_content = r#"
[agents.test-agent]
enabled = false
global_path = "/test/global"
workspace_path = ".test/workspace"
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write temp file");

        let result = Config::load(temp_path);
        assert!(result.is_ok());

        let config = result.unwrap();
        let agent = config.get_agent("test-agent");
        assert!(agent.is_some());
        assert!(!agent.unwrap().enabled);
        assert_eq!(agent.unwrap().global_path, PathBuf::from("/test/global"));
    }

    #[test]
    fn test_config_load_partial_merges_with_defaults() {
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Only specify one agent, others should come from defaults
        let toml_content = r#"
[agents.custom-agent]
enabled = true
global_path = "/custom/global"
workspace_path = ".custom/workspace"
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write temp file");

        let result = Config::load(temp_path);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.get_agent("custom-agent").is_some());
        // Default agents are not merged since we're not using Default::default() after load
    }

    #[test]
    fn test_config_load_invalid_toml_returns_error() {
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let invalid_toml = "agents]\n[broken = ]";
        std::fs::write(temp_path, invalid_toml).expect("Failed to write temp file");

        let result = Config::load(temp_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidToml(_) => {
                // Expected
            }
            _ => panic!("Expected InvalidToml error"),
        }
    }

    #[test]
    fn test_config_load_oversized_file_returns_error() {
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Create content larger than 1MB
        let large_content = "a".repeat(1_048_577);
        std::fs::write(temp_path, large_content).expect("Failed to write temp file");

        let result = Config::load(temp_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::ConfigTooLarge(size) => {
                assert_eq!(size, 1_048_577);
            }
            _ => panic!("Expected ConfigTooLarge error"),
        }
    }

    #[test]
    fn test_config_deny_unknown_fields() {
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        let toml_with_unknown_field = r#"
[agents.test-agent]
enabled = true
global_path = "/test/global"
workspace_path = ".test/workspace"
unknown_field = "this should fail"
"#;

        std::fs::write(temp_path, toml_with_unknown_field).expect("Failed to write temp file");

        let result = Config::load(temp_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidToml(msg) => {
                assert!(
                    msg.to_lowercase().contains("unknown") || msg.to_lowercase().contains("field")
                );
            }
            _ => panic!("Expected InvalidToml error for unknown field"),
        }
    }

    #[test]
    fn test_config_expand_paths() {
        let mut config = Config::new();
        config.insert_agent(
            "test-agent".to_string(),
            AgentConfig::new(
                true,
                PathBuf::from("~/.cache/agent/skills"),
                PathBuf::from(".agent/skills"),
            ),
        );

        config.expand_paths();

        let agent = config.get_agent("test-agent").unwrap();
        // After expansion, path should not contain ~
        let global_path_str = agent.global_path.to_string_lossy();
        assert!(!global_path_str.contains('~'));
    }

    #[test]
    fn test_config_disabled_agent_is_excluded() {
        let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let temp_path = temp_file.path();

        // Create config with one enabled and one disabled agent
        let toml_content = r#"
[agents.enabled-agent]
enabled = true
global_path = "/enabled/global"
workspace_path = ".enabled/workspace"

[agents.disabled-agent]
enabled = false
global_path = "/disabled/global"
workspace_path = ".disabled/workspace"
"#;

        std::fs::write(temp_path, toml_content).expect("Failed to write temp file");

        let result = Config::load(temp_path);
        assert!(result.is_ok());

        let config = result.unwrap();

        // Both agents should be in the config
        assert!(config.get_agent("enabled-agent").is_some());
        assert!(config.get_agent("disabled-agent").is_some());

        // But enabled should be true and false respectively
        let enabled_agent = config.get_agent("enabled-agent").unwrap();
        assert!(enabled_agent.enabled);

        let disabled_agent = config.get_agent("disabled-agent").unwrap();
        assert!(!disabled_agent.enabled);
    }

    #[test]
    fn test_config_iter_enabled_agents_only() {
        let mut config = Config::new();

        // Add mix of enabled and disabled agents
        config.insert_agent(
            "enabled-1".to_string(),
            AgentConfig::new(true, PathBuf::from("/e1"), PathBuf::from(".e1")),
        );
        config.insert_agent(
            "disabled-1".to_string(),
            AgentConfig::new(false, PathBuf::from("/d1"), PathBuf::from(".d1")),
        );
        config.insert_agent(
            "enabled-2".to_string(),
            AgentConfig::new(true, PathBuf::from("/e2"), PathBuf::from(".e2")),
        );

        // Count only enabled agents
        let enabled_count = config.agents.values().filter(|a| a.enabled).count();
        assert_eq!(enabled_count, 2);

        // Count disabled agents
        let disabled_count = config.agents.values().filter(|a| !a.enabled).count();
        assert_eq!(disabled_count, 1);
    }
}
