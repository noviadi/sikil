//! Show command implementation
//!
//! This module provides functionality for displaying detailed information
//! about a specific Agent Skill.

use crate::cli::output::Output;
use crate::core::config::Config;
use crate::core::errors::SikilError;
use crate::core::scanner::Scanner;
use anyhow::Result;

/// Arguments for the show command
#[derive(Debug, Clone)]
pub struct ShowArgs {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Whether to disable cache
    pub no_cache: bool,
    /// Name of the skill to show
    pub name: String,
}

/// Output format for the show command
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShowOutput {
    /// Skill name from metadata
    pub name: String,
    /// Directory name (if different from name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_name: Option<String>,
    /// Description
    pub description: String,
    /// Version (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Author (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// License (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Whether this skill is managed
    pub managed: bool,
    /// Canonical path (for managed skills)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_path: Option<String>,
    /// All installations of this skill
    pub installations: Vec<ShowInstallationOutput>,
    /// File tree information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_tree: Option<ShowFileTree>,
    /// Total size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_size_bytes: Option<u64>,
}

/// Output format for a single installation
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShowInstallationOutput {
    /// Agent name
    pub agent: String,
    /// Path to the installation
    pub path: String,
    /// Scope (global or workspace)
    pub scope: String,
    /// Whether this is a symlink
    pub is_symlink: bool,
    /// Symlink target (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<String>,
}

/// Output format for file tree information
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShowFileTree {
    /// Whether SKILL.md exists
    pub has_skill_md: bool,
    /// Whether scripts/ directory exists
    pub has_scripts_dir: bool,
    /// Whether references/ directory exists
    pub has_references_dir: bool,
    /// Total file count
    pub file_count: usize,
}

/// Executes the show command
///
/// This function:
/// 1. Creates a scanner with the given configuration
/// 2. Scans all agent directories for skills
/// 3. Finds the skill by name across all installations
/// 4. Aggregates all installations for the skill
/// 5. Returns error if skill not found
///
/// # Arguments
///
/// * `args` - Show arguments including name, json_mode, and cache settings
/// * `config` - Agent configuration
///
/// # Errors
///
/// Returns an error if:
/// - The skill is not found across any agent directories
///
/// # Examples
///
/// ```no_run
/// use sikil::commands::show::{execute_show, ShowArgs};
/// use sikil::core::config::Config;
///
/// let args = ShowArgs {
///     json_mode: false,
///     no_cache: false,
///     name: "my-skill".to_string(),
/// };
/// let config = Config::default();
/// execute_show(args, &config).unwrap();
/// ```
pub fn execute_show(args: ShowArgs, config: &Config) -> Result<()> {
    let output = Output::new(args.json_mode);

    // Create scanner (with or without cache based on args)
    let scanner = if args.no_cache {
        Scanner::without_cache(config.clone())
    } else {
        Scanner::new(config.clone())
    };

    // Scan all agents
    let scan_result = scanner.scan_all_agents();

    // Find the skill by name
    let skill = scan_result
        .skills
        .get(&args.name)
        .ok_or_else(|| SikilError::SkillNotFound {
            name: args.name.clone(),
        })?;

    // Build output structure
    let show_output = build_show_output(skill, &args.name)?;

    // Output results
    if args.json_mode {
        output.print_json(&show_output)?;
    } else {
        print_human_readable(&output, &show_output);
    }

    Ok(())
}

/// Builds the ShowOutput structure from a Skill
fn build_show_output(skill: &crate::core::skill::Skill, name: &str) -> Result<ShowOutput> {
    // Build installations list
    let installations: Vec<ShowInstallationOutput> = skill
        .installations
        .iter()
        .map(|inst| ShowInstallationOutput {
            agent: inst.agent.to_string(),
            path: inst.path.to_string_lossy().to_string(),
            scope: format_scope(inst.scope),
            is_symlink: inst.is_symlink.unwrap_or(false),
            symlink_target: inst
                .symlink_target
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
        })
        .collect();

    // Get canonical path for managed skills
    let canonical_path = if skill.is_managed {
        skill
            .repo_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
    } else {
        // For unmanaged skills, use the first installation path
        skill
            .installations
            .first()
            .map(|inst| inst.path.to_string_lossy().to_string())
    };

    // Determine the base path for file tree and size calculation
    let base_path = skill
        .repo_path
        .as_deref()
        .or_else(|| skill.installations.first().map(|i| i.path.as_path()));

    // Build file tree information (only if we have a path to examine)
    let (file_tree, total_size_bytes) = if let Some(bp) = base_path {
        let tree = build_file_tree(bp);
        let size = calculate_dir_size(bp).ok();
        (Some(tree), size)
    } else {
        (None, None)
    };

    let directory_name = if skill.directory_name != name {
        Some(skill.directory_name.clone())
    } else {
        None
    };

    Ok(ShowOutput {
        name: skill.metadata.name.clone(),
        directory_name,
        description: skill.metadata.description.clone(),
        version: skill.metadata.version.clone(),
        author: skill.metadata.author.clone(),
        license: skill.metadata.license.clone(),
        managed: skill.is_managed,
        canonical_path,
        installations,
        file_tree,
        total_size_bytes,
    })
}

