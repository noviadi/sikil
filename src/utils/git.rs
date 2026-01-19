//! Git URL parsing and validation utilities
//!
//! This module provides secure parsing of Git URLs with support for:
//! - Short form: `user/repo` → `https://github.com/user/repo.git`
//! - Short form with subdirectory: `user/repo/path/to/skill`
//! - HTTPS URLs: `https://github.com/user/repo.git`
//! - HTTPS URLs without .git suffix: `https://github.com/user/repo`
//!
//! # Security
//!
//! This parser enforces strict security rules:
//! - Only allows HTTPS URLs from GitHub.com
//! - Rejects file:// protocol (potential local file access)
//! - Rejects URLs with whitespace or NUL characters
//! - Rejects URLs starting with `-` (argument injection protection)

use crate::core::errors::SikilError;

/// A parsed Git URL with all components extracted
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedGitUrl {
    /// The full HTTPS URL to clone
    pub clone_url: String,
    /// The owner/username of the repository
    pub owner: String,
    /// The repository name
    pub repo: String,
    /// Optional subdirectory path within the repository
    pub subdirectory: Option<String>,
}

impl ParsedGitUrl {
    /// Create a new ParsedGitUrl
    fn new(clone_url: String, owner: String, repo: String, subdirectory: Option<String>) -> Self {
        Self {
            clone_url,
            owner,
            repo,
            subdirectory,
        }
    }
}

/// Parse a Git URL string into a structured ParsedGitUrl
///
/// # Supported Formats
///
/// - Short form: `user/repo` → expands to `https://github.com/user/repo.git`
/// - Short form with subdirectory: `user/repo/path/to/skill`
/// - HTTPS URL: `https://github.com/user/repo.git`
/// - HTTPS URL without .git: `https://github.com/user/repo`
///
/// # Security Checks
///
/// - Only GitHub.com URLs are allowed
/// - `file://` protocol is rejected
/// - URLs with whitespace or NUL characters are rejected
/// - URLs starting with `-` are rejected (argument injection protection)
///
/// # Errors
///
/// Returns `SikilError::InvalidGitUrl` if:
/// - The URL format is not recognized
/// - The URL uses a disallowed protocol (e.g., file://)
/// - The URL contains invalid characters
/// - The URL is not from GitHub.com
///
/// # Examples
///
/// ```
/// use sikil::utils::git::parse_git_url;
///
/// // Short form
/// let url = parse_git_url("owner/repo").unwrap();
/// assert_eq!(url.clone_url, "https://github.com/owner/repo.git");
/// assert_eq!(url.owner, "owner");
/// assert_eq!(url.repo, "repo");
/// assert!(url.subdirectory.is_none());
///
/// // Short form with subdirectory
/// let url = parse_git_url("owner/repo/skills/my-skill").unwrap();
/// assert_eq!(url.subdirectory, Some("skills/my-skill".to_string()));
///
/// // HTTPS URL
/// let url = parse_git_url("https://github.com/owner/repo.git").unwrap();
/// assert_eq!(url.clone_url, "https://github.com/owner/repo.git");
/// ```
pub fn parse_git_url(input: &str) -> Result<ParsedGitUrl, SikilError> {
    // Security check: reject URLs starting with '-' (argument injection)
    if input.starts_with('-') {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "URL cannot start with '-'".to_string(),
        });
    }

    // Security check: reject URLs with NUL characters
    if input.contains('\0') {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "URL contains NUL character".to_string(),
        });
    }

    // Security check: reject file:// protocol
    if input.to_lowercase().starts_with("file://") {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "file:// protocol is not allowed".to_string(),
        });
    }

    let trimmed = input.trim();

    // Security check: reject if trimming changed the string (has leading/trailing whitespace)
    if trimmed != input {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "URL cannot contain leading or trailing whitespace".to_string(),
        });
    }

    // Security check: reject URLs with internal whitespace
    if trimmed.chars().any(|c| c.is_whitespace()) {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "URL cannot contain whitespace".to_string(),
        });
    }

    // Try HTTPS URL format first
    if trimmed.to_lowercase().starts_with("https://") {
        return parse_https_url(trimmed);
    }

    // Try short form: owner/repo or owner/repo/path/to/skill
    parse_short_form(trimmed)
}

