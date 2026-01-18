//! SKILL.md parser for extracting YAML frontmatter and metadata
//!
//! This module provides functions for parsing SKILL.md files, which use
//! YAML frontmatter delimited by `---` markers.

use crate::core::errors::SikilError;
use crate::core::skill::SkillMetadata;
use fs_err as fs;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Extracts YAML frontmatter from SKILL.md content.
///
/// SKILL.md files use YAML frontmatter delimited by `---` markers:
///
/// ```text
/// ---
/// name: my-skill
/// description: A useful skill
/// ---
///
/// # My Skill
///
/// Documentation here...
/// ```
///
/// # Arguments
///
/// * `content` - The full content of a SKILL.md file
///
/// # Returns
///
/// * `Ok(&str)` - The YAML frontmatter content (without the `---` markers)
/// * `Err(SikilError)` - If the frontmatter is missing or malformed
///
/// # Examples
///
/// ```
/// use sikil::core::parser::extract_frontmatter;
/// use sikil::core::errors::SikilError;
///
/// let content = r#"
/// ---
/// name: test-skill
/// description: A test skill
/// ---
///
/// # Documentation
/// "#;
///
/// let frontmatter = extract_frontmatter(content)?;
/// assert!(frontmatter.contains("name: test-skill"));
/// # Ok::<(), SikilError>(())
/// ```
///
/// # Errors
///
/// * Returns `InvalidSkillMd` if no `---` markers are found
/// * Returns `InvalidSkillMd` if only one `---` marker is found (malformed)
pub fn extract_frontmatter(content: &str) -> Result<&str, SikilError> {
    // Find the first occurrence of '---'
    let first_marker = content
        .find("---")
        .ok_or_else(|| SikilError::InvalidSkillMd {
            path: PathBuf::from("<content>"),
            reason: "missing frontmatter delimiters (no '---' markers found)".to_string(),
        })?;

    // Skip past the first '---' marker
    let after_first = &content[first_marker + 3..];

    // Find the second occurrence of '---' after the first one
    let second_marker = after_first
        .find("---")
        .ok_or_else(|| SikilError::InvalidSkillMd {
            path: PathBuf::from("<content>"),
            reason: "malformed frontmatter (only one '---' marker found, expected two)".to_string(),
        })?;

    // Extract the content between the markers
    let frontmatter = after_first[..second_marker].trim();

    // Ensure the first marker is at the start of the file (allowing only leading whitespace)
    let before_first = &content[..first_marker];
    if !before_first.trim().is_empty() {
        return Err(SikilError::InvalidSkillMd {
            path: PathBuf::from("<content>"),
            reason: format!(
                "frontmatter must be at the start of the file (found '{}' before first '---')",
                before_first.trim()
            ),
        });
    }

    Ok(frontmatter)
}

/// Raw YAML structure for parsing SKILL.md frontmatter
///
/// This struct uses `Option` for all fields to allow graceful handling
/// of missing fields, which we validate after parsing.
#[derive(Debug, Deserialize)]
struct RawSkillMetadata {
    /// Primary identifier (required)
    name: Option<String>,

    /// Human-readable description (required)
    description: Option<String>,

    /// Optional version string
    version: Option<String>,

    /// Optional author
    author: Option<String>,

    /// Optional license
    license: Option<String>,
}

