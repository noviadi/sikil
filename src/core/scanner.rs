//! Directory scanner for discovering Agent Skills
//!
//! This module provides functionality for scanning directories to find
//! Agent Skills, which are identified by the presence of SKILL.md files
//! in subdirectories.

use crate::core::config::Config;
use crate::core::errors::SikilError;
use crate::core::parser::parse_skill_md;
use crate::core::skill::{Agent, Installation, Scope, Skill, SkillMetadata};
use crate::utils::symlink::read_symlink_target;
use fs_err as fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
}