/// Parse an HTTPS GitHub URL
fn parse_https_url(input: &str) -> Result<ParsedGitUrl, SikilError> {
    // Only allow github.com
    if !input.to_lowercase().starts_with("https://github.com/") {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "only GitHub.com URLs are supported".to_string(),
        });
    }

    // Remove the protocol and domain
    let path = input
        .strip_prefix("https://github.com/")
        .or_else(|| input.strip_prefix("https://GITHUB.COM/"))
        .or_else(|| input.strip_prefix("https://Github.com/"))
        .ok_or_else(|| SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "invalid GitHub URL format".to_string(),
        })?;

    // Remove .git suffix if present
    let path = path.strip_suffix(".git").unwrap_or(path);

    // Split into owner, repo, and optional subdirectory
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() < 2 {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "GitHub URL must include owner/repo".to_string(),
        });
    }

    let owner = parts[0].to_string();
    let repo = parts[1].to_string();
    let subdirectory = if parts.len() > 2 {
        Some(parts[2..].join("/"))
    } else {
        None
    };

    // Validate owner and repo are not empty
    if owner.is_empty() {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "owner cannot be empty".to_string(),
        });
    }

    if repo.is_empty() {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "repository name cannot be empty".to_string(),
        });
    }

    Ok(ParsedGitUrl::new(
        input.to_string(),
        owner,
        repo,
        subdirectory,
    ))
}

/// Parse a short-form Git URL (owner/repo or owner/repo/path/to/skill)
fn parse_short_form(input: &str) -> Result<ParsedGitUrl, SikilError> {
    // Check if it looks like a short form (contains at least one /)
    if !input.contains('/') {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "invalid URL format (expected owner/repo or https://github.com/owner/repo)"
                .to_string(),
        });
    }

    let parts: Vec<&str> = input.split('/').collect();

    if parts.len() < 2 {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "short form must include owner/repo".to_string(),
        });
    }

    let owner = parts[0].to_string();
    let repo = parts[1].to_string();
    let subdirectory = if parts.len() > 2 {
        Some(parts[2..].join("/"))
    } else {
        None
    };

    // Validate owner and repo are not empty
    if owner.is_empty() {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "owner cannot be empty".to_string(),
        });
    }

    if repo.is_empty() {
        return Err(SikilError::InvalidGitUrl {
            url: input.to_string(),
            reason: "repository name cannot be empty".to_string(),
        });
    }

    // Construct the full HTTPS URL
    let clone_url = format!("https://github.com/{}/{}.git", owner, repo);

    Ok(ParsedGitUrl::new(clone_url, owner, repo, subdirectory))
}

#[cfg(test)]
mod tests {
    use super::*;

    // M3-E02-T01-S03: Parse short form: user/repo
    #[test]
    fn test_parse_short_form_basic() {
        let result = parse_git_url("owner/repo").unwrap();
        assert_eq!(result.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
        assert!(result.subdirectory.is_none());
    }

    // M3-E02-T01-S04: Parse short form with subdirectory
    #[test]
    fn test_parse_short_form_with_subdirectory() {
        let result = parse_git_url("owner/repo/skills/my-skill").unwrap();
        assert_eq!(result.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
        assert_eq!(result.subdirectory, Some("skills/my-skill".to_string()));
    }

    #[test]
    fn test_parse_short_form_with_nested_subdirectory() {
        let result = parse_git_url("owner/repo/path/to/deep/skill").unwrap();
        assert_eq!(result.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(result.subdirectory, Some("path/to/deep/skill".to_string()));
    }

    // M3-E02-T01-S05: Parse HTTPS URL
    #[test]
    fn test_parse_https_url_with_git_suffix() {
        let result = parse_git_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(result.clone_url, "https://github.com/owner/repo.git");
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
        assert!(result.subdirectory.is_none());
    }

    // M3-E02-T01-S06: Parse HTTPS URL without .git suffix
    #[test]
    fn test_parse_https_url_without_git_suffix() {
        let result = parse_git_url("https://github.com/owner/repo").unwrap();
        assert_eq!(result.clone_url, "https://github.com/owner/repo");
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
        assert!(result.subdirectory.is_none());
    }

    #[test]
    fn test_parse_https_url_with_subdirectory() {
        let result = parse_git_url("https://github.com/owner/repo.git/skills/my-skill").unwrap();
        assert_eq!(
            result.clone_url,
            "https://github.com/owner/repo.git/skills/my-skill"
        );
        assert_eq!(result.subdirectory, Some("skills/my-skill".to_string()));
    }

    // M3-E02-T01-S07: Reject file:// protocol
    #[test]
    fn test_reject_file_protocol() {
        let result = parse_git_url("file:///etc/passwd");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("file:// protocol is not allowed"));
    }

    #[test]
    fn test_reject_file_protocol_uppercase() {
        let result = parse_git_url("FILE:///etc/passwd");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("file:// protocol is not allowed"));
    }

    // M3-E02-T01-S08: Reject URLs with whitespace or NUL characters
    #[test]
    fn test_reject_whitespace_leading() {
        let result = parse_git_url(" owner/repo");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("cannot contain leading or trailing whitespace"));
    }

