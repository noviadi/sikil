//! Conflict detection for Agent Skills
//!
//! This module provides functionality for detecting conflicts when multiple
//! skills with the same name exist in different locations.

use crate::core::scanner::ScanResult;
use crate::core::skill::Installation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Type of conflict detected between skill installations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Multiple unmanaged (physical) installations with the same skill name
    /// This indicates the user has duplicate skill directories that should be resolved
    DuplicateUnmanaged,

    /// Multiple managed installations (symlinks) pointing to the same repo location
    /// This is actually OK - they're all symlinks to the same managed skill
    /// This is informational only, not an error
    DuplicateManaged,
}

impl ConflictType {
    /// Returns a human-readable description of this conflict type
    pub fn description(&self) -> &'static str {
        match self {
            ConflictType::DuplicateUnmanaged => {
                "Multiple physical directories with the same skill name. \
                 Only one should exist or they should be consolidated."
            }
            ConflictType::DuplicateManaged => {
                "Multiple symlinks pointing to the same managed skill. \
                 This is normal behavior for a managed skill."
            }
        }
    }

    /// Returns whether this conflict type is an error (requires resolution)
    pub fn is_error(&self) -> bool {
        match self {
            ConflictType::DuplicateUnmanaged => true,
            ConflictType::DuplicateManaged => false,
        }
    }
}

/// Represents a conflict detected for a skill name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// The skill name that has conflicts
    pub skill_name: String,

    /// All installation locations for this skill
    pub locations: Vec<ConflictLocation>,

    /// The type of conflict
    pub conflict_type: ConflictType,
}

impl Conflict {
    /// Creates a new Conflict
    pub fn new(
        skill_name: String,
        locations: Vec<ConflictLocation>,
        conflict_type: ConflictType,
    ) -> Self {
        Self {
            skill_name,
            locations,
            conflict_type,
        }
    }

    /// Returns a human-readable summary of this conflict
    pub fn summary(&self) -> String {
        format!(
            "{}: {} at {} location(s)",
            self.skill_name,
            self.conflict_type_description(),
            self.locations.len()
        )
    }

    /// Returns a description of the conflict type
    fn conflict_type_description(&self) -> &'static str {
        match self.conflict_type {
            ConflictType::DuplicateUnmanaged => "duplicate unmanaged",
            ConflictType::DuplicateManaged => "duplicate managed",
        }
    }

    /// Returns recommendations for resolving this conflict
    pub fn recommendations(&self) -> Vec<String> {
        match self.conflict_type {
            ConflictType::DuplicateUnmanaged => {
                vec![
                    "Remove duplicate skill directories and keep only one".to_string(),
                    "Use 'sikil adopt' to manage one of the duplicates".to_string(),
                    "Rename conflicting directories to use unique skill names".to_string(),
                ]
            }
            ConflictType::DuplicateManaged => {
                vec!["No action needed - this is normal for managed skills".to_string()]
            }
        }
    }
}

/// A single location involved in a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictLocation {
    /// Agent at this location
    pub agent: String,

    /// Path to the skill installation
    pub path: PathBuf,

    /// Whether this is a managed installation (symlink to repo)
    pub is_managed: bool,

    /// If managed, the path to the repo entry
    pub repo_path: Option<PathBuf>,
}

impl ConflictLocation {
    /// Creates a new ConflictLocation from an Installation
    pub fn from_installation(
        installation: &Installation,
        is_managed: bool,
        repo_path: Option<PathBuf>,
    ) -> Self {
        Self {
            agent: installation.agent.to_string(),
            path: installation.path.clone(),
            is_managed,
            repo_path,
        }
    }

    /// Creates a new ConflictLocation manually
    pub fn new(agent: String, path: PathBuf, is_managed: bool, repo_path: Option<PathBuf>) -> Self {
        Self {
            agent,
            path,
            is_managed,
            repo_path,
        }
    }
}

