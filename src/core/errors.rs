//! Core error types for Sikil
//!
//! This module defines the error types used throughout the application,
//! using `thiserror` for consistent error display and source chaining.

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for Sikil operations
#[derive(Error, Debug)]
pub enum SikilError {
    /// SKILL.md file is invalid or malformed
    #[error("Invalid SKILL.md in {path}: {reason}")]
    InvalidSkillMd { path: PathBuf, reason: String },

    /// A skill could not be found
    #[error("Skill not found: {name}")]
    SkillNotFound { name: String },

    /// A directory does not exist
    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    /// Symlink operation failed
    #[error("Symlink error: {reason}")]
    SymlinkError {
        reason: String,
        #[source]
        source: Option<std::io::Error>,
    },

    /// Git operation failed
    #[error("Git error: {reason}")]
    GitError { reason: String },

    /// Configuration error
    #[error("Configuration error: {reason}")]
    ConfigError { reason: String },

    /// Resource already exists
    #[error("Already exists: {resource}")]
    AlreadyExists { resource: String },

    /// Permission denied
    #[error("Permission denied: {operation} on {path}")]
    PermissionDenied { operation: String, path: PathBuf },

    /// Validation error
    #[error("Validation failed: {reason}")]
    ValidationError { reason: String },

    /// Path traversal attempt detected
    #[error("Path traversal detected: {path}")]
    PathTraversal { path: String },

    /// Symlink not allowed in this context
    #[error("Symlink not allowed: {reason}")]
    SymlinkNotAllowed { reason: String },

    /// Invalid Git URL
    #[error("Invalid Git URL: {url} - {reason}")]
    InvalidGitUrl { url: String, reason: String },

    /// Configuration file exceeds maximum size
    #[error("Configuration file too large: {size} bytes (maximum 1048576 bytes)")]
    ConfigTooLarge { size: u64 },
}

/// Configuration-specific error type
#[derive(Error, Debug)]
pub enum ConfigError {
    /// File read error
    #[error("Failed to read config file: {0}")]
    FileRead(String),

    /// Invalid TOML syntax
    #[error("Invalid TOML in config: {0}")]
    InvalidToml(String),

    /// Configuration file exceeds maximum size
    #[error("Configuration file too large: {0} bytes (maximum 1048576 bytes)")]
    ConfigTooLarge(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_invalid_skill_md() {
        let err = SikilError::InvalidSkillMd {
            path: PathBuf::from("/test/skill/SKILL.md"),
            reason: "missing required field 'name'".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid SKILL.md in /test/skill/SKILL.md: missing required field 'name'"
        );
    }

    #[test]
    fn test_error_display_skill_not_found() {
        let err = SikilError::SkillNotFound {
            name: "my-skill".to_string(),
        };
        assert_eq!(err.to_string(), "Skill not found: my-skill");
    }

    #[test]
    fn test_error_display_directory_not_found() {
        let err = SikilError::DirectoryNotFound {
            path: PathBuf::from("/nonexistent/path"),
        };
        assert_eq!(err.to_string(), "Directory not found: /nonexistent/path");
    }

    #[test]
    fn test_error_display_symlink_error() {
        let err = SikilError::SymlinkError {
            reason: "target does not exist".to_string(),
            source: None,
        };
        assert_eq!(err.to_string(), "Symlink error: target does not exist");
    }

    #[test]
    fn test_error_display_git_error() {
        let err = SikilError::GitError {
            reason: "git is not installed".to_string(),
        };
        assert_eq!(err.to_string(), "Git error: git is not installed");
    }

    #[test]
    fn test_error_display_config_error() {
        let err = SikilError::ConfigError {
            reason: "invalid TOML syntax".to_string(),
        };
        assert_eq!(err.to_string(), "Configuration error: invalid TOML syntax");
    }

    #[test]
    fn test_error_display_already_exists() {
        let err = SikilError::AlreadyExists {
            resource: "skill 'my-skill'".to_string(),
        };
        assert_eq!(err.to_string(), "Already exists: skill 'my-skill'");
    }

    #[test]
    fn test_error_display_permission_denied() {
        let err = SikilError::PermissionDenied {
            operation: "write".to_string(),
            path: PathBuf::from("/protected/file"),
        };
        assert_eq!(
            err.to_string(),
            "Permission denied: write on /protected/file"
        );
    }

    #[test]
    fn test_error_display_validation_error() {
        let err = SikilError::ValidationError {
            reason: "skill name contains invalid characters".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Validation failed: skill name contains invalid characters"
        );
    }

    #[test]
    fn test_error_display_path_traversal() {
        let err = SikilError::PathTraversal {
            path: "../../../etc/passwd".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Path traversal detected: ../../../etc/passwd"
        );
    }

    #[test]
    fn test_error_display_symlink_not_allowed() {
        let err = SikilError::SymlinkNotAllowed {
            reason: "symlinks not permitted in skill source".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Symlink not allowed: symlinks not permitted in skill source"
        );
    }

    #[test]
    fn test_error_display_invalid_git_url() {
        let err = SikilError::InvalidGitUrl {
            url: "file:///etc/passwd".to_string(),
            reason: "file:// protocol is not allowed".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid Git URL: file:///etc/passwd - file:// protocol is not allowed"
        );
    }

    #[test]
    fn test_error_display_config_too_large() {
        let err = SikilError::ConfigTooLarge { size: 2_500_000 };
        assert_eq!(
            err.to_string(),
            "Configuration file too large: 2500000 bytes (maximum 1048576 bytes)"
        );
    }

    #[test]
    fn test_error_debug_format() {
        let err = SikilError::SkillNotFound {
            name: "test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("SkillNotFound"));
        assert!(debug_str.contains("test"));
    }
}
