//! Directory scanner for discovering Agent Skills
//!
//! This module provides functionality for scanning directories to find
//! Agent Skills, which are identified by the presence of SKILL.md files
//! in subdirectories.

use crate::core::config::Config;
use crate::core::errors::SikilError;
use crate::core::parser::parse_skill_md;
use crate::core::skill::{Agent, Installation, Scope, Skill, SkillMetadata};
use crate::utils::paths::get_repo_path;
use crate::utils::symlink::{read_symlink_target, resolve_realpath};
use fs_err as fs;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// Classification of a skill installation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationType {
    /// Managed skill: symlink pointing to ~/.sikil/repo/
    Managed,
    /// Unmanaged skill: physical directory
    Unmanaged,
    /// Broken symlink: target does not exist
    BrokenSymlink,
    /// Foreign symlink: points to location outside ~/.sikil/repo/
    ForeignSymlink,
}

/// Classifies a skill installation by its type.
///
/// This function determines whether a skill installation is:
/// - **Managed**: A symlink pointing to `~/.sikil/repo/`
/// - **Unmanaged**: A physical directory (not a symlink)
/// - **BrokenSymlink**: A symlink whose target does not exist
/// - **ForeignSymlink**: A symlink pointing outside `~/.sikil/repo/`
///
/// # Arguments
///
/// * `path` - The path to the skill installation to classify
///
/// # Returns
///
/// An `InstallationType` indicating how the skill is installed
///
/// # Examples
///
/// ```no_run
/// use sikil::core::scanner::classify_installation;
/// use std::path::Path;
///
/// let installation_type = classify_installation(Path::new("~/.claude/skills/my-skill"));
/// match installation_type {
///     sikil::core::scanner::InstallationType::Managed => {
///         println!("This skill is managed by Sikil");
///     }
///     sikil::core::scanner::InstallationType::Unmanaged => {
///         println!("This skill is unmanaged");
///     }
///     _ => {}
/// }
/// ```
pub fn classify_installation(path: &Path) -> InstallationType {
    use crate::utils::symlink::is_symlink;

    // Check if it's a symlink
    if is_symlink(path) {
        // Try to resolve the symlink target
        match resolve_realpath(path) {
            Ok(real_target) => {
                // Check if the target is under the repo path
                let repo_path = get_repo_path();
                if real_target.starts_with(&repo_path) {
                    InstallationType::Managed
                } else {
                    InstallationType::ForeignSymlink
                }
            }
            Err(_) => {
                // Failed to resolve - likely a broken symlink
                InstallationType::BrokenSymlink
            }
        }
    } else {
        // Not a symlink - it's a physical directory (unmanaged)
        InstallationType::Unmanaged
    }
}

/// A single skill entry found during scanning
#[derive(Debug, Clone)]
pub struct SkillEntry {
    /// Metadata parsed from SKILL.md
    pub metadata: SkillMetadata,
    /// Directory name (may differ from metadata.name)
    pub directory_name: String,
    /// Full path to the skill directory
    pub path: PathBuf,
    /// Whether this entry is a symlink
    pub is_symlink: bool,
    /// If symlink, the target path
    pub symlink_target: Option<PathBuf>,
    /// The agent this skill belongs to (if known from scan context)
    pub agent: Option<Agent>,
    /// The scope (global or workspace)
    pub scope: Scope,
}

impl SkillEntry {
    /// Creates a new SkillEntry
    pub fn new(
        metadata: SkillMetadata,
        directory_name: String,
        path: PathBuf,
        is_symlink: bool,
        symlink_target: Option<PathBuf>,
        agent: Option<Agent>,
        scope: Scope,
    ) -> Self {
        Self {
            metadata,
            directory_name,
            path,
            is_symlink,
            symlink_target,
            agent,
            scope,
        }
    }

    /// Converts this entry to a full Skill with an installation
    pub fn to_skill(self) -> Skill {
        let installation = Installation::new(
            self.agent.unwrap_or(Agent::ClaudeCode),
            self.path.clone(),
            self.scope,
        )
        .with_is_symlink(self.is_symlink)
        .with_symlink_target(self.symlink_target.unwrap_or_default());

        Skill::new(self.metadata, self.directory_name).with_installation(installation)
    }
}

/// Result of a multi-agent scan operation
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// All discovered skills, keyed by skill name
    pub skills: HashMap<String, Skill>,
    /// Number of entries scanned
    pub entries_found: usize,
    /// Number of entries that failed to parse
    pub parse_errors: Vec<(PathBuf, String)>,
}

