//! Validate command implementation
//!
//! This module provides functionality for validating Agent Skills
//! by checking their SKILL.md files and directory structure.

use crate::cli::output::Output;
use crate::core::errors::SikilError;
use crate::core::parser::{extract_frontmatter, parse_skill_md, validate_skill_name};
use anyhow::Result;
use fs_err as fs;
use std::path::{Path, PathBuf};

/// Arguments for the validate command
#[derive(Debug, Clone)]
pub struct ValidateArgs {
    /// Whether to output in JSON format
    pub json_mode: bool,
    /// Path to the skill directory or SKILL.md file to validate
    pub path: PathBuf,
}

/// Result of validating a skill
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationResult {
    /// Whether the skill passed all validations
    pub passed: bool,
    /// Path to the skill that was validated
    pub skill_path: String,
    /// Individual validation checks
    pub checks: Vec<ValidationCheck>,
    /// Warnings (non-fatal issues)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
    /// Detected directories in the skill
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_directories: Option<DetectedDirectories>,
}

/// A single validation check
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationCheck {
    /// Name of the check
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Error message if the check failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Detected directories in a skill
#[derive(Debug, Clone, serde::Serialize)]
pub struct DetectedDirectories {
    /// Whether scripts/ directory exists
    pub has_scripts: bool,
    /// Whether references/ directory exists
    pub has_references: bool,
}

/// Executes the validate command
///
/// This function:
/// 1. Checks if SKILL.md exists
/// 2. Checks if YAML frontmatter is valid
/// 3. Checks if required fields are present
/// 4. Checks name format constraints
/// 5. Checks description length (1-1024)
///
/// # Arguments
///
/// * `args` - Validate arguments including path and json_mode
///
/// # Errors
///
/// Returns an error if:
/// - The path does not exist
/// - The path is not a directory or SKILL.md file
///
/// # Examples
///
/// ```no_run
/// use sikil::commands::validate::{execute_validate, ValidateArgs};
/// use std::path::PathBuf;
///
/// let args = ValidateArgs {
///     json_mode: false,
///     path: PathBuf::from("/path/to/skill"),
/// };
/// execute_validate(args).unwrap();
/// ```
pub fn execute_validate(args: ValidateArgs) -> Result<()> {
    let output = Output::new(args.json_mode);

    // Determine the skill directory and SKILL.md path
    let (skill_dir, skill_md_path) = resolve_paths(&args.path)?;

    // Run all validations
    let mut checks: Vec<ValidationCheck> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Check 1: SKILL.md exists
    let check = check_skill_md_exists(&skill_md_path);
    checks.push(check.clone());

    if !check.passed {
        // If SKILL.md doesn't exist, we can't continue with other checks
        let result = ValidationResult {
            passed: false,
            skill_path: skill_dir.to_string_lossy().to_string(),
            checks,
            warnings: Some(warnings),
            detected_directories: None,
        };

        if args.json_mode {
            output.print_json(&result)?;
        } else {
            print_human_readable(&output, &result);
        }

        std::process::exit(1);
    }

    // Check 2: YAML frontmatter is valid
    let check = check_frontmatter_valid(&skill_md_path);
    checks.push(check.clone());

    if !check.passed {
        // If frontmatter is invalid, we can't parse metadata
        let result = ValidationResult {
            passed: false,
            skill_path: skill_dir.to_string_lossy().to_string(),
            checks,
            warnings: Some(warnings),
            detected_directories: None,
        };

        if args.json_mode {
            output.print_json(&result)?;
        } else {
            print_human_readable(&output, &result);
        }

        std::process::exit(1);
    }

    // Check 3: Parse metadata and validate required fields
    let metadata = match parse_skill_md(&skill_md_path) {
        Ok(meta) => meta,
        Err(e) => {
            // If parsing failed, add a check for required fields
            checks.push(ValidationCheck {
                name: "Required fields present".to_string(),
                passed: false,
                error: Some(e.to_string()),
            });

            let result = ValidationResult {
                passed: false,
                skill_path: skill_dir.to_string_lossy().to_string(),
                checks,
                warnings: Some(warnings),
                detected_directories: None,
            };

            if args.json_mode {
                output.print_json(&result)?;
            } else {
                print_human_readable(&output, &result);
            }

            std::process::exit(1);
        }
    };

    checks.push(ValidationCheck {
        name: "Required fields present".to_string(),
        passed: true,
        error: None,
    });

    // Check 4: Name format constraints
    let check = check_name_format(&metadata.name);
    checks.push(check);

    // Check 5: Description length (1-1024)
    let check = check_description_length(&metadata.description);
    checks.push(check);

    // Check for optional fields (warnings)
    if metadata.version.is_none() {
        warnings.push("Optional field 'version' is missing".to_string());
    }
    if metadata.author.is_none() {
        warnings.push("Optional field 'author' is missing".to_string());
    }
    if metadata.license.is_none() {
        warnings.push("Optional field 'license' is missing".to_string());
    }

    // Detect directories
    let detected_directories = detect_directories(&skill_dir);

    // Determine if all checks passed
    let passed = checks.iter().all(|c| c.passed);

    let result = ValidationResult {
        passed,
        skill_path: skill_dir.to_string_lossy().to_string(),
        checks,
        warnings: if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        },
        detected_directories: Some(detected_directories),
    };

    // Output results
    if args.json_mode {
        output.print_json(&result)?;
    } else {
        print_human_readable(&output, &result);
    }

    // Exit with appropriate code
    if passed {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Resolves the skill directory and SKILL.md path from the input path
fn resolve_paths(path: &Path) -> Result<(PathBuf, PathBuf)> {
    if !path.exists() {
        return Err(SikilError::DirectoryNotFound {
            path: path.to_path_buf(),
        }
        .into());
    }

    if path.is_file() {
        // If it's a file, it should be SKILL.md
        if path.file_name() == Some(std::ffi::OsStr::new("SKILL.md")) {
            Ok((
                path.parent().unwrap_or(path).to_path_buf(),
                path.to_path_buf(),
            ))
        } else {
            Err(SikilError::InvalidSkillMd {
                path: path.to_path_buf(),
                reason: "expected SKILL.md file or skill directory".to_string(),
            }
            .into())
        }
    } else if path.is_dir() {
        // If it's a directory, look for SKILL.md inside
        let skill_md = path.join("SKILL.md");
        Ok((path.to_path_buf(), skill_md))
    } else {
        Err(SikilError::DirectoryNotFound {
            path: path.to_path_buf(),
        }
        .into())
    }
}

/// Checks if SKILL.md exists
fn check_skill_md_exists(path: &Path) -> ValidationCheck {
    let exists = path.exists();
    ValidationCheck {
        name: "SKILL.md exists".to_string(),
        passed: exists,
        error: if exists {
            None
        } else {
            Some("SKILL.md file not found".to_string())
        },
    }
}

/// Checks if YAML frontmatter is valid
fn check_frontmatter_valid(path: &Path) -> ValidationCheck {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return ValidationCheck {
                name: "YAML frontmatter is valid".to_string(),
                passed: false,
                error: Some(format!("Failed to read file: {}", e)),
            };
        }
    };

    match extract_frontmatter(&content) {
        Ok(_) => ValidationCheck {
            name: "YAML frontmatter is valid".to_string(),
            passed: true,
            error: None,
        },
        Err(e) => ValidationCheck {
            name: "YAML frontmatter is valid".to_string(),
            passed: false,
            error: Some(e.to_string()),
        },
    }
}

