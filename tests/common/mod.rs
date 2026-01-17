//! Common test utilities and helpers
//!
//! This module provides reusable test helpers for setting up test environments,
//! creating temporary skill directories, and generating mock SKILL.md files.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a temporary directory for test skill storage.
///
/// Returns a `TempDir` which will be automatically cleaned up when dropped.
pub fn setup_temp_skill_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Creates a temporary skill directory structure with a given name.
///
/// # Arguments
/// * `base_dir` - The base directory to create the skill in
/// * `skill_name` - The name of the skill directory to create
///
/// # Returns
/// The path to the created skill directory
pub fn create_skill_dir(base_dir: &Path, skill_name: &str) -> PathBuf {
    let skill_path = base_dir.join(skill_name);
    fs::create_dir_all(&skill_path).expect("Failed to create skill directory");
    skill_path
}

/// Creates a mock SKILL.md file with the given content.
///
/// # Arguments
/// * `skill_dir` - The directory to create the SKILL.md file in
/// * `content` - The content to write to the SKILL.md file
pub fn create_skill_md(skill_dir: &Path, content: &str) {
    let skill_md_path = skill_dir.join("SKILL.md");
    fs::write(skill_md_path, content).expect("Failed to write SKILL.md");
}

/// Creates a mock SKILL.md file with the specified metadata fields.
///
/// # Arguments
/// * `skill_dir` - The directory to create the SKILL.md file in
/// * `name` - The skill name (required)
/// * `description` - The skill description (required)
/// * `version` - Optional version string
/// * `author` - Optional author string
/// * `license` - Optional license string
pub fn create_skill_md_with_metadata(
    skill_dir: &Path,
    name: &str,
    description: &str,
    version: Option<&str>,
    author: Option<&str>,
    license: Option<&str>,
) {
    let mut content = String::from("---\n");
    content.push_str(&format!("name: \"{}\"\n", name));
    content.push_str(&format!("description: \"{}\"\n", description));

    if let Some(v) = version {
        content.push_str(&format!("version: \"{}\"\n", v));
    }
    if let Some(a) = author {
        content.push_str(&format!("author: \"{}\"\n", a));
    }
    if let Some(l) = license {
        content.push_str(&format!("license: \"{}\"\n", l));
    }

    content.push_str("---\n");
    // Add minimal body content
    content.push_str("# Skill\n\n");
    content.push_str(&format!("{} skill description.", name));

    create_skill_md(skill_dir, &content);
}

/// Creates a minimal valid SKILL.md file with only required fields.
///
/// # Arguments
/// * `skill_dir` - The directory to create the SKILL.md file in
/// * `name` - The skill name
/// * `description` - The skill description
pub fn create_minimal_skill_md(skill_dir: &Path, name: &str, description: &str) {
    create_skill_md_with_metadata(skill_dir, name, description, None, None, None);
}

/// Creates a complete SKILL.md file with all optional fields.
///
/// # Arguments
/// * `skill_dir` - The directory to create the SKILL.md file in
/// * `name` - The skill name
/// * `description` - The skill description
pub fn create_complete_skill_md(skill_dir: &Path, name: &str, description: &str) {
    create_skill_md_with_metadata(
        skill_dir,
        name,
        description,
        Some("0.1.0"),
        Some("Test Author"),
        Some("MIT"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_temp_skill_dir() {
        let temp_dir = setup_temp_skill_dir();
        assert!(temp_dir.path().exists());
        // TempDir is cleaned up when dropped
    }

    #[test]
    fn test_create_skill_dir() {
        let temp_dir = setup_temp_skill_dir();
        let skill_path = create_skill_dir(temp_dir.path(), "test-skill");
        assert!(skill_path.exists());
        assert!(skill_path.is_dir());
        assert_eq!(skill_path.file_name().unwrap(), "test-skill");
    }

    #[test]
    fn test_create_skill_md() {
        let temp_dir = setup_temp_skill_dir();
        let skill_path = create_skill_dir(temp_dir.path(), "test-skill");
        create_skill_md(&skill_path, "# Test Skill\n\nThis is a test.");

        let skill_md = skill_path.join("SKILL.md");
        assert!(skill_md.exists());
        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains("# Test Skill"));
    }

    #[test]
    fn test_create_minimal_skill_md() {
        let temp_dir = setup_temp_skill_dir();
        let skill_path = create_skill_dir(temp_dir.path(), "minimal-skill");
        create_minimal_skill_md(&skill_path, "minimal-skill", "A minimal test skill");

        let skill_md = skill_path.join("SKILL.md");
        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains("name: \"minimal-skill\""));
        assert!(content.contains("description: \"A minimal test skill\""));
        assert!(content.contains("---"));
    }

    #[test]
    fn test_create_complete_skill_md() {
        let temp_dir = setup_temp_skill_dir();
        let skill_path = create_skill_dir(temp_dir.path(), "complete-skill");
        create_complete_skill_md(&skill_path, "complete-skill", "A complete test skill");

        let skill_md = skill_path.join("SKILL.md");
        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains("name: \"complete-skill\""));
        assert!(content.contains("description: \"A complete test skill\""));
        assert!(content.contains("version: \"0.1.0\""));
        assert!(content.contains("author: \"Test Author\""));
        assert!(content.contains("license: \"MIT\""));
    }
}