impl ScanResult {
    /// Creates a new empty ScanResult
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            entries_found: 0,
            parse_errors: Vec::new(),
        }
    }

    /// Adds a skill entry to this result
    pub fn add_entry(&mut self, entry: SkillEntry) {
        self.entries_found += 1;

        let skill_name = entry.metadata.name.clone();

        // Convert to Skill
        let skill = entry.to_skill();

        // Merge with existing skill if present
        if let Some(existing) = self.skills.get_mut(&skill_name) {
            // Add the installation to the existing skill
            if let Some(installation) = skill.installations.first() {
                existing.installations.push(installation.clone());
            }
            // Update managed status if this entry is a managed symlink
            if skill.is_managed {
                existing.is_managed = true;
                existing.repo_path = skill.repo_path;
            }
        } else {
            self.skills.insert(skill_name, skill);
        }
    }

    /// Records a parse error
    pub fn add_error(&mut self, path: PathBuf, error: String) {
        self.parse_errors.push((path, error));
    }

    /// Returns the number of unique skills found
    pub fn skill_count(&self) -> usize {
        self.skills.len()
    }

    /// Returns all skills as a vector
    pub fn all_skills(&self) -> Vec<Skill> {
        self.skills.values().cloned().collect()
    }
}

impl Default for ScanResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Scanner for discovering Agent Skills in directories
#[derive(Debug, Clone)]
pub struct Scanner {
    /// Configuration for agent paths
    #[allow(dead_code)]
    config: Config,
}

impl Scanner {
    /// Creates a new Scanner with the given configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Scans a single directory for Agent Skills
    ///
    /// This function walks through the directory, looking for subdirectories
    /// that contain SKILL.md files. Each valid skill is added to the result.
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to scan
    /// * `agent` - The agent this directory belongs to
    /// * `scope` - The scope (global or workspace)
    /// * `result` - The ScanResult to populate
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sikil::core::scanner::Scanner;
    /// use sikil::core::config::Config;
    /// use sikil::core::skill::{Agent, Scope};
    ///
    /// let config = Config::default();
    /// let scanner = Scanner::new(config);
    /// let mut result = sikil::core::scanner::ScanResult::new();
    ///
    /// scanner.scan_directory(
    ///     std::path::Path::new("~/.claude/skills"),
    ///     Agent::ClaudeCode,
    ///     Scope::Global,
    ///     &mut result
    /// ).unwrap();
    /// ```
    pub fn scan_directory(
        &self,
        path: &Path,
        agent: Agent,
        scope: Scope,
        result: &mut ScanResult,
    ) -> Result<(), SikilError> {
        // Check if directory exists
        if !path.exists() {
            return Err(SikilError::DirectoryNotFound {
                path: path.to_path_buf(),
            });
        }

        // Check if it's a directory
        if !path.is_dir() {
            return Err(SikilError::DirectoryNotFound {
                path: path.to_path_buf(),
            });
        }

        // Walk through the directory
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => {
                return Err(SikilError::DirectoryNotFound {
                    path: path.to_path_buf(),
                });
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue, // Skip entries we can't read
            };

            let entry_path = entry.path();
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue, // Skip entries we can't get file type for
            };

            // Skip non-directories and non-symlinks
            if !file_type.is_dir() && !file_type.is_symlink() {
                continue;
            }

            // Get directory name
            let directory_name = match entry_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // Skip hidden directories (starting with .)
            if directory_name.starts_with('.') {
                continue;
            }

            // Check if it's a symlink
            let is_symlink = file_type.is_symlink();

            // Get symlink target if it is a symlink
            let symlink_target = if is_symlink {
                read_symlink_target(&entry_path).ok()
            } else {
                None
            };

