//! Core skill model types
//!
//! This module defines the fundamental data structures for representing
//! Agent Skills, including metadata and installation information.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Metadata parsed from a SKILL.md file's YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Primary identifier (required)
    pub name: String,

    /// Human-readable description (required)
    pub description: String,

    /// Optional version string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Optional author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Optional license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl SkillMetadata {
    /// Creates a new SkillMetadata with the required fields.
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            version: None,
            author: None,
            license: None,
        }
    }

    /// Sets the version.
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the author.
    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Sets the license.
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }
}

/// Represents a skill discovered on the filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Metadata from SKILL.md
    pub metadata: SkillMetadata,

    /// Directory name (may differ from metadata.name)
    pub directory_name: String,

    /// All installations across agents
    pub installations: Vec<Installation>,

    /// Whether this skill is managed (exists in ~/.sikil/repo/)
    pub is_managed: bool,

    /// Path in managed repo (if managed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<PathBuf>,
}

impl Skill {
    /// Creates a new Skill with the given metadata and directory name.
    pub fn new(metadata: SkillMetadata, directory_name: String) -> Self {
        Self {
            metadata,
            directory_name,
            installations: Vec::new(),
            is_managed: false,
            repo_path: None,
        }
    }

    /// Adds an installation to this skill.
    pub fn with_installation(mut self, installation: Installation) -> Self {
        self.installations.push(installation);
        self
    }

    /// Sets the managed flag and repo path.
    pub fn with_repo(mut self, repo_path: PathBuf) -> Self {
        self.is_managed = true;
        self.repo_path = Some(repo_path);
        self
    }

    /// Returns true if this skill has no installations.
    pub fn is_orphan(&self) -> bool {
        self.installations.is_empty()
    }
}

/// Represents a skill installation at an agent location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    /// Which agent this installation belongs to
    pub agent: Agent,

    /// Absolute path to the installation
    pub path: PathBuf,

    /// Global or workspace scope
    pub scope: Scope,

    /// Whether this installation is a symlink
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symlink: Option<bool>,

    /// If is_symlink, the target path of the symlink
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<PathBuf>,
}

impl Installation {
    /// Creates a new Installation.
    pub fn new(agent: Agent, path: PathBuf, scope: Scope) -> Self {
        Self {
            agent,
            path,
            scope,
            is_symlink: None,
            symlink_target: None,
        }
    }

    /// Sets whether this installation is a symlink.
    pub fn with_is_symlink(mut self, is_symlink: bool) -> Self {
        self.is_symlink = Some(is_symlink);
        self
    }

    /// Sets the symlink target path.
    pub fn with_symlink_target(mut self, target: PathBuf) -> Self {
        self.symlink_target = Some(target);
        self
    }
}

/// Supported AI coding agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    /// Claude Code (Anthropic)
    ClaudeCode,
    /// Windsurf (Codeium)
    Windsurf,
    /// OpenCode
    OpenCode,
    /// KiloCode
    KiloCode,
    /// Amp
    Amp,
}

impl Agent {
    /// Returns all supported agents.
    pub fn all() -> &'static [Agent] {
        &[
            Agent::ClaudeCode,
            Agent::Windsurf,
            Agent::OpenCode,
            Agent::KiloCode,
            Agent::Amp,
        ]
    }

    /// Returns the CLI-friendly name for this agent.
    pub fn cli_name(&self) -> &'static str {
        match self {
            Agent::ClaudeCode => "claude-code",
            Agent::Windsurf => "windsurf",
            Agent::OpenCode => "opencode",
            Agent::KiloCode => "kilo-code",
            Agent::Amp => "amp",
        }
    }

    /// Parses an agent from its CLI name.
    pub fn from_cli_name(s: &str) -> Option<Self> {
        match s {
            "claude-code" => Some(Agent::ClaudeCode),
            "windsurf" => Some(Agent::Windsurf),
            "opencode" => Some(Agent::OpenCode),
            "kilo-code" => Some(Agent::KiloCode),
            "amp" => Some(Agent::Amp),
            _ => None,
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cli_name())
    }
}

