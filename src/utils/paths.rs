//! Path utility functions for Sikil
//!
//! This module provides utilities for working with file paths, including
//! shell expansion, resolving standard Sikil directories, and ensuring
//! directories exist.

use fs_err as fs;
use std::path::{Path, PathBuf};

/// Expands a path string, handling shell expansions like `~` and `$HOME`.
///
/// # Arguments
///
/// * `path` - A path string that may contain shell expansions
///
/// # Returns
///
/// A `PathBuf` with all expansions resolved
///
/// # Examples
///
/// ```
/// use sikil::utils::paths::expand_path;
/// use std::path::PathBuf;
///
/// // Expands ~ to home directory
/// let path = expand_path("~/Documents");
/// assert!(path.to_string_lossy().contains(std::env::var("HOME").unwrap().as_str()));
///
/// // Expands environment variables
/// std::env::set_var("TEST_DIR", "/tmp/test");
/// let path = expand_path("$TEST_DIR/file.txt");
/// assert_eq!(path, PathBuf::from("/tmp/test/file.txt"));
/// ```
pub fn expand_path(path: &str) -> PathBuf {
    // Use shellexpand to handle ~ and environment variables
    let expanded = shellexpand::full(path)
        .map(|s| s.into_owned())
        .unwrap_or_else(|_| path.to_string());

    PathBuf::from(expanded)
}

/// Returns the path to the Sikil repository directory.
///
/// The repository is where managed skills are stored, typically at
/// `~/.sikil/repo/`.
///
/// # Returns
///
/// A `PathBuf` pointing to the repository directory
///
/// # Examples
///
/// ```
/// use sikil::utils::paths::get_repo_path;
///
/// let repo_path = get_repo_path();
/// assert!(repo_path.ends_with(".sikil/repo"));
/// ```
pub fn get_repo_path() -> PathBuf {
    let user_dirs = directories::UserDirs::new().expect("Unable to determine home directory");
    let home = user_dirs.home_dir();
    home.join(".sikil").join("repo")
}

/// Returns the path to the Sikil configuration file.
///
/// The config file is typically at `~/.sikil/config.toml`.
///
/// # Returns
///
/// A `PathBuf` pointing to the configuration file
///
/// # Examples
///
/// ```
/// use sikil::utils::paths::get_config_path;
///
/// let config_path = get_config_path();
/// assert!(config_path.ends_with(".sikil/config.toml"));
/// ```
pub fn get_config_path() -> PathBuf {
    let user_dirs = directories::UserDirs::new().expect("Unable to determine home directory");
    let home = user_dirs.home_dir();
    home.join(".sikil").join("config.toml")
}

/// Returns the path to the Sikil cache database.
///
/// The cache database is typically at `~/.sikil/cache.json`.
///
/// # Returns
///
/// A `PathBuf` pointing to the cache database
///
/// # Examples
///
/// ```
/// use sikil::utils::paths::get_cache_path;
///
/// let cache_path = get_cache_path();
/// assert!(cache_path.ends_with(".sikil/cache.json"));
/// ```
pub fn get_cache_path() -> PathBuf {
    let user_dirs = directories::UserDirs::new().expect("Unable to determine home directory");
    let home = user_dirs.home_dir();
    home.join(".sikil").join("cache.json")
}

/// Ensures a directory exists, creating it and any parent directories if necessary.
///
/// # Arguments
///
/// * `path` - The directory path to ensure exists
///
/// # Returns
///
/// * `Ok(())` if the directory exists or was created successfully
/// * `Err(std::io::Error)` if directory creation failed
///
/// # Examples
///
/// ```
/// use sikil::utils::paths::ensure_dir_exists;
/// use std::path::Path;
///
/// let temp_dir = std::env::temp_dir().join("sikil_test_ensure_dir");
/// let result = ensure_dir_exists(&temp_dir);
/// assert!(result.is_ok());
/// assert!(temp_dir.exists());
///
/// // Clean up
/// std::fs::remove_dir_all(temp_dir).ok();
/// ```
pub fn ensure_dir_exists(path: &Path) -> Result<(), std::io::Error> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_expand_path_tilde() {
        let user_dirs = directories::UserDirs::new().expect("Unable to determine home directory");
        let home = user_dirs.home_dir();
        let path = expand_path("~/test");
        assert!(path.starts_with(home));
        assert!(path.ends_with("test"));
    }

    #[test]
    fn test_expand_path_env_var() {
        std::env::set_var("SIKIL_TEST_VAR", "/tmp/test");
        let path = expand_path("$SIKIL_TEST_VAR/file.txt");
        assert_eq!(path, PathBuf::from("/tmp/test/file.txt"));
        std::env::remove_var("SIKIL_TEST_VAR");
    }

    #[test]
    fn test_expand_path_relative() {
        let path = expand_path("relative/path");
        assert_eq!(path, PathBuf::from("relative/path"));
    }

    #[test]
    fn test_expand_path_absolute() {
        let path = expand_path("/absolute/path");
        assert_eq!(path, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_get_repo_path() {
        let repo_path = get_repo_path();
        let user_dirs = directories::UserDirs::new().expect("Unable to determine home directory");
        let home = user_dirs.home_dir();
        assert!(repo_path.starts_with(home));
        assert!(repo_path.ends_with(".sikil/repo"));
    }

    #[test]
    fn test_get_config_path() {
        let config_path = get_config_path();
        let user_dirs = directories::UserDirs::new().expect("Unable to determine home directory");
        let home = user_dirs.home_dir();
        assert!(config_path.starts_with(home));
        assert!(config_path.ends_with(".sikil/config.toml"));
    }

    #[test]
    fn test_get_cache_path() {
        let cache_path = get_cache_path();
        let user_dirs = directories::UserDirs::new().expect("Unable to determine home directory");
        let home = user_dirs.home_dir();
        assert!(cache_path.starts_with(home));
        assert!(cache_path.ends_with(".sikil/cache.json"));
    }

    #[test]
    fn test_ensure_dir_exists_creates_directory() {
        let temp_dir = std::env::temp_dir().join("sikil_test_create_dir");
        let sub_dir = temp_dir.join("nested").join("directory");

        // Ensure directory doesn't exist
        let _ = fs::remove_dir_all(&temp_dir);

        let result = ensure_dir_exists(&sub_dir);
        assert!(result.is_ok());
        assert!(sub_dir.exists());

        // Clean up
        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_ensure_dir_exists_existing_directory() {
        let temp_dir = std::env::temp_dir().join("sikil_test_existing_dir");

        // Create directory first
        fs::create_dir_all(&temp_dir).ok();

        let result = ensure_dir_exists(&temp_dir);
        assert!(result.is_ok());
        assert!(temp_dir.exists());

        // Clean up
        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_ensure_dir_exists_empty_path() {
        let empty_path = Path::new("");
        // Empty path should either succeed (no-op) or fail gracefully
        // The actual behavior depends on the filesystem
        let result = ensure_dir_exists(empty_path);
        // We don't assert a specific result since behavior may vary
        // Just ensure it doesn't panic
        let _ = result;
    }
}