/// Checks if the name format is valid
fn check_name_format(name: &str) -> ValidationCheck {
    match validate_skill_name(name) {
        Ok(_) => ValidationCheck {
            name: "Name format is valid".to_string(),
            passed: true,
            error: None,
        },
        Err(e) => ValidationCheck {
            name: "Name format is valid".to_string(),
            passed: false,
            error: Some(e.to_string()),
        },
    }
}

/// Checks if the description length is within bounds (1-1024)
fn check_description_length(description: &str) -> ValidationCheck {
    let len = description.len();
    let valid = (1..=1024).contains(&len);

    ValidationCheck {
        name: "Description length is valid (1-1024)".to_string(),
        passed: valid,
        error: if valid {
            None
        } else if len == 0 {
            Some("Description is empty".to_string())
        } else {
            Some(format!(
                "Description is too long: {} characters (max 1024)",
                len
            ))
        },
    }
}

/// Detects standard directories in the skill
fn detect_directories(skill_dir: &Path) -> DetectedDirectories {
    let scripts_dir = skill_dir.join("scripts");
    let references_dir = skill_dir.join("references");

    DetectedDirectories {
        has_scripts: scripts_dir.is_dir(),
        has_references: references_dir.is_dir(),
    }
}

/// Prints human-readable output for the validate command
fn print_human_readable(output: &Output, result: &ValidationResult) {
    output.print_info(&format!("Validating skill at: {}", result.skill_path));
    output.print_info("");

    // Print each check
    for check in &result.checks {
        if check.passed {
            output.print_success(&format!("✓ {}", check.name));
        } else {
            output.print_error(&format!("✗ {}", check.name));
            if let Some(ref error) = check.error {
                output.print_error(&format!("  {}", error));
            }
        }
    }

    // Print warnings
    if let Some(ref warnings) = result.warnings {
        if !warnings.is_empty() {
            output.print_info("");
            output.print_warning("Warnings:");
            for warning in warnings {
                output.print_warning(&format!("  - {}", warning));
            }
        }
    }

    // Print detected directories
    if let Some(ref dirs) = result.detected_directories {
        output.print_info("");
        output.print_info("Detected directories:");
        output.print_info(&format!(
            "  scripts/: {}",
            if dirs.has_scripts { "✓" } else { "✗" }
        ));
        output.print_info(&format!(
            "  references/: {}",
            if dirs.has_references { "✓" } else { "✗" }
        ));
    }

    // Print final status
    output.print_info("");
    if result.passed {
        output.print_success("PASSED");
    } else {
        output.print_error("FAILED");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_valid_skill_md(dir: &Path) {
        let content = r#"---
name: test-skill
description: A test skill for validation
version: 1.0.0
author: Test Author
license: MIT
---

# Test Skill

This is a test skill."#;
        fs::write(dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn test_resolve_paths_with_directory() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path();

        let (resolved_dir, skill_md) = resolve_paths(skill_dir).unwrap();
        assert_eq!(resolved_dir, skill_dir);
        assert_eq!(skill_md, skill_dir.join("SKILL.md"));
    }

    #[test]
    fn test_resolve_paths_with_skill_md_file() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path();
        create_valid_skill_md(skill_dir);

        let skill_md_path = skill_dir.join("SKILL.md");
        let (resolved_dir, resolved_md) = resolve_paths(&skill_md_path).unwrap();
        assert_eq!(resolved_dir, skill_dir);
        assert_eq!(resolved_md, skill_md_path);
    }

    #[test]
    fn test_resolve_paths_with_nonexistent_path() {
        let result = resolve_paths(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_skill_md_exists_when_exists() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_skill_md(temp_dir.path());

        let skill_md = temp_dir.path().join("SKILL.md");
        let check = check_skill_md_exists(&skill_md);
        assert!(check.passed);
        assert!(check.error.is_none());
    }

    #[test]
    fn test_check_skill_md_exists_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let skill_md = temp_dir.path().join("SKILL.md");
        let check = check_skill_md_exists(&skill_md);
        assert!(!check.passed);
        assert!(check.error.is_some());
    }

    #[test]
    fn test_check_frontmatter_valid_when_valid() {
        let temp_dir = TempDir::new().unwrap();
        create_valid_skill_md(temp_dir.path());

        let skill_md = temp_dir.path().join("SKILL.md");
        let check = check_frontmatter_valid(&skill_md);
        assert!(check.passed);
        assert!(check.error.is_none());
    }

    #[test]
    fn test_check_frontmatter_valid_when_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"name: test-skill
description: No frontmatter markers"#;
        fs::write(temp_dir.path().join("SKILL.md"), content).unwrap();

        let skill_md = temp_dir.path().join("SKILL.md");
        let check = check_frontmatter_valid(&skill_md);
        assert!(!check.passed);
        assert!(check.error.is_some());
    }

    #[test]
    fn test_check_name_format_when_valid() {
        let check = check_name_format("test-skill");
        assert!(check.passed);
        assert!(check.error.is_none());
    }

    #[test]
    fn test_check_name_format_when_invalid() {
        let check = check_name_format("-invalid");
        assert!(!check.passed);
        assert!(check.error.is_some());
    }

    #[test]
    fn test_check_description_length_when_valid() {
        let check = check_description_length("A valid description");
        assert!(check.passed);
        assert!(check.error.is_none());
    }

    #[test]
    fn test_check_description_length_when_empty() {
        let check = check_description_length("");
        assert!(!check.passed);
        assert!(check.error.is_some());
        assert!(check.error.unwrap().contains("empty"));
    }

    #[test]
    fn test_check_description_length_when_too_long() {
        let long_desc = "a".repeat(1025);
        let check = check_description_length(&long_desc);
        assert!(!check.passed);
        assert!(check.error.is_some());
        assert!(check.error.unwrap().contains("too long"));
    }

    #[test]
    fn test_check_description_length_when_exactly_max() {
        let desc = "a".repeat(1024);
        let check = check_description_length(&desc);
        assert!(check.passed);
    }

    #[test]
    fn test_detect_directories_with_both() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("scripts")).unwrap();
        fs::create_dir(temp_dir.path().join("references")).unwrap();

        let dirs = detect_directories(temp_dir.path());
        assert!(dirs.has_scripts);
        assert!(dirs.has_references);
    }

    #[test]
    fn test_detect_directories_with_none() {
        let temp_dir = TempDir::new().unwrap();
        let dirs = detect_directories(temp_dir.path());
        assert!(!dirs.has_scripts);
        assert!(!dirs.has_references);
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            passed: true,
            skill_path: "/path/to/skill".to_string(),
            checks: vec![ValidationCheck {
                name: "Test check".to_string(),
                passed: true,
                error: None,
            }],
            warnings: None,
            detected_directories: Some(DetectedDirectories {
                has_scripts: true,
                has_references: false,
            }),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"skill_path\":\"/path/to/skill\""));
        assert!(json.contains("\"has_scripts\":true"));
    }

    #[test]
    fn test_validation_check_serialization() {
        let check = ValidationCheck {
            name: "Test check".to_string(),
            passed: false,
            error: Some("Test error".to_string()),
        };

        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("\"name\":\"Test check\""));
        assert!(json.contains("\"passed\":false"));
        assert!(json.contains("\"error\":\"Test error\""));
    }

    #[test]
    fn test_detected_directories_serialization() {
        let dirs = DetectedDirectories {
            has_scripts: false,
            has_references: true,
        };

        let json = serde_json::to_string(&dirs).unwrap();
        assert!(json.contains("\"has_scripts\":false"));
        assert!(json.contains("\"has_references\":true"));
    }
}