            // Try to parse SKILL.md
            let skill_md_path = entry_path.join("SKILL.md");
            match self.parse_skill_entry(&skill_md_path, &entry_path, &directory_name) {
                Ok(metadata) => {
                    let skill_entry = SkillEntry::new(
                        metadata,
                        directory_name,
                        entry_path,
                        is_symlink,
                        symlink_target,
                        Some(agent),
                        scope,
                    );
                    result.add_entry(skill_entry);
                }
                Err(e) => {
                    // Record the error but continue scanning
                    result.add_error(skill_md_path, e.to_string());
                }
            }
        }

        Ok(())
    }

    /// Parses a SKILL.md file and extracts metadata
    ///
    /// This is a helper method that handles missing or invalid SKILL.md files
    /// gracefully by returning an error instead of panicking.
    fn parse_skill_entry(
        &self,
        skill_md_path: &Path,
        directory_path: &Path,
        directory_name: &str,
    ) -> Result<SkillMetadata, SikilError> {
        // Check if SKILL.md exists
        if !skill_md_path.exists() {
            return Err(SikilError::InvalidSkillMd {
                path: directory_path.to_path_buf(),
                reason: format!("SKILL.md not found in directory '{}'", directory_name),
            });
        }

        // Parse the SKILL.md file
        parse_skill_md(skill_md_path)
    }

    /// Scans all configured agent directories for skills
    ///
    /// This method performs a comprehensive scan across:
    /// 1. Global paths for all enabled agents
    /// 2. Workspace paths relative to current working directory
    /// 3. The managed skills repository (~/.sikil/repo/)
    ///
    /// Non-existent directories are skipped gracefully.
    ///
    /// # Returns
    ///
    /// A `ScanResult` containing all discovered skills aggregated by name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use sikil::core::scanner::Scanner;
    /// use sikil::core::config::Config;
    ///
    /// let config = Config::default();
    /// let scanner = Scanner::new(config);
    /// let result = scanner.scan_all_agents();
    ///
    /// println!("Found {} skills", result.skill_count());
    /// ```
    pub fn scan_all_agents(&self) -> ScanResult {
        let mut result = ScanResult::new();

        // Scan global paths for all enabled agents
        for (agent_name, agent_config) in &self.config.agents {
            if !agent_config.enabled {
                continue;
            }

            // Parse agent name to Agent enum
            if let Some(agent) = Agent::from_cli_name(agent_name) {
                // Scan global path
                if agent_config.global_path.exists() {
                    if let Err(e) = self.scan_directory(
                        &agent_config.global_path,
                        agent,
                        Scope::Global,
                        &mut result,
                    ) {
                        // Log error but continue scanning other paths
                        result.add_error(agent_config.global_path.clone(), e.to_string());
                    }
                }

                // Scan workspace path (relative to CWD)
                let workspace_path = if agent_config.workspace_path.is_absolute() {
                    agent_config.workspace_path.clone()
                } else {
                    // Relative to current working directory
                    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                    cwd.join(&agent_config.workspace_path)
                };

                if workspace_path.exists() {
                    if let Err(e) =
                        self.scan_directory(&workspace_path, agent, Scope::Workspace, &mut result)
                    {
                        result.add_error(workspace_path, e.to_string());
                    }
                }
            }
        }

        // Scan the managed skills repository
        let repo_path = get_repo_path();
        if repo_path.exists() {
            self.scan_repo(&repo_path, &mut result);
        }

        result
    }

    /// Scans the managed skills repository
    ///
    /// The repo contains skill directories stored under ~/.sikil/repo/.
    /// Each subdirectory in the repo is a potential managed skill.
    ///
    /// Skills found in the repo are marked as managed, and their installations
    /// are discovered by scanning the agent directories that may symlink to them.
    fn scan_repo(&self, repo_path: &Path, result: &mut ScanResult) {
        let entries = match fs::read_dir(repo_path) {
            Ok(entries) => entries,
            Err(_) => return, // Skip if we can't read the repo
        };

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

            // Try to parse SKILL.md
            let skill_md_path = entry_path.join("SKILL.md");
            match self.parse_skill_entry(&skill_md_path, &entry_path, &dir_name) {
                Ok(metadata) => {
                    // Create a skill entry for the repo
                    let skill_entry = SkillEntry::new(
                        metadata,
                        dir_name.clone(),
                        entry_path.clone(),
                        false,
                        None,
                        None, // No specific agent in repo
                        Scope::Global,
                    );

                    // Convert to skill and mark as managed
                    let mut skill = skill_entry.to_skill();
                    skill.is_managed = true;
                    skill.repo_path = Some(entry_path);

                    // Add to result or merge with existing
                    let skill_name = skill.metadata.name.clone();
                    result.entries_found += 1;

                    if let Some(existing) = result.skills.get_mut(&skill_name) {
                        // Update managed status
                        existing.is_managed = true;
                        existing.repo_path = skill.repo_path;
                    } else {
                        result.skills.insert(skill_name, skill);
                    }
                }
                Err(e) => {
                    // Record error but continue scanning
                    result.add_error(skill_md_path, e.to_string());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_new() {
        let config = Config::default();
        let scanner = Scanner::new(config);
        // Scanner is created successfully
        assert_eq!(scanner.config.agents.len(), 5); // 5 default agents
    }

    #[test]
    fn test_scan_result_new() {
        let result = ScanResult::new();
        assert_eq!(result.skill_count(), 0);
        assert_eq!(result.entries_found, 0);
        assert!(result.parse_errors.is_empty());
    }

    #[test]
    fn test_scan_result_default() {
        let result = ScanResult::default();
        assert_eq!(result.skill_count(), 0);
        assert_eq!(result.entries_found, 0);
    }

    #[test]
    fn test_scan_result_add_entry() {
        let mut result = ScanResult::new();
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let entry = SkillEntry::new(
            metadata,
            "test-skill".to_string(),
            PathBuf::from("/test/skills/test-skill"),
            false,
            None,
            Some(Agent::ClaudeCode),
            Scope::Global,
        );

        result.add_entry(entry);
        assert_eq!(result.entries_found, 1);
        assert_eq!(result.skill_count(), 1);
    }

    #[test]
    fn test_scan_result_add_error() {
        let mut result = ScanResult::new();
        result.add_error(PathBuf::from("/test/SKILL.md"), "parse error".to_string());

        assert_eq!(result.parse_errors.len(), 1);
        assert_eq!(result.parse_errors[0].0, PathBuf::from("/test/SKILL.md"));
        assert_eq!(result.parse_errors[0].1, "parse error");
    }

    #[test]
    fn test_scan_result_all_skills() {
        let mut result = ScanResult::new();
        let metadata1 = SkillMetadata::new("skill1".to_string(), "First skill".to_string());
        let metadata2 = SkillMetadata::new("skill2".to_string(), "Second skill".to_string());

        let entry1 = SkillEntry::new(
            metadata1,
            "skill1".to_string(),
            PathBuf::from("/test/skill1"),
            false,
            None,
            Some(Agent::ClaudeCode),
            Scope::Global,
        );

        let entry2 = SkillEntry::new(
            metadata2,
            "skill2".to_string(),
            PathBuf::from("/test/skill2"),
            false,
            None,
            Some(Agent::Windsurf),
            Scope::Global,
        );

        result.add_entry(entry1);
        result.add_entry(entry2);

        let skills = result.all_skills();
        assert_eq!(skills.len(), 2);
    }

    #[test]
    fn test_skill_entry_new() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let entry = SkillEntry::new(
            metadata,
            "test-skill".to_string(),
            PathBuf::from("/test/skills/test-skill"),
            false,
            None,
            Some(Agent::ClaudeCode),
            Scope::Global,
        );

        assert_eq!(entry.metadata.name, "test-skill");
        assert_eq!(entry.directory_name, "test-skill");
        assert_eq!(entry.path, PathBuf::from("/test/skills/test-skill"));
        assert!(!entry.is_symlink);
        assert!(entry.symlink_target.is_none());
        assert_eq!(entry.agent, Some(Agent::ClaudeCode));
        assert_eq!(entry.scope, Scope::Global);
    }

    #[test]
    fn test_skill_entry_with_symlink() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let target = PathBuf::from("/home/user/.sikil/repo/test-skill");
        let entry = SkillEntry::new(
            metadata,
            "test-skill".to_string(),
            PathBuf::from("/test/skills/test-skill"),
            true,
            Some(target.clone()),
            Some(Agent::ClaudeCode),
            Scope::Global,
        );

        assert!(entry.is_symlink);
        assert_eq!(entry.symlink_target, Some(target));
    }

    #[test]
    fn test_skill_entry_to_skill() {
        let metadata = SkillMetadata::new("test-skill".to_string(), "A test skill".to_string());
        let entry = SkillEntry::new(
            metadata,
            "test-skill".to_string(),
            PathBuf::from("/test/skills/test-skill"),
            false,
            None,
            Some(Agent::ClaudeCode),
            Scope::Global,
        );

        let skill = entry.to_skill();
        assert_eq!(skill.metadata.name, "test-skill");
        assert_eq!(skill.directory_name, "test-skill");
        assert_eq!(skill.installations.len(), 1);
        assert_eq!(skill.installations[0].agent, Agent::ClaudeCode);
        assert_eq!(skill.installations[0].scope, Scope::Global);
    }

    #[test]
    fn test_scanner_nonexistent_directory() {
        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        let result_err = scanner.scan_directory(
            Path::new("/nonexistent/directory"),
            Agent::ClaudeCode,
            Scope::Global,
            &mut result,
        );

        assert!(result_err.is_err());
        match result_err.unwrap_err() {
            SikilError::DirectoryNotFound { .. } => {
                // Expected
            }
            _ => panic!("Expected DirectoryNotFound error"),
        }
    }

    #[test]
    fn test_scanner_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        assert_eq!(result.entries_found, 0);
        assert_eq!(result.skill_count(), 0);
    }

    #[test]
    fn test_scanner_with_valid_skill() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("my-skill");
        fs::create_dir(&skill_dir).unwrap();

        let skill_md_content = r#"---