/// Builds file tree information for a skill directory
fn build_file_tree(path: &std::path::Path) -> ShowFileTree {
    let skill_md_path = path.join("SKILL.md");
    let scripts_path = path.join("scripts");
    let references_path = path.join("references");

    let has_skill_md = skill_md_path.exists();
    let has_scripts_dir = scripts_path.is_dir();
    let has_references_dir = references_path.is_dir();

    // Count files in the skill directory
    let file_count = count_files(path);

    ShowFileTree {
        has_skill_md,
        has_scripts_dir,
        has_references_dir,
        file_count,
    }
}

/// Counts files in a directory recursively
fn count_files(path: &std::path::Path) -> usize {
    use fs_err as fs;

    let mut count = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                // Skip .git directory
                if let Some(name) = entry_path.file_name() {
                    if name == ".git" {
                        continue;
                    }
                }
                count += count_files(&entry_path);
            } else if entry_path.is_file() {
                count += 1;
            }
        }
    }

    count
}

/// Calculates the total size of a directory in bytes
fn calculate_dir_size(path: &std::path::Path) -> Result<u64, SikilError> {
    use fs_err as fs;

    let mut total_size = 0;

    let entries = fs::read_dir(path).map_err(|_| SikilError::DirectoryNotFound {
        path: path.to_path_buf(),
    })?;

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|_| SikilError::DirectoryNotFound {
                path: entry_path.clone(),
            })?;

        if file_type.is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        } else if file_type.is_dir() {
            // Skip .git directory
            if let Some(name) = entry_path.file_name() {
                if name == ".git" {
                    continue;
                }
            }
            total_size += calculate_dir_size(&entry_path)?;
        }
    }

    Ok(total_size)
}

/// Formats the scope enum as a display string
fn format_scope(scope: crate::core::skill::Scope) -> String {
    match scope {
        crate::core::skill::Scope::Global => "global".to_string(),
        crate::core::skill::Scope::Workspace => "workspace".to_string(),
    }
}

/// Prints human-readable output for the show command
fn print_human_readable(output: &Output, show_output: &ShowOutput) {
    // Print header with skill name
    output.print_success(&format!("Skill: {}", show_output.name));

    // Print directory name note if different
    if let Some(ref dir_name) = show_output.directory_name {
        output.print_info(&format!("Directory: {}", dir_name));
    }

    // Print managed status
    if show_output.managed {
        output.print_success("Status: Managed");
        if let Some(ref canonical) = show_output.canonical_path {
            output.print_info(&format!("Canonical: {}", canonical));
        }
    } else {
        output.print_warning("Status: Unmanaged");
    }

    // Print description
    output.print_info(&format!("Description: {}", show_output.description));

    // Print optional metadata
    if let Some(ref version) = show_output.version {
        output.print_info(&format!("Version: {}", version));
    }
    if let Some(ref author) = show_output.author {
        output.print_info(&format!("Author: {}", author));
    }
    if let Some(ref license) = show_output.license {
        output.print_info(&format!("License: {}", license));
    }

    // Print file tree info
    if let Some(ref tree) = show_output.file_tree {
        output.print_info("");
        output.print_info("Files:");
        output.print_info(&format!(
            "  SKILL.md: {}",
            if tree.has_skill_md { "✓" } else { "✗" }
        ));
        output.print_info(&format!(
            "  scripts/: {}",
            if tree.has_scripts_dir { "✓" } else { "✗" }
        ));
        output.print_info(&format!(
            "  references/: {}",
            if tree.has_references_dir {
                "✓"
            } else {
                "✗"
            }
        ));
        output.print_info(&format!("  Total files: {}", tree.file_count));
    }

    // Print total size
    if let Some(size) = show_output.total_size_bytes {
        output.print_info(&format!("Total size: {}", format_bytes(size)));
    }

    // Print installations
    output.print_info("");
    output.print_info(&format!(
        "Installations ({}):",
        show_output.installations.len()
    ));

    if show_output.installations.is_empty() {
        output.print_warning("  No installations found");
    } else {
        for inst in &show_output.installations {
            let symlink_info = if inst.is_symlink {
                format!(" → {}", inst.symlink_target.as_deref().unwrap_or("unknown"))
            } else {
                String::new()
            };

            output.print_info(&format!(
                "  {} ({}, {}){}",
                inst.agent, inst.scope, inst.path, symlink_info
            ));
        }
    }
}

