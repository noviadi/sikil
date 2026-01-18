//! SKILL.md parser for extracting YAML frontmatter and metadata
//!
//! This module provides functions for parsing SKILL.md files, which use
//! YAML frontmatter delimited by `---` markers.

use crate::core::errors::SikilError;
use std::path::PathBuf;

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
}