/// Installation scope: global or workspace-local
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// Global installation (e.g., ~/.claude/skills/)
    Global,
    /// Workspace installation (e.g., .claude/skills/)
    Workspace,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_metadata_new() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        assert_eq!(metadata.name, "test-skill");
        assert_eq!(metadata.description, "A test skill");
        assert!(metadata.version.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.license.is_none());
    }

    #[test]
    fn test_skill_metadata_builder() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string())
            .with_version("1.0.0".to_string())
            .with_author("Test Author".to_string())
            .with_license("MIT".to_string());

        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.license, Some("MIT".to_string()));
    }

    #[test]
    fn test_skill_new() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let skill = Skill::new(metadata, "test-skill-dir".to_string());

        assert_eq!(skill.metadata.name, "test-skill");
        assert_eq!(skill.directory_name, "test-skill-dir");
        assert!(skill.installations.is_empty());
        assert!(!skill.is_managed);
        assert!(skill.repo_path.is_none());
    }

    #[test]
    fn test_skill_with_installation() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let installation = Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/home/user/.claude/skills/test-skill"),
            Scope::Global,
        );

        let skill = Skill::new(metadata, "test-skill".to_string()).with_installation(installation);

        assert_eq!(skill.installations.len(), 1);
        assert_eq!(skill.installations[0].agent, Agent::ClaudeCode);
    }

    #[test]
    fn test_skill_with_repo() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let repo_path = PathBuf::from("/home/user/.sikil/repo/test-skill");

        let skill = Skill::new(metadata, "test-skill".to_string()).with_repo(repo_path.clone());

        assert!(skill.is_managed);
        assert_eq!(skill.repo_path, Some(repo_path));
    }

    #[test]
    fn test_skill_is_orphan() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let skill = Skill::new(metadata, "test-skill".to_string());
        assert!(skill.is_orphan());

        let installation = Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/home/user/.claude/skills/test-skill"),
            Scope::Global,
        );
        let skill = skill.with_installation(installation);
        assert!(!skill.is_orphan());
    }

    #[test]
    fn test_agent_cli_name() {
        assert_eq!(Agent::ClaudeCode.cli_name(), "claude-code");
        assert_eq!(Agent::Windsurf.cli_name(), "windsurf");
        assert_eq!(Agent::OpenCode.cli_name(), "opencode");
        assert_eq!(Agent::KiloCode.cli_name(), "kilo-code");
        assert_eq!(Agent::Amp.cli_name(), "amp");
    }

    #[test]
    fn test_agent_from_cli_name() {
        assert_eq!(Agent::from_cli_name("claude-code"), Some(Agent::ClaudeCode));
        assert_eq!(Agent::from_cli_name("windsurf"), Some(Agent::Windsurf));
        assert_eq!(Agent::from_cli_name("opencode"), Some(Agent::OpenCode));
        assert_eq!(Agent::from_cli_name("kilo-code"), Some(Agent::KiloCode));
        assert_eq!(Agent::from_cli_name("amp"), Some(Agent::Amp));
        assert_eq!(Agent::from_cli_name("unknown"), None);
    }

    #[test]
    fn test_agent_all() {
        let all = Agent::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&Agent::ClaudeCode));
        assert!(all.contains(&Agent::Windsurf));
        assert!(all.contains(&Agent::OpenCode));
        assert!(all.contains(&Agent::KiloCode));
        assert!(all.contains(&Agent::Amp));
    }

    #[test]
    fn test_installation_new() {
        let installation = Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/home/user/.claude/skills/test-skill"),
            Scope::Global,
        );

        assert_eq!(installation.agent, Agent::ClaudeCode);
        assert_eq!(
            installation.path,
            PathBuf::from("/home/user/.claude/skills/test-skill")
        );
        assert_eq!(installation.scope, Scope::Global);
        assert!(installation.is_symlink.is_none());
        assert!(installation.symlink_target.is_none());
    }

    #[test]
    fn test_installation_with_symlink() {
        let installation = Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/home/user/.claude/skills/test-skill"),
            Scope::Global,
        )
        .with_is_symlink(true)
        .with_symlink_target(PathBuf::from("/home/user/.sikil/repo/test-skill"));

        assert_eq!(installation.is_symlink, Some(true));
        assert_eq!(
            installation.symlink_target,
            Some(PathBuf::from("/home/user/.sikil/repo/test-skill"))
        );
    }

    #[test]
    fn test_scope_equality() {
        assert_eq!(Scope::Global, Scope::Global);
        assert_eq!(Scope::Workspace, Scope::Workspace);
        assert_ne!(Scope::Global, Scope::Workspace);
    }

    #[test]
    fn test_serialization() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string())
            .with_version("1.0.0".to_string());

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"name\":\"test-skill\""));
        assert!(json.contains("\"version\":\"1.0.0\""));

        let deserialized: SkillMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-skill");
        assert_eq!(deserialized.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_agent_display() {
        assert_eq!(Agent::ClaudeCode.to_string(), "claude-code");
        assert_eq!(Agent::Windsurf.to_string(), "windsurf");
        assert_eq!(Agent::OpenCode.to_string(), "opencode");
        assert_eq!(Agent::KiloCode.to_string(), "kilo-code");
        assert_eq!(Agent::Amp.to_string(), "amp");
    }
}