name: my-skill
description: A test skill
---

# My Skill
"#;
        fs::write(skill_dir.join("SKILL.md"), skill_md_content).unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        assert_eq!(result.entries_found, 1);
        assert_eq!(result.skill_count(), 1);
        assert!(result.skills.contains_key("my-skill"));
        assert_eq!(result.skills["my-skill"].metadata.name, "my-skill");
        assert_eq!(
            result.skills["my-skill"].metadata.description,
            "A test skill"
        );
    }

    #[test]
    fn test_scanner_with_multiple_skills() {
        let temp_dir = TempDir::new().unwrap();

        // Create first skill
        let skill1_dir = temp_dir.path().join("skill1");
        fs::create_dir(&skill1_dir).unwrap();
        fs::write(
            skill1_dir.join("SKILL.md"),
            r#"---
name: skill1
description: First skill
---"#,
        )
        .unwrap();

        // Create second skill
        let skill2_dir = temp_dir.path().join("skill2");
        fs::create_dir(&skill2_dir).unwrap();
        fs::write(
            skill2_dir.join("SKILL.md"),
            r#"---
name: skill2
description: Second skill
---"#,
        )
        .unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        assert_eq!(result.entries_found, 2);
        assert_eq!(result.skill_count(), 2);
        assert!(result.skills.contains_key("skill1"));
        assert!(result.skills.contains_key("skill2"));
    }

    #[test]
    fn test_scanner_skips_hidden_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Create hidden directory with SKILL.md
        let hidden_dir = temp_dir.path().join(".hidden-skill");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(
            hidden_dir.join("SKILL.md"),
            r#"---
name: hidden-skill
description: Should be skipped
---"#,
        )
        .unwrap();

        // Create normal directory
        let normal_dir = temp_dir.path().join("normal-skill");
        fs::create_dir(&normal_dir).unwrap();
        fs::write(
            normal_dir.join("SKILL.md"),
            r#"---
name: normal-skill
description: Should be found
---"#,
        )
        .unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        assert_eq!(result.entries_found, 1);
        assert!(result.skills.contains_key("normal-skill"));
        assert!(!result.skills.contains_key("hidden-skill"));
    }

    #[test]
    fn test_scanner_with_invalid_skill_md() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("invalid-skill");
        fs::create_dir(&skill_dir).unwrap();

        // Create invalid SKILL.md (missing required field)
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: invalid-skill
---"#,
        )
        .unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        // Should have an error but not crash
        assert_eq!(result.entries_found, 0);
        assert_eq!(result.parse_errors.len(), 1);
        assert!(result.parse_errors[0].1.contains("missing required field"));
    }

    #[test]
    fn test_scanner_with_missing_skill_md() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("no-md-skill");
        fs::create_dir(&skill_dir).unwrap();
        // Don't create SKILL.md

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        // Should have an error but not crash
        assert_eq!(result.entries_found, 0);
        assert_eq!(result.parse_errors.len(), 1);
        assert!(result.parse_errors[0].1.contains("SKILL.md not found"));
    }

    #[test]
    fn test_scanner_with_symlink_skill() {
        let temp_dir = TempDir::new().unwrap();
        let target_temp = TempDir::new().unwrap();

        // Create the target skill directory outside the scan directory
        let target_dir = target_temp.path().join("target-skill");
        fs::create_dir(&target_dir).unwrap();
        fs::write(
            target_dir.join("SKILL.md"),
            r#"---
name: symlink-skill
description: A symlinked skill
---"#,
        )
        .unwrap();

        // Create a symlink in the scanned directory
        let link_path = temp_dir.path().join("link-to-skill");
        std::os::unix::fs::symlink(&target_dir, &link_path).unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        assert_eq!(result.entries_found, 1);
        assert!(result.skills.contains_key("symlink-skill"));

        let skill = &result.skills["symlink-skill"];
        assert_eq!(skill.installations.len(), 1);
        assert_eq!(skill.installations[0].is_symlink, Some(true));
        assert!(skill.installations[0].symlink_target.is_some());
    }

    #[test]
    fn test_scanner_with_file_skipped() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file (not a directory)
        fs::write(temp_dir.path().join("not-a-dir"), "content").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        assert_eq!(result.entries_found, 0);
    }

    #[test]
    fn test_scan_result_merges_same_skill_different_agents() {
        let mut result = ScanResult::new();

        // Add skill from Claude Code
        let metadata1 = SkillMetadata::new("my-skill".to_string(), "A skill".to_string());
        let entry1 = SkillEntry::new(
            metadata1,
            "my-skill".to_string(),
            PathBuf::from("/claude/skills/my-skill"),
            false,
            None,
            Some(Agent::ClaudeCode),
            Scope::Global,
        );
        result.add_entry(entry1);

        // Add same skill from Windsurf
        let metadata2 = SkillMetadata::new("my-skill".to_string(), "A skill".to_string());
        let entry2 = SkillEntry::new(
            metadata2,
            "my-skill".to_string(),
            PathBuf::from("/windsurf/skills/my-skill"),
            false,
            None,
            Some(Agent::Windsurf),
            Scope::Global,
        );
        result.add_entry(entry2);

        // Should have one skill with two installations
        assert_eq!(result.skill_count(), 1);
        let skill = &result.skills["my-skill"];
        assert_eq!(skill.installations.len(), 2);
        assert_eq!(skill.installations[0].agent, Agent::ClaudeCode);
        assert_eq!(skill.installations[1].agent, Agent::Windsurf);
    }

    #[test]
    fn test_scanner_directory_name_differs_from_metadata_name() {
        let temp_dir = TempDir::new().unwrap();

        // Directory name is "my-skill-v2"
        let skill_dir = temp_dir.path().join("my-skill-v2");
        fs::create_dir(&skill_dir).unwrap();

        // But metadata name is "my-skill"
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: my-skill
description: A test skill
---"#,
        )
        .unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        assert_eq!(result.entries_found, 1);
        let skill = &result.skills["my-skill"];
        assert_eq!(skill.metadata.name, "my-skill");
        assert_eq!(skill.directory_name, "my-skill-v2");
    }

    #[test]
    fn test_scanner_respects_workspace_scope() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("workspace-skill");
        fs::create_dir(&skill_dir).unwrap();

        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: workspace-skill