/// Detects conflicts in a scan result
///
/// This function analyzes the scan result to identify:
/// - **DuplicateUnmanaged**: Multiple unmanaged installations with the same skill name
/// - **DuplicateManaged**: Multiple managed installations (symlinks to same repo)
///
/// An installation is classified as managed when `is_symlink == Some(true)` AND
/// the symlink target starts with the global repo path or the project-managed
/// store path.
///
/// Managed installations from different scopes (global vs project) do not
/// conflict with each other.
///
/// # Arguments
///
/// * `scan_result` - The scan result to analyze for conflicts
/// * `repo_path` - The global repo root path (e.g., `~/.sikil/repo/`)
/// * `project_skills_path` - Optional project skills root path
///   (e.g., `<project_root>/.sikil/skills/`)
///
/// # Returns
///
/// A vector of conflicts detected. Returns empty vector if no conflicts found.
pub fn detect_conflicts(
    scan_result: &ScanResult,
    repo_path: &Path,
    project_skills_path: Option<&Path>,
) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    for (skill_name, skill) in &scan_result.skills {
        let mut unmanaged_locations: Vec<ConflictLocation> = Vec::new();
        let mut managed_groups: HashMap<PathBuf, Vec<ConflictLocation>> = HashMap::new();

        for installation in &skill.installations {
            let symlink_target = installation.symlink_target.as_ref();
            let is_managed = installation.is_symlink == Some(true)
                && symlink_target
                    .map(|t| {
                        t.starts_with(repo_path)
                            || project_skills_path.is_some_and(|psp| t.starts_with(psp))
                    })
                    .unwrap_or(false);

            if is_managed {
                let inst_repo_path = symlink_target.unwrap().clone();
                managed_groups
                    .entry(inst_repo_path.clone())
                    .or_default()
                    .push(ConflictLocation::from_installation(
                        installation,
                        true,
                        Some(inst_repo_path),
                    ));
            } else {
                unmanaged_locations.push(ConflictLocation::from_installation(
                    installation,
                    false,
                    None,
                ));
            }
        }

        // Check for duplicate unmanaged conflicts
        if unmanaged_locations.len() > 1 {
            let unique_paths: std::collections::HashSet<&PathBuf> =
                unmanaged_locations.iter().map(|loc| &loc.path).collect();

            if unique_paths.len() > 1 {
                conflicts.push(Conflict::new(
                    skill_name.clone(),
                    unmanaged_locations,
                    ConflictType::DuplicateUnmanaged,
                ));
            }
        }

        // Check for duplicate managed conflicts within each repo_path group
        for locations in managed_groups.into_values() {
            if locations.len() > 1 {
                conflicts.push(Conflict::new(
                    skill_name.clone(),
                    locations,
                    ConflictType::DuplicateManaged,
                ));
            }
        }
    }

    conflicts
}

/// Filters conflicts to only return error-level conflicts
///
/// This is useful for commands that want to report only problems
/// that require user action.
///
/// # Arguments
///
/// * `conflicts` - The conflicts to filter
///
/// # Returns
///
/// A vector containing only the conflicts that are errors
pub fn filter_error_conflicts(conflicts: &[Conflict]) -> Vec<&Conflict> {
    conflicts
        .iter()
        .filter(|c| c.conflict_type.is_error())
        .collect()
}

/// Filters conflicts for display based on verbose mode
///
/// When verbose is false, this excludes `DuplicateManaged` conflicts (informational only).
/// When verbose is true, all conflicts are included.
///
/// # Arguments
///
/// * `conflicts` - The conflicts to filter
/// * `verbose` - Whether to include informational conflicts
///
/// # Returns
///
/// A vector containing conflicts that should be displayed
pub fn filter_displayable_conflicts(conflicts: &[Conflict], verbose: bool) -> Vec<&Conflict> {
    if verbose {
        conflicts.iter().collect()
    } else {
        conflicts
            .iter()
            .filter(|c| c.conflict_type.is_error())
            .collect()
    }
}

/// Formats a conflict for human-readable display
///
/// # Arguments
///
/// * `conflict` - The conflict to format
///
/// # Returns
///
/// A formatted string with conflict details
pub fn format_conflict(conflict: &Conflict) -> String {
    let mut result = String::new();

    // Conflict header with skill name
    let status_indicator = if conflict.conflict_type.is_error() {
        "✗"
    } else {
        "ℹ"
    };
    result.push_str(&format!(
        "{} {} ({})\n",
        status_indicator,
        conflict.skill_name,
        conflict.conflict_type_description()
    ));

    // Description
    result.push_str(&format!("  {}\n", conflict.conflict_type.description()));

    // Locations
    result.push_str("  Locations:\n");
    for (i, location) in conflict.locations.iter().enumerate() {
        let managed_status = if location.is_managed {
            "managed"
        } else {
            "unmanaged"
        };
        result.push_str(&format!(
            "    {}. {} ({}) @ {}\n",
            i + 1,
            location.agent,
            managed_status,
            location.path.display()
        ));
        if let Some(ref repo) = location.repo_path {
            result.push_str(&format!("       → repo: {}\n", repo.display()));
        }
    }

    result
}