/// Formats a byte count as a human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::skill::{Agent, Installation, Scope, Skill, SkillMetadata};
    use std::path::PathBuf;

    #[test]
    fn test_format_scope() {
        assert_eq!(format_scope(Scope::Global), "global");
        assert_eq!(format_scope(Scope::Workspace), "workspace");
    }

    #[test]
    fn test_show_installation_output_serialization() {
        let output = ShowInstallationOutput {
            agent: "claude-code".to_string(),
            path: "/home/user/.claude/skills/my-skill".to_string(),
            scope: "global".to_string(),
            is_symlink: true,
            symlink_target: Some("/home/user/.sikil/repo/my-skill".to_string()),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"agent\":\"claude-code\""));
        assert!(json.contains("\"scope\":\"global\""));
        assert!(json.contains("\"is_symlink\":true"));
    }

    #[test]
    fn test_show_installation_output_without_symlink() {
        let output = ShowInstallationOutput {
            agent: "claude-code".to_string(),
            path: "/home/user/.claude/skills/my-skill".to_string(),
            scope: "global".to_string(),
            is_symlink: false,
            symlink_target: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        // symlink_target should be skipped when None
        assert!(!json.contains("symlink_target"));
    }

    #[test]
    fn test_show_file_tree_serialization() {
        let tree = ShowFileTree {
            has_skill_md: true,
            has_scripts_dir: true,
            has_references_dir: false,
            file_count: 5,
        };

        let json = serde_json::to_string(&tree).unwrap();
        assert!(json.contains("\"has_skill_md\":true"));
        assert!(json.contains("\"has_references_dir\":false"));
        assert!(json.contains("\"file_count\":5"));
    }

    #[test]
    fn test_show_output_serialization() {
        let output = ShowOutput {
            name: "my-skill".to_string(),
            directory_name: None,
            description: "A test skill".to_string(),
            version: Some("1.0.0".to_string()),
            author: Some("Test Author".to_string()),
            license: Some("MIT".to_string()),
            managed: true,
            canonical_path: Some("/home/user/.sikil/repo/my-skill".to_string()),
            installations: vec![ShowInstallationOutput {
                agent: "claude-code".to_string(),
                path: "/home/user/.claude/skills/my-skill".to_string(),
                scope: "global".to_string(),
                is_symlink: true,
                symlink_target: Some("/home/user/.sikil/repo/my-skill".to_string()),
            }],
            file_tree: Some(ShowFileTree {
                has_skill_md: true,
                has_scripts_dir: true,
                has_references_dir: false,
                file_count: 5,
            }),
            total_size_bytes: Some(1024),
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"name\":\"my-skill\""));
        assert!(json.contains("\"managed\":true"));
        assert!(json.contains("\"version\":\"1.0.0\""));
    }

    #[test]
    fn test_show_output_with_different_directory_name() {
        let output = ShowOutput {
            name: "my-skill".to_string(),
            directory_name: Some("my-skill-v2".to_string()),
            description: "A test skill".to_string(),
            version: None,
            author: None,
            license: None,
            managed: false,
            canonical_path: None,
            installations: vec![],
            file_tree: None,
            total_size_bytes: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"directory_name\":\"my-skill-v2\""));
    }

    #[test]
    fn test_show_output_skips_optional_fields_when_none() {
        let output = ShowOutput {
            name: "my-skill".to_string(),
            directory_name: None,
            description: "A test skill".to_string(),
            version: None,
            author: None,
            license: None,
            managed: false,
            canonical_path: None,
            installations: vec![],
            file_tree: None,
            total_size_bytes: None,
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(!json.contains("directory_name"));
        assert!(!json.contains("version"));
        assert!(!json.contains("author"));
        assert!(!json.contains("license"));
        assert!(!json.contains("canonical_path"));
        assert!(!json.contains("file_tree"));
        assert!(!json.contains("total_size_bytes"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_show_args_new() {
        let args = ShowArgs {
            json_mode: true,
            no_cache: false,
            name: "test-skill".to_string(),
        };

        assert!(args.json_mode);
        assert!(!args.no_cache);
        assert_eq!(args.name, "test-skill");
    }

    #[test]
    fn test_build_show_output_with_minimal_skill() {
        let skill = Skill::new(
            SkillMetadata::new("test-skill".to_string(), "A test skill".to_string()),
            "test-skill".to_string(),
        );

        let result = build_show_output(&skill, "test-skill");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.name, "test-skill");
        assert_eq!(output.description, "A test skill");
        assert!(!output.managed);
        assert!(output.directory_name.is_none());
        assert!(output.installations.is_empty());
    }

    #[test]
    fn test_build_show_output_with_installations() {
        let skill = Skill::new(
            SkillMetadata::new("multi-skill".to_string(), "A multi-agent skill".to_string()),
            "multi-skill".to_string(),
        )
        .with_installation(Installation::new(
            Agent::ClaudeCode,
            PathBuf::from("/claude/skills/multi-skill"),
            Scope::Global,
        ))
        .with_installation(Installation::new(
            Agent::Windsurf,
            PathBuf::from("/windsurf/skills/multi-skill"),
            Scope::Global,
        ));

        let result = build_show_output(&skill, "multi-skill");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.installations.len(), 2);
        assert_eq!(output.installations[0].agent, "claude-code");
        assert_eq!(output.installations[1].agent, "windsurf");
    }

    #[test]
    fn test_build_show_output_with_different_directory_name() {
        let skill = Skill::new(
            SkillMetadata::new("my-skill".to_string(), "A test skill".to_string()),
            "my-skill-v2".to_string(),
        );

        let result = build_show_output(&skill, "my-skill");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.name, "my-skill");
        assert_eq!(output.directory_name, Some("my-skill-v2".to_string()));
    }

    #[test]
    fn test_build_file_tree_with_skill_md() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path();

        // Create SKILL.md
        std::fs::write(skill_path.join("SKILL.md"), "# Test").unwrap();

        let tree = build_file_tree(skill_path);
        assert!(tree.has_skill_md);
        assert!(!tree.has_scripts_dir);
        assert!(!tree.has_references_dir);
        assert_eq!(tree.file_count, 1); // Only SKILL.md
    }

    #[test]
    fn test_build_file_tree_with_scripts_and_references() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path();

        // Create SKILL.md
        std::fs::write(skill_path.join("SKILL.md"), "# Test").unwrap();

        // Create scripts directory
        std::fs::create_dir(skill_path.join("scripts")).unwrap();
        std::fs::write(skill_path.join("scripts").join("run.sh"), "#!/bin/bash").unwrap();

        // Create references directory
        std::fs::create_dir(skill_path.join("references")).unwrap();
        std::fs::write(skill_path.join("references").join("doc.md"), "# Reference").unwrap();

        let tree = build_file_tree(skill_path);
        assert!(tree.has_skill_md);
        assert!(tree.has_scripts_dir);
        assert!(tree.has_references_dir);
        assert_eq!(tree.file_count, 3); // SKILL.md, run.sh, doc.md
    }

    #[test]
    fn test_count_files_excludes_git() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let skill_path = temp_dir.path();

        // Create regular files
        std::fs::write(skill_path.join("SKILL.md"), "# Test").unwrap();
        std::fs::write(skill_path.join("script.sh"), "#!/bin/bash").unwrap();

        // Create .git directory with files
        std::fs::create_dir_all(skill_path.join(".git/objects")).unwrap();
        std::fs::write(skill_path.join(".git/config"), "[core]").unwrap();
        std::fs::write(skill_path.join(".git/objects/data"), "git data").unwrap();

        let count = count_files(skill_path);
        // Should count SKILL.md and script.sh, but not .git contents
        assert_eq!(count, 2);
    }
}