description: A workspace skill
---"#,
        )
        .unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner
            .scan_directory(
                temp_dir.path(),
                Agent::ClaudeCode,
                Scope::Workspace,
                &mut result,
            )
            .unwrap();

        let skill = &result.skills["workspace-skill"];
        assert_eq!(skill.installations[0].scope, Scope::Workspace);
    }

    #[test]
    fn test_scan_all_agents_empty_config() {
        let config = Config::new(); // Empty config, no agents
        let scanner = Scanner::new(config);
        let result = scanner.scan_all_agents();

        // Should have scanned nothing since no agents are configured
        assert_eq!(result.skill_count(), 0);
        assert_eq!(result.entries_found, 0);
    }

    #[test]
    fn test_scan_all_agents_with_default_config() {
        let config = Config::default();
        let scanner = Scanner::new(config);
        let result = scanner.scan_all_agents();

        // Should complete without error even if no skills exist
        // (directories likely don't exist on test system)
        // skill_count() returns usize which is always >= 0
        assert!(result.skill_count() < 100);
    }

    #[test]
    fn test_scan_all_agents_with_mock_directories() {
        let temp_base = TempDir::new().unwrap();
        let temp_workspace = TempDir::new().unwrap();

        // Create mock global directories for Claude Code
        let claude_global = temp_base
            .path()
            .join("global")
            .join("claude")
            .join("skills");
        fs::create_dir_all(&claude_global).unwrap();

        // Create a skill in Claude Code global
        let skill_dir = claude_global.join("test-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: test-skill
description: A test skill
---"#,
        )
        .unwrap();

        // Create a config that points to our temp directories
        let mut config = Config::new();
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                claude_global.clone(),
                PathBuf::from(".nowhere"), // Use a path that won't exist in temp_workspace
            ),
        );

        // Change to workspace temp directory for workspace scan
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_workspace.path()).unwrap();

        let scanner = Scanner::new(config);
        let result = scanner.scan_all_agents();

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();

        assert_eq!(result.skill_count(), 1);
        assert!(result.skills.contains_key("test-skill"));
    }

    #[test]
    fn test_scan_all_agents_skips_nonexistent_directories() {
        let mut config = Config::new();
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                PathBuf::from("/nonexistent/global/path"),
                PathBuf::from("/nonexistent/workspace/path"),
            ),
        );

        let scanner = Scanner::new(config);
        let result = scanner.scan_all_agents();

        // Should not error, just return empty results
        assert_eq!(result.skill_count(), 0);
        assert_eq!(result.entries_found, 0);
    }

    #[test]
    fn test_scan_all_agents_disabled_agent_skipped() {
        let temp_base = TempDir::new().unwrap();

        // Create directory for enabled agent
        let windsurf_global = temp_base.path().join("windsurf").join("skills");
        fs::create_dir_all(&windsurf_global).unwrap();

        // Create a skill
        let skill_dir = windsurf_global.join("windsurf-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: windsurf-skill
description: A Windsurf skill
---"#,
        )
        .unwrap();

        // Create config with enabled and disabled agents
        let mut config = Config::new();
        config.insert_agent(
            "windsurf".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                windsurf_global,
                PathBuf::from(".windsurf/skills"),
            ),
        );
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                false, // Disabled
                PathBuf::from("/nonexistent/claude"),
                PathBuf::from(".claude/skills"),
            ),
        );

        let scanner = Scanner::new(config);
        let result = scanner.scan_all_agents();

        // Should only find the Windsurf skill
        assert_eq!(result.skill_count(), 1);
        assert!(result.skills.contains_key("windsurf-skill"));
    }

    #[test]
    fn test_scan_all_agents_aggregates_by_skill_name() {
        let temp_base = TempDir::new().unwrap();

        // Create directories for two agents
        let claude_global = temp_base.path().join("claude").join("skills");
        let windsurf_global = temp_base.path().join("windsurf").join("skills");
        fs::create_dir_all(&claude_global).unwrap();
        fs::create_dir_all(&windsurf_global).unwrap();

        // Create the same skill in both agent directories
        for base in [&claude_global, &windsurf_global] {
            let skill_dir = base.join("shared-skill");
            fs::create_dir(&skill_dir).unwrap();
            fs::write(
                skill_dir.join("SKILL.md"),
                r#"---
name: shared-skill
description: A skill shared across agents
---"#,
            )
            .unwrap();
        }

        let mut config = Config::new();
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                claude_global,
                PathBuf::from(".claude/skills"),
            ),
        );
        config.insert_agent(
            "windsurf".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                windsurf_global,
                PathBuf::from(".windsurf/skills"),
            ),
        );

        let scanner = Scanner::new(config);
        let result = scanner.scan_all_agents();

        // Should have one skill with two installations
        assert_eq!(result.skill_count(), 1);
        let skill = &result.skills["shared-skill"];
        assert_eq!(skill.installations.len(), 2);

        // Verify both agents are represented
        let agents: Vec<_> = skill.installations.iter().map(|i| i.agent).collect();
        assert!(agents.contains(&Agent::ClaudeCode));
        assert!(agents.contains(&Agent::Windsurf));
    }

    #[test]
    fn test_scan_repo_with_managed_skills() {
        let temp_repo = TempDir::new().unwrap();

        // Create a managed skill in the repo
        let skill_dir = temp_repo.path().join("managed-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: managed-skill
description: A managed skill
---"#,
        )
        .unwrap();

        let config = Config::new();
        // Override repo path for testing
        // Note: We can't easily override get_repo_path, so we'll test scan_repo directly

        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner.scan_repo(temp_repo.path(), &mut result);

        assert_eq!(result.skill_count(), 1);
        let skill = &result.skills["managed-skill"];
        assert!(skill.is_managed);
        assert_eq!(skill.repo_path, Some(skill_dir.clone()));
    }

    #[test]
    fn test_scan_repo_skips_hidden_directories() {
        let temp_repo = TempDir::new().unwrap();

        // Create hidden directory with skill
        let hidden_dir = temp_repo.path().join(".hidden-skill");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(
            hidden_dir.join("SKILL.md"),
            r#"---
name: hidden-skill
description: Should be skipped
---"#,
        )
        .unwrap();

        // Create normal directory
        let normal_dir = temp_repo.path().join("normal-skill");
        fs::create_dir(&normal_dir).unwrap();
        fs::write(
            normal_dir.join("SKILL.md"),
            r#"---
name: normal-skill
description: Should be found
---"#,
        )
        .unwrap();

        let config = Config::default();
        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        scanner.scan_repo(temp_repo.path(), &mut result);

        assert_eq!(result.skill_count(), 1);
        assert!(result.skills.contains_key("normal-skill"));
        assert!(!result.skills.contains_key("hidden-skill"));
    }

    #[test]
    fn test_scan_repo_merges_with_existing_skill() {
        let temp_repo = TempDir::new().unwrap();
        let temp_agent_dir = TempDir::new().unwrap();

        // Create managed skill in repo
        let repo_skill = temp_repo.path().join("repo-skill");
        fs::create_dir(&repo_skill).unwrap();
        fs::write(
            repo_skill.join("SKILL.md"),
            r#"---
name: my-skill
description: A managed skill
---"#,
        )
        .unwrap();

        // Create symlink in agent directory
        let agent_skill = temp_agent_dir.path().join("my-skill");
        std::os::unix::fs::symlink(&repo_skill, &agent_skill).unwrap();

        let mut config = Config::new();
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                temp_agent_dir.path().to_path_buf(),
                PathBuf::from(".claude/skills"),
            ),
        );

        let scanner = Scanner::new(config);
        let mut result = ScanResult::new();

        // First scan agent directory
        scanner
            .scan_directory(
                temp_agent_dir.path(),
                Agent::ClaudeCode,
                Scope::Global,
                &mut result,
            )
            .unwrap();

        // Then scan repo (should merge)
        scanner.scan_repo(temp_repo.path(), &mut result);

        // Should have one skill that is marked as managed
        assert_eq!(result.skill_count(), 1);
        let skill = &result.skills["my-skill"];
        assert!(skill.is_managed);
        assert_eq!(skill.repo_path, Some(repo_skill));
        assert_eq!(skill.installations.len(), 1);
    }

    #[test]
    fn test_scan_all_agents_with_workspace_path() {
        let temp_base = TempDir::new().unwrap();
        let temp_path = temp_base.into_path(); // Prevent auto-drop while we're in it

        // Create mock workspace directory
        let workspace_dir = temp_path.join(".claude").join("skills");
        fs::create_dir_all(&workspace_dir).unwrap();

        // Create a workspace skill
        let skill_dir = workspace_dir.join("workspace-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: workspace-skill
description: A workspace-local skill
---"#,
        )
        .unwrap();

        // Change to temp directory
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_path).unwrap();

        // Create config with relative workspace path
        let mut config = Config::new();
        config.insert_agent(
            "claude-code".to_string(),
            crate::core::config::AgentConfig::new(
                true,
                PathBuf::from("/nonexistent/global"), // Doesn't exist
                PathBuf::from(".claude/skills"),      // Relative workspace path
            ),
        );

        let scanner = Scanner::new(config);
        let result = scanner.scan_all_agents();

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();

        // Should find the workspace skill
        assert_eq!(result.skill_count(), 1);
        assert!(result.skills.contains_key("workspace-skill"));

        // Verify it's a workspace installation
        let skill = &result.skills["workspace-skill"];
        assert_eq!(skill.installations[0].scope, Scope::Workspace);
    }

    #[test]
    fn test_classify_installation_unmanaged() {
        let temp_dir = TempDir::new().unwrap();

        // Create a physical directory
        let skill_dir = temp_dir.path().join("unmanaged-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test").unwrap();

        // Should be classified as Unmanaged
        let result = classify_installation(&skill_dir);
        assert_eq!(result, InstallationType::Unmanaged);
    }

    #[test]
    fn test_classify_installation_managed() {
        let temp_dir = TempDir::new().unwrap();

        // Create a mock repo structure
        let repo = temp_dir.path().join(".sikil").join("repo");
        let skill = repo.join("my-skill");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# Test Skill").unwrap();

        // Create symlink pointing to repo
        let link = temp_dir.path().join("skill-link");
        std::os::unix::fs::symlink(&skill, &link).unwrap();

        // Temporarily set HOME to temp dir for this test
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());

        let result = classify_installation(&link);
        assert_eq!(result, InstallationType::Managed);

        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[test]
    fn test_classify_installation_broken_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let link = temp_dir.path().join("broken-link");

        // Create a symlink to a non-existent target
        std::os::unix::fs::symlink("/nonexistent/target", &link).unwrap();

        let result = classify_installation(&link);
        assert_eq!(result, InstallationType::BrokenSymlink);
    }

    #[test]
    fn test_classify_installation_foreign_symlink() {
        let temp_dir = TempDir::new().unwrap();

        // Create a skill outside the repo
        let skill = temp_dir.path().join("other-skill");
        fs::create_dir(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# Test Skill").unwrap();

        // Create symlink pointing outside repo
        let link = temp_dir.path().join("skill-link");
        std::os::unix::fs::symlink(&skill, &link).unwrap();

        let result = classify_installation(&link);
        assert_eq!(result, InstallationType::ForeignSymlink);
    }
}
