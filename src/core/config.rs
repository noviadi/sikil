use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Agent-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Default for Config {
    fn default() -> Self {
        let mut agents = HashMap::new();

        // Claude Code
        agents.insert(
            "claude-code".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.cache/claude-code/skills"),
                workspace_path: PathBuf::from(".claude/skills"),
            },
        );

        // Windsurf
        agents.insert(
            "windsurf".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.cache/windsurf/skills"),
                workspace_path: PathBuf::from(".windsurf/skills"),
            },
        );

        // OpenCode
        agents.insert(
            "opencode".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.cache/opencode/skills"),
                workspace_path: PathBuf::from(".opencode/skills"),
            },
        );

        // KiloCode
        agents.insert(
            "kilocode".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.cache/kilocode/skills"),
                workspace_path: PathBuf::from(".kilocode/skills"),
            },
        );

        // Amp
        agents.insert(
            "amp".to_string(),
            AgentConfig {
                enabled: true,
                global_path: PathBuf::from("~/.cache/amp/skills"),
                workspace_path: PathBuf::from(".amp/skills"),
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

        for (_, agent_config) in &config.agents {
            assert!(
                agent_config.enabled,
                "All agents should be enabled by default"
            );
        }
    }

    #[test]
    fn test_config_default_agent_paths() {
        let config = Config::default();

        let claude_code = config.get_agent("claude-code").unwrap();
        assert_eq!(
            claude_code.global_path,
            PathBuf::from("~/.cache/claude-code/skills")
        );
        assert_eq!(claude_code.workspace_path, PathBuf::from(".claude/skills"));

        let amp = config.get_agent("amp").unwrap();
        assert_eq!(amp.global_path, PathBuf::from("~/.cache/amp/skills"));
        assert_eq!(amp.workspace_path, PathBuf::from(".amp/skills"));
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
}