/// Formats conflicts as a summary with counts
///
/// # Arguments
///
/// * `conflicts` - The conflicts to summarize
/// * `verbose` - Whether to show "N info" (true) or "N info suppressed" (false)
///
/// # Returns
///
/// A formatted summary string
pub fn format_conflicts_summary(conflicts: &[Conflict], verbose: bool) -> String {
    let error_count = conflicts
        .iter()
        .filter(|c| c.conflict_type.is_error())
        .count();
    let info_count = conflicts.len() - error_count;

    let mut parts = Vec::new();
    if error_count > 0 {
        parts.push(format!(
            "{} error{}",
            error_count,
            if error_count == 1 { "" } else { "s" }
        ));
    }
    if info_count > 0 {
        let info_text = if verbose {
            format!("{} info", info_count)
        } else {
            format!("{} info suppressed", info_count)
        };
        parts.push(info_text);
    }

    if parts.is_empty() {
        "No conflicts detected".to_string()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::skill::{Agent, Scope, Skill, SkillMetadata};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_conflict_type_description() {
        assert_eq!(
            ConflictType::DuplicateUnmanaged.description(),
            "Multiple physical directories with the same skill name. \
             Only one should exist or they should be consolidated."
        );
        assert_eq!(
            ConflictType::DuplicateManaged.description(),
            "Multiple symlinks pointing to the same managed skill. \
             This is normal behavior for a managed skill."
        );
    }

    #[test]
    fn test_conflict_type_is_error() {
        assert!(ConflictType::DuplicateUnmanaged.is_error());
        assert!(!ConflictType::DuplicateManaged.is_error());
    }

    #[test]
    fn test_conflict_new() {
        let locations = vec![ConflictLocation::new(
            "claude-code".to_string(),
            PathBuf::from("/test/skill"),
            false,
            None,
        )];

        let conflict = Conflict::new(
            "test-skill".to_string(),
            locations,
            ConflictType::DuplicateUnmanaged,
        );

        assert_eq!(conflict.skill_name, "test-skill");
        assert_eq!(conflict.locations.len(), 1);
        assert_eq!(conflict.conflict_type, ConflictType::DuplicateUnmanaged);
    }

    #[test]
    fn test_conflict_summary() {
        let locations = vec![
            ConflictLocation::new(
                "claude-code".to_string(),
                PathBuf::from("/test1"),
                false,
                None,
            ),
            ConflictLocation::new("windsurf".to_string(), PathBuf::from("/test2"), false, None),
        ];

        let conflict = Conflict::new(
            "test-skill".to_string(),
            locations,
            ConflictType::DuplicateUnmanaged,
        );

        let summary = conflict.summary();
        assert!(summary.contains("test-skill"));
        assert!(summary.contains("duplicate unmanaged"));
        assert!(summary.contains("2 location"));
    }

    #[test]
    fn test_conflict_recommendations_unmanaged() {
        let conflict = Conflict::new("test".to_string(), vec![], ConflictType::DuplicateUnmanaged);

        let recs = conflict.recommendations();
        assert_eq!(recs.len(), 3);
        assert!(recs[0].contains("Remove duplicate"));
        assert!(recs[1].contains("adopt"));
        assert!(recs[2].contains("Rename"));
    }

    #[test]
    fn test_conflict_recommendations_managed() {
        let conflict = Conflict::new("test".to_string(), vec![], ConflictType::DuplicateManaged);

        let recs = conflict.recommendations();
        assert_eq!(recs.len(), 1);
        assert!(recs[0].contains("No action needed"));
    }

    #[test]
    fn test_conflict_location_new() {
        let location = ConflictLocation::new(
            "claude-code".to_string(),
            PathBuf::from("/test/skill"),
            true,
            Some(PathBuf::from("/repo/skill")),
        );

        assert_eq!(location.agent, "claude-code");
        assert_eq!(location.path, PathBuf::from("/test/skill"));
        assert!(location.is_managed);
        assert_eq!(location.repo_path, Some(PathBuf::from("/repo/skill")));
    }

    #[test]
    fn test_detect_conflicts_no_conflicts() {
        let mut scan_result = ScanResult::new();
        let repo_path = PathBuf::from("/home/user/.sikil/repo");

        let metadata = SkillMetadata::new("test-skill".to_string(), "A test".to_string());
        let skill = Skill::new(metadata, "test-skill".to_string());
        scan_result.skills.insert("test-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_path, None);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_duplicate_unmanaged() {
        let mut scan_result = ScanResult::new();
        let repo_path = PathBuf::from("/home/user/.sikil/repo");

        let metadata = SkillMetadata::new("dupe-skill".to_string(), "Duplicate".to_string());
        let mut skill = Skill::new(metadata, "dupe-skill".to_string());

        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/dupe-skill"),
                Scope::Global,
            )
            .with_is_symlink(false),
        );

        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/skills/dupe-skill"),
                Scope::Global,
            )
            .with_is_symlink(false),
        );

        scan_result.skills.insert("dupe-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_path, None);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].skill_name, "dupe-skill");
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicateUnmanaged);
        assert_eq!(conflicts[0].locations.len(), 2);
    }

    #[test]
    fn test_detect_conflicts_duplicate_managed() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("repo");
        let skill_repo = repo_root.join("managed-skill");
        fs::create_dir_all(&skill_repo).unwrap();

        let mut scan_result = ScanResult::new();

        let metadata = SkillMetadata::new("managed-skill".to_string(), "Managed".to_string());
        let mut skill = Skill::new(metadata, "managed-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(skill_repo.clone());

        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/managed-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/skills/managed-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        scan_result
            .skills
            .insert("managed-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, None);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].skill_name, "managed-skill");
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicateManaged);
        assert!(!conflicts[0].conflict_type.is_error());
    }

    #[test]
    fn test_detect_conflicts_same_path_not_duplicate() {
        let mut scan_result = ScanResult::new();
        let repo_path = PathBuf::from("/home/user/.sikil/repo");

        let metadata = SkillMetadata::new("same-path".to_string(), "Same path".to_string());
        let mut skill = Skill::new(metadata, "same-path".to_string());

        let same_path = PathBuf::from("/shared/skills/same-path");
        skill.installations.push(
            Installation::new(Agent::ClaudeCode, same_path.clone(), Scope::Global)
                .with_is_symlink(false),
        );

        skill.installations.push(
            Installation::new(Agent::Windsurf, same_path, Scope::Global).with_is_symlink(false),
        );

        scan_result.skills.insert("same-path".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_path, None);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_mixed_managed_unmanaged() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("repo");
        let skill_repo = repo_root.join("mixed-skill");
        fs::create_dir_all(&skill_repo).unwrap();

        let mut scan_result = ScanResult::new();

        let metadata = SkillMetadata::new("mixed-skill".to_string(), "Mixed".to_string());
        let mut skill = Skill::new(metadata, "mixed-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(skill_repo.clone());

        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/mixed-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/skills/mixed-skill"),
                Scope::Global,
            )
            .with_is_symlink(false),
        );

        scan_result.skills.insert("mixed-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, None);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_filter_error_conflicts() {
        let conflicts = vec![
            Conflict::new(
                "error-skill".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new(
                "info-skill".to_string(),
                vec![],
                ConflictType::DuplicateManaged,
            ),
        ];

        let errors = filter_error_conflicts(&conflicts);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].skill_name, "error-skill");
    }

    #[test]
    fn test_filter_error_conflicts_empty() {
        let conflicts = vec![
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info2".to_string(), vec![], ConflictType::DuplicateManaged),
        ];

        let errors = filter_error_conflicts(&conflicts);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_empty_result() {
        let scan_result = ScanResult::new();
        let repo_path = PathBuf::from("/home/user/.sikil/repo");
        let conflicts = detect_conflicts(&scan_result, &repo_path, None);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_conflicts_multiple_skills() {
        let mut scan_result = ScanResult::new();
        let repo_path = PathBuf::from("/home/user/.sikil/repo");

        // Add first skill with conflict
        let metadata1 = SkillMetadata::new("conflict-1".to_string(), "Conflict 1".to_string());
        let mut skill1 = Skill::new(metadata1, "conflict-1".to_string());
        skill1.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/conflict-1"),
                Scope::Global,
            )
            .with_is_symlink(false),
        );
        skill1.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/conflict-1"),
                Scope::Global,
            )
            .with_is_symlink(false),
        );
        scan_result.skills.insert("conflict-1".to_string(), skill1);

        // Add second skill with no conflict
        let metadata2 = SkillMetadata::new("no-conflict".to_string(), "No conflict".to_string());
        let skill2 = Skill::new(metadata2, "no-conflict".to_string());
        scan_result.skills.insert("no-conflict".to_string(), skill2);

        // Add third skill with conflict
        let metadata3 = SkillMetadata::new("conflict-2".to_string(), "Conflict 2".to_string());
        let mut skill3 = Skill::new(metadata3, "conflict-2".to_string());
        skill3.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/conflict-2"),
                Scope::Global,
            )
            .with_is_symlink(false),
        );
        skill3.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/conflict-2"),
                Scope::Global,
            )
            .with_is_symlink(false),
        );
        scan_result.skills.insert("conflict-2".to_string(), skill3);

        let conflicts = detect_conflicts(&scan_result, &repo_path, None);
        assert_eq!(conflicts.len(), 2);

        let conflict_names: Vec<_> = conflicts.iter().map(|c| &c.skill_name).collect();
        assert!(conflict_names.contains(&&"conflict-1".to_string()));
        assert!(conflict_names.contains(&&"conflict-2".to_string()));
    }

    #[test]
    fn test_conflict_location_from_installation() {
        let installation = Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/test/skill"),
            Scope::Global,
        );

        let location = ConflictLocation::from_installation(&installation, false, None);
        assert_eq!(location.agent, "claude-code");
        assert_eq!(location.path, PathBuf::from("/test/skill"));
        assert!(!location.is_managed);
        assert!(location.repo_path.is_none());

        let location_managed = ConflictLocation::from_installation(
            &installation,
            true,
            Some(PathBuf::from("/repo/skill")),
        );
        assert!(location_managed.is_managed);
        assert_eq!(
            location_managed.repo_path,
            Some(PathBuf::from("/repo/skill"))
        );
    }

    #[test]
    fn test_detect_conflicts_with_symlink_target_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("repo");
        let skill_repo = repo_root.join("symlink-skill");
        fs::create_dir_all(&skill_repo).unwrap();

        let mut scan_result = ScanResult::new();

        let metadata = SkillMetadata::new("symlink-skill".to_string(), "Symlink".to_string());
        let mut skill = Skill::new(metadata, "symlink-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(skill_repo.clone());

        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/symlink-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        scan_result
            .skills
            .insert("symlink-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, None);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_format_conflict_unmanaged() {
        let locations = vec![
            ConflictLocation::new(
                "claude-code".to_string(),
                PathBuf::from("/claude/skills/test"),
                false,
                None,
            ),
            ConflictLocation::new(
                "windsurf".to_string(),
                PathBuf::from("/windsurf/skills/test"),
                false,
                None,
            ),
        ];

        let conflict = Conflict::new(
            "test-skill".to_string(),
            locations,
            ConflictType::DuplicateUnmanaged,
        );
        let formatted = format_conflict(&conflict);

        assert!(formatted.contains("test-skill"));
        assert!(formatted.contains("duplicate unmanaged"));
        assert!(formatted.contains("claude-code"));
        assert!(formatted.contains("windsurf"));
        assert!(formatted.contains("unmanaged"));
        assert!(formatted.contains("✗")); // Error indicator
    }

    #[test]
    fn test_format_conflict_managed() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("repo").join("managed");
        fs::create_dir_all(&repo_path).unwrap();

        let locations = vec![
            ConflictLocation::new(
                "claude-code".to_string(),
                PathBuf::from("/claude/skills/managed"),
                true,
                Some(repo_path.clone()),
            ),
            ConflictLocation::new(
                "windsurf".to_string(),
                PathBuf::from("/windsurf/skills/managed"),
                true,
                Some(repo_path),
            ),
        ];

        let conflict = Conflict::new(
            "managed-skill".to_string(),
            locations,
            ConflictType::DuplicateManaged,
        );
        let formatted = format_conflict(&conflict);

        assert!(formatted.contains("managed-skill"));
        assert!(formatted.contains("duplicate managed"));
        assert!(formatted.contains("managed"));
        assert!(formatted.contains("repo:")); // Shows repo path
        assert!(formatted.contains("ℹ")); // Info indicator (not error)
    }

    #[test]
    fn test_format_conflicts_summary_empty() {
        let conflicts: Vec<Conflict> = vec![];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "No conflicts detected");
    }

    #[test]
    fn test_format_conflicts_summary_errors_only() {
        let conflicts = vec![
            Conflict::new(
                "error1".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new(
                "error2".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
        ];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "2 errors");
    }

    #[test]
    fn test_format_conflicts_summary_info_only() {
        let conflicts = vec![
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info2".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info3".to_string(), vec![], ConflictType::DuplicateManaged),
        ];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "3 info");
    }

    #[test]
    fn test_format_conflicts_summary_mixed() {
        let conflicts = vec![
            Conflict::new(
                "error1".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new(
                "error2".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
        ];
        let summary = format_conflicts_summary(&conflicts, true);
        assert!(summary.contains("2 errors"));
        assert!(summary.contains("1 info"));
    }

    #[test]
    fn test_filter_displayable_conflicts_verbose_false_excludes_managed() {
        let conflicts = vec![
            Conflict::new(
                "error-skill".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new(
                "managed-skill".to_string(),
                vec![],
                ConflictType::DuplicateManaged,
            ),
        ];

        let displayable = filter_displayable_conflicts(&conflicts, false);
        assert_eq!(displayable.len(), 1);
        assert_eq!(displayable[0].skill_name, "error-skill");
    }

    #[test]
    fn test_filter_displayable_conflicts_verbose_true_includes_all() {
        let conflicts = vec![
            Conflict::new(
                "error-skill".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new(
                "managed-skill".to_string(),
                vec![],
                ConflictType::DuplicateManaged,
            ),
        ];

        let displayable = filter_displayable_conflicts(&conflicts, true);
        assert_eq!(displayable.len(), 2);
        let names: Vec<_> = displayable.iter().map(|c| &c.skill_name).collect();
        assert!(names.contains(&&"error-skill".to_string()));
        assert!(names.contains(&&"managed-skill".to_string()));
    }

    #[test]
    fn test_filter_displayable_conflicts_verbose_false_only_managed_returns_empty() {
        let conflicts = vec![
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info2".to_string(), vec![], ConflictType::DuplicateManaged),
        ];

        let displayable = filter_displayable_conflicts(&conflicts, false);
        assert_eq!(displayable.len(), 0);
    }

    #[test]
    fn test_filter_displayable_conflicts_verbose_true_only_managed_includes_all() {
        let conflicts = vec![
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info2".to_string(), vec![], ConflictType::DuplicateManaged),
        ];

        let displayable = filter_displayable_conflicts(&conflicts, true);
        assert_eq!(displayable.len(), 2);
    }

    #[test]
    fn test_filter_displayable_conflicts_empty_conflicts() {
        let conflicts: Vec<Conflict> = vec![];

        let displayable_false = filter_displayable_conflicts(&conflicts, false);
        assert_eq!(displayable_false.len(), 0);

        let displayable_true = filter_displayable_conflicts(&conflicts, true);
        assert_eq!(displayable_true.len(), 0);
    }

    #[test]
    fn test_filter_displayable_conflicts_verbose_false_only_unmanaged_includes_all() {
        let conflicts = vec![
            Conflict::new(
                "error1".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new(
                "error2".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
        ];

        let displayable = filter_displayable_conflicts(&conflicts, false);
        assert_eq!(displayable.len(), 2);
    }

    // Tests for format_conflicts_summary with verbose parameter

    #[test]
    fn test_format_conflicts_summary_verbose_false_info_only_shows_suppressed() {
        let conflicts = vec![
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info2".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info3".to_string(), vec![], ConflictType::DuplicateManaged),
        ];
        let summary = format_conflicts_summary(&conflicts, false);
        assert_eq!(summary, "3 info suppressed");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_true_info_only_shows_info() {
        let conflicts = vec![
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info2".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new("info3".to_string(), vec![], ConflictType::DuplicateManaged),
        ];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "3 info");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_false_errors_only_shows_errors() {
        let conflicts = vec![
            Conflict::new(
                "error1".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new(
                "error2".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
        ];
        let summary = format_conflicts_summary(&conflicts, false);
        assert_eq!(summary, "2 errors");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_true_errors_only_shows_errors() {
        let conflicts = vec![
            Conflict::new(
                "error1".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new(
                "error2".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
        ];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "2 errors");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_false_mixed_shows_suppressed() {
        let conflicts = vec![
            Conflict::new(
                "error1".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new(
                "error2".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
        ];
        let summary = format_conflicts_summary(&conflicts, false);
        assert_eq!(summary, "2 errors, 1 info suppressed");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_true_mixed_shows_info() {
        let conflicts = vec![
            Conflict::new(
                "error1".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
            Conflict::new("info1".to_string(), vec![], ConflictType::DuplicateManaged),
            Conflict::new(
                "error2".to_string(),
                vec![],
                ConflictType::DuplicateUnmanaged,
            ),
        ];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "2 errors, 1 info");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_false_empty_shows_no_conflicts() {
        let conflicts: Vec<Conflict> = vec![];
        let summary = format_conflicts_summary(&conflicts, false);
        assert_eq!(summary, "No conflicts detected");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_true_empty_shows_no_conflicts() {
        let conflicts: Vec<Conflict> = vec![];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "No conflicts detected");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_false_single_error() {
        let conflicts = vec![Conflict::new(
            "error1".to_string(),
            vec![],
            ConflictType::DuplicateUnmanaged,
        )];
        let summary = format_conflicts_summary(&conflicts, false);
        assert_eq!(summary, "1 error");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_false_single_info_suppressed() {
        let conflicts = vec![Conflict::new(
            "info1".to_string(),
            vec![],
            ConflictType::DuplicateManaged,
        )];
        let summary = format_conflicts_summary(&conflicts, false);
        assert_eq!(summary, "1 info suppressed");
    }

    #[test]
    fn test_format_conflicts_summary_verbose_true_single_info() {
        let conflicts = vec![Conflict::new(
            "info1".to_string(),
            vec![],
            ConflictType::DuplicateManaged,
        )];
        let summary = format_conflicts_summary(&conflicts, true);
        assert_eq!(summary, "1 info");
    }

    // --- Project-aware conflict detection tests ---

    /// AC1: Managed when symlink target is under global repo path
    #[test]
    fn test_managed_when_symlink_under_global_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("repo");
        let skill_repo = repo_root.join("my-skill");
        fs::create_dir_all(&skill_repo).unwrap();

        let mut scan_result = ScanResult::new();
        let metadata = SkillMetadata::new("my-skill".to_string(), "Test".to_string());
        let mut skill = Skill::new(metadata, "my-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(skill_repo.clone());

        // Symlink to global repo -> managed
        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/my-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        // Another symlink to same global repo -> DuplicateManaged
        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/skills/my-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        scan_result.skills.insert("my-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, None);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicateManaged);
        assert!(conflicts[0].locations.iter().all(|l| l.is_managed));
    }

    /// AC1: Managed when symlink target is under project skills path
    #[test]
    fn test_managed_when_symlink_under_project_skills() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("global-repo");
        let project_root = temp_dir.path().join("project");
        let project_skills = project_root.join(".sikil").join("skills");
        let skill_repo = project_skills.join("my-skill");
        fs::create_dir_all(&skill_repo).unwrap();

        let mut scan_result = ScanResult::new();
        let metadata = SkillMetadata::new("my-skill".to_string(), "Test".to_string());
        let mut skill = Skill::new(metadata, "my-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(skill_repo.clone());

        // Symlink to project skills -> managed
        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                project_root.join(".claude/skills/my-skill"),
                Scope::Workspace,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        // Another symlink to same project skill -> DuplicateManaged
        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                project_root.join(".windsurf/skills/my-skill"),
                Scope::Workspace,
            )
            .with_is_symlink(true)
            .with_symlink_target(skill_repo.clone()),
        );

        scan_result.skills.insert("my-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, Some(&project_skills));
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicateManaged);
    }

    /// AC1: Not managed when symlink target is outside both stores
    #[test]
    fn test_not_managed_when_symlink_outside_stores() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("global-repo");
        let project_skills = temp_dir.path().join("project/.sikil/skills");

        let mut scan_result = ScanResult::new();
        let metadata = SkillMetadata::new("my-skill".to_string(), "Test".to_string());
        let mut skill = Skill::new(metadata, "my-skill".to_string());

        // Symlink to location outside both stores -> not managed
        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/my-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(PathBuf::from("/random/place/my-skill")),
        );

        // Another symlink to a different outside location
        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/skills/my-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(PathBuf::from("/other/random/place/my-skill")),
        );

        scan_result.skills.insert("my-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, Some(&project_skills));
        // Both are classified as unmanaged since targets are outside stores
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicateUnmanaged);
    }

    /// AC3: Cross-scope (global + project) installations do NOT conflict
    #[test]
    fn test_cross_scope_no_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("global-repo");
        let global_skill = repo_root.join("my-skill");
        fs::create_dir_all(&global_skill).unwrap();

        let project_skills = temp_dir.path().join("project/.sikil/skills");
        let project_skill = project_skills.join("my-skill");
        fs::create_dir_all(&project_skill).unwrap();

        let mut scan_result = ScanResult::new();
        let metadata = SkillMetadata::new("my-skill".to_string(), "Test".to_string());
        let mut skill = Skill::new(metadata, "my-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(global_skill.clone());

        // Global managed installation
        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/my-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(global_skill.clone()),
        );

        // Project managed installation (different scope)
        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/project/.windsurf/skills/my-skill"),
                Scope::Workspace,
            )
            .with_is_symlink(true)
            .with_symlink_target(project_skill.clone()),
        );

        scan_result.skills.insert("my-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, Some(&project_skills));
        // Cross-scope should NOT conflict
        assert_eq!(conflicts.len(), 0);
    }

    /// AC3: Multiple global + multiple project managed -> DuplicateManaged within each scope only
    #[test]
    fn test_cross_scope_duplicate_managed_within_each_scope() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("global-repo");
        let global_skill = repo_root.join("my-skill");
        fs::create_dir_all(&global_skill).unwrap();

        let project_skills = temp_dir.path().join("project/.sikil/skills");
        let project_skill = project_skills.join("my-skill");
        fs::create_dir_all(&project_skill).unwrap();

        let mut scan_result = ScanResult::new();
        let metadata = SkillMetadata::new("my-skill".to_string(), "Test".to_string());
        let mut skill = Skill::new(metadata, "my-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(global_skill.clone());

        // Two global managed installations
        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/claude/skills/my-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(global_skill.clone()),
        );
        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/windsurf/skills/my-skill"),
                Scope::Global,
            )
            .with_is_symlink(true)
            .with_symlink_target(global_skill.clone()),
        );

        // Two project managed installations
        skill.installations.push(
            Installation::new(
                Agent::OpenCode,
                PathBuf::from("/project/.opencode/skill/my-skill"),
                Scope::Workspace,
            )
            .with_is_symlink(true)
            .with_symlink_target(project_skill.clone()),
        );
        skill.installations.push(
            Installation::new(
                Agent::KiloCode,
                PathBuf::from("/project/.kilocode/skills/my-skill"),
                Scope::Workspace,
            )
            .with_is_symlink(true)
            .with_symlink_target(project_skill.clone()),
        );

        scan_result.skills.insert("my-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, Some(&project_skills));
        // Should have 2 DuplicateManaged: one for global, one for project
        assert_eq!(conflicts.len(), 2);
        assert!(conflicts
            .iter()
            .all(|c| c.conflict_type == ConflictType::DuplicateManaged));

        // Each conflict should have exactly 2 locations
        assert!(conflicts.iter().all(|c| c.locations.len() == 2));
    }

    /// AC2: Two managed installs with same repo_path are considered duplicates
    #[test]
    fn test_duplicate_managed_same_repo_path_within_project() {
        let temp_dir = TempDir::new().unwrap();
        let repo_root = temp_dir.path().join("global-repo");
        let project_skills = temp_dir.path().join("project/.sikil/skills");
        let project_skill = project_skills.join("shared-skill");
        fs::create_dir_all(&project_skill).unwrap();

        let mut scan_result = ScanResult::new();
        let metadata = SkillMetadata::new("shared-skill".to_string(), "Test".to_string());
        let mut skill = Skill::new(metadata, "shared-skill".to_string());
        skill.is_managed = true;
        skill.repo_path = Some(project_skill.clone());

        // Two project-managed symlinks to same repo_path
        skill.installations.push(
            Installation::new(
                Agent::ClaudeCode,
                PathBuf::from("/project/.claude/skills/shared-skill"),
                Scope::Workspace,
            )
            .with_is_symlink(true)
            .with_symlink_target(project_skill.clone()),
        );
        skill.installations.push(
            Installation::new(
                Agent::Windsurf,
                PathBuf::from("/project/.windsurf/skills/shared-skill"),
                Scope::Workspace,
            )
            .with_is_symlink(true)
            .with_symlink_target(project_skill.clone()),
        );

        scan_result.skills.insert("shared-skill".to_string(), skill);

        let conflicts = detect_conflicts(&scan_result, &repo_root, Some(&project_skills));
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicateManaged);
        assert_eq!(conflicts[0].locations.len(), 2);
    }
}