/// Parses a SKILL.md file and extracts its metadata.
///
/// This function reads a SKILL.md file, extracts the YAML frontmatter,
/// and parses it into a `SkillMetadata` struct.
///
/// # Arguments
///
/// * `path` - Path to the SKILL.md file
///
/// # Returns
///
/// * `Ok(SkillMetadata)` - The parsed metadata
/// * `Err(SikilError)` - If the file cannot be read, parsed, or validated
///
/// # Examples
///
/// ```no_run
/// use sikil::core::parser::parse_skill_md;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = parse_skill_md(Path::new("/path/to/SKILL.md"))?;
/// println!("Skill: {}", metadata.name);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// * Returns `InvalidSkillMd` if the file cannot be read
/// * Returns `InvalidSkillMd` if the frontmatter is malformed
/// * Returns `InvalidSkillMd` if required fields (name, description) are missing
pub fn parse_skill_md(path: &Path) -> Result<SkillMetadata, SikilError> {
    // Read the file content
    let content = fs::read_to_string(path).map_err(|e| SikilError::InvalidSkillMd {
        path: path.to_path_buf(),
        reason: format!("failed to read file: {}", e),
    })?;

    // Extract the YAML frontmatter
    let frontmatter = extract_frontmatter(&content).map_err(|e| match e {
        SikilError::InvalidSkillMd { path: _, reason } => SikilError::InvalidSkillMd {
            path: path.to_path_buf(),
            reason,
        },
        _ => e,
    })?;

    // Parse the YAML into our raw struct
    let raw: RawSkillMetadata =
        serde_yaml::from_str(frontmatter).map_err(|e| SikilError::InvalidSkillMd {
            path: path.to_path_buf(),
            reason: format!("failed to parse YAML: {}", e),
        })?;

    // Validate required fields
    let name = raw.name.ok_or_else(|| SikilError::InvalidSkillMd {
        path: path.to_path_buf(),
        reason: "missing required field 'name'".to_string(),
    })?;

    let description = raw.description.ok_or_else(|| SikilError::InvalidSkillMd {
        path: path.to_path_buf(),
        reason: "missing required field 'description'".to_string(),
    })?;

    // Build and return the SkillMetadata
    Ok(SkillMetadata {
        name,
        description,
        version: raw.version,
        author: raw.author,
        license: raw.license,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter_valid() {
        let content = r#"---
name: test-skill
description: A test skill
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert_eq!(frontmatter, "name: test-skill\ndescription: A test skill");
    }

    #[test]
    fn test_extract_frontmatter_valid_with_leading_newline() {
        let content = r#"
---
name: test-skill
description: A test skill
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert!(frontmatter.contains("name: test-skill"));
    }

    #[test]
    fn test_extract_frontmatter_valid_multiline() {
        let content = r#"---
name: test-skill
description: A test skill
version: 1.0.0
author: Test Author
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert!(frontmatter.contains("name: test-skill"));
        assert!(frontmatter.contains("version: 1.0.0"));
    }

    #[test]
    fn test_extract_frontmatter_missing_markers() {
        let content = r#"name: test-skill
description: A test skill

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing frontmatter delimiters"));
    }

    #[test]
    fn test_extract_frontmatter_single_marker() {
        let content = r#"---
name: test-skill
description: A test skill

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("only one '---' marker found"));
    }

    #[test]
    fn test_extract_frontmatter_not_at_start() {
        let content = r#"# Some documentation

---
name: test-skill
description: A test skill
---

# More documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("frontmatter must be at the start"));
    }

    #[test]
    fn test_extract_frontmatter_empty_frontmatter() {
        let content = r#"---
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert!(frontmatter.is_empty());
    }

    #[test]
    fn test_extract_frontmatter_with_empty_lines_in_frontmatter() {
        let content = r#"---
name: test-skill

description: A test skill
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert!(frontmatter.contains("name: test-skill"));
    }

    #[test]
    fn test_extract_frontmatter_preserves_internal_spacing() {
        let content = r#"---
name: test-skill
description: |
  A multi-line
  description
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert!(frontmatter.contains("name: test-skill"));
        assert!(frontmatter.contains("|"));
    }

    #[test]
    fn test_extract_frontmatter_empty_content() {
        let content = "";

        let result = extract_frontmatter(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing frontmatter delimiters"));
    }

    #[test]
    fn test_extract_frontmatter_only_whitespace() {
        let content = "   \n  \n  ";

        let result = extract_frontmatter(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing frontmatter delimiters"));
    }

    #[test]
    fn test_extract_frontmatter_three_markers() {
        let content = r#"---
name: test-skill
---
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        // Should extract between first and second marker
        let frontmatter = result.unwrap();
        assert_eq!(frontmatter, "name: test-skill");
    }

    #[test]
    fn test_extract_frontmatter_marker_with_spaces() {
        let content = r#"---
name: test-skill
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert!(frontmatter.contains("name: test-skill"));
    }

    #[test]
    fn test_extract_frontmatter_complex_yaml() {
        let content = r#"---
name: complex-skill
description: A skill with complex YAML
metadata:
  key1: value1
  key2: value2
tags:
  - tag1
  - tag2
---

# Documentation"#;

        let result = extract_frontmatter(content);
        assert!(result.is_ok());
        let frontmatter = result.unwrap();
        assert!(frontmatter.contains("name: complex-skill"));
        assert!(frontmatter.contains("metadata:"));
        assert!(frontmatter.contains("tags:"));
    }

    // Tests for parse_skill_md
    use std::fs;

    #[test]
    fn test_parse_skill_md_valid_minimal() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"---
name: test-skill
description: A test skill
---

# Documentation"#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "test-skill");
        assert_eq!(metadata.description, "A test skill");
        assert!(metadata.version.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.license.is_none());
    }

    #[test]
    fn test_parse_skill_md_valid_with_all_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"---
name: complete-skill
description: A skill with all fields
version: 1.0.0
author: Test Author
license: MIT
---

# Documentation"#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "complete-skill");
        assert_eq!(metadata.description, "A skill with all fields");
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.license, Some("MIT".to_string()));
    }

    #[test]
    fn test_parse_skill_md_missing_name() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"---
description: A test skill
---

# Documentation"#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing required field 'name'"));
    }

    #[test]
    fn test_parse_skill_md_missing_description() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"---
name: test-skill
---

# Documentation"#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("missing required field 'description'"));
    }

    #[test]
    fn test_parse_skill_md_missing_both_required_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"---
version: 1.0.0
---

# Documentation"#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should report missing name first (checked first)
        assert!(err.to_string().contains("missing required field 'name'"));
    }

    #[test]
    fn test_parse_skill_md_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("nonexistent.md");

        let result = parse_skill_md(&skill_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("failed to read file"));
    }

    #[test]
    fn test_parse_skill_md_invalid_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"---
name: test-skill
description: [invalid
---

# Documentation"#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("failed to parse YAML"));
    }

    #[test]
    fn test_parse_skill_md_no_frontmatter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"# Just Markdown

No frontmatter here."#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing frontmatter delimiters"));
    }

    #[test]
    fn test_parse_skill_md_multiline_description() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_path = temp_dir.path().join("SKILL.md");
        let content = r#"---
name: test-skill
description: |
  This is a
  multi-line
  description
---

# Documentation"#;
        fs::write(&skill_path, content).unwrap();

        let result = parse_skill_md(&skill_path);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "test-skill");
        assert!(metadata.description.contains("multi-line"));
    }
}