    #[test]
    fn test_reject_whitespace_trailing() {
        let result = parse_git_url("owner/repo ");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("cannot contain leading or trailing whitespace"));
    }

    #[test]
    fn test_reject_whitespace_internal() {
        let result = parse_git_url("owner /repo");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("cannot contain whitespace"));
    }

    #[test]
    fn test_reject_nul_character() {
        let result = parse_git_url("owner/rep\0o");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("NUL character"));
    }

    // M3-E02-T01-S09: Reject URLs starting with -
    #[test]
    fn test_reject_starts_with_dash() {
        let result = parse_git_url("-evil-flag");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("cannot start with '-'"));
    }

    #[test]
    fn test_reject_starts_with_dash_short_form() {
        let result = parse_git_url("-evil/repo");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("cannot start with '-'"));
    }

    // M3-E02-T01-S10: Reject non-GitHub URLs
    #[test]
    fn test_reject_non_github_url() {
        let result = parse_git_url("https://gitlab.com/owner/repo.git");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("only GitHub.com URLs are supported"));
    }

    #[test]
    fn test_reject_non_github_url_bitbucket() {
        let result = parse_git_url("https://bitbucket.org/owner/repo.git");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("only GitHub.com URLs are supported"));
    }

    #[test]
    fn test_reject_http_url() {
        let result = parse_git_url("http://github.com/owner/repo.git");
        assert!(result.is_err());
        // Should fall through to short form parsing and fail
    }

    #[test]
    fn test_reject_invalid_short_form() {
        let result = parse_git_url("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_empty_owner() {
        let result = parse_git_url("/repo");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("owner cannot be empty"));
    }

    #[test]
    fn test_reject_empty_repo() {
        let result = parse_git_url("owner/");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("repository name cannot be empty"));
    }

    #[test]
    fn test_github_url_case_insensitive_domain() {
        let result = parse_git_url("https://GITHUB.COM/owner/repo.git").unwrap();
        assert_eq!(result.clone_url, "https://GITHUB.COM/owner/repo.git");
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_github_url_mixed_case_domain() {
        let result = parse_git_url("https://Github.com/owner/repo.git").unwrap();
        assert_eq!(result.clone_url, "https://Github.com/owner/repo.git");
        assert_eq!(result.owner, "owner");
        assert_eq!(result.repo, "repo");
    }

    #[test]
    fn test_parsed_git_url_debug() {
        let url = ParsedGitUrl::new(
            "https://github.com/owner/repo.git".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            Some("path/to/skill".to_string()),
        );
        let debug_str = format!("{:?}", url);
        assert!(debug_str.contains("owner"));
        assert!(debug_str.contains("repo"));
        assert!(debug_str.contains("path/to/skill"));
    }

    #[test]
    fn test_parsed_git_url_clone() {
        let url1 = ParsedGitUrl::new(
            "https://github.com/owner/repo.git".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            None,
        );
        let url2 = url1.clone();
        assert_eq!(url1, url2);
    }

    #[test]
    fn test_parsed_git_url_equality() {
        let url1 = ParsedGitUrl::new(
            "https://github.com/owner/repo.git".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            Some("subdir".to_string()),
        );
        let url2 = ParsedGitUrl::new(
            "https://github.com/owner/repo.git".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            Some("subdir".to_string()),
        );
        assert_eq!(url1, url2);
    }

    #[test]
    fn test_parsed_git_url_inequality() {
        let url1 = ParsedGitUrl::new(
            "https://github.com/owner/repo.git".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            Some("subdir1".to_string()),
        );
        let url2 = ParsedGitUrl::new(
            "https://github.com/owner/repo.git".to_string(),
            "owner".to_string(),
            "repo".to_string(),
            Some("subdir2".to_string()),
        );
        assert_ne!(url1, url2);
    }
}
