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

/// Walks upward from `start`, looking for a project root marker.
///
/// Returns the first directory containing `.sikil/manifest.toml` (preferred) or
/// `.git` (file or directory, fallback). Returns `None` if no marker is found
/// before reaching the filesystem root.
///
/// Manifest wins over `.git` regardless of depth — a full upward scan for
/// manifest is performed first, and only if none is found does the function
/// fall back to scanning for `.git`.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    // First pass: look for .sikil/manifest.toml
    let mut dir = if start.is_absolute() {
        Some(start.to_path_buf())
    } else {
        std::env::current_dir()
            .ok()?
            .join(start)
            .canonicalize()
            .ok()
    };

    while let Some(d) = dir {
        if d.join(".sikil").join("manifest.toml").exists() {
            return Some(d);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }

    // Second pass: look for .git (file or directory)
    let mut dir = if start.is_absolute() {
        Some(start.to_path_buf())
    } else {
        std::env::current_dir()
            .ok()?
            .join(start)
            .canonicalize()
            .ok()
    };

    while let Some(d) = dir {
        let git = d.join(".git");
        if git.exists() {
            return Some(d);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }

    None
}

/// Convenience wrapper around `find_project_root(env::current_dir()?)`.
pub fn get_project_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    find_project_root(&cwd)
}

/// Returns `<project_root>/.sikil/manifest.toml`.
pub fn get_manifest_path(project_root: &Path) -> PathBuf {
    project_root.join(".sikil").join("manifest.toml")
}

/// Returns `<project_root>/.sikil/lock.toml`.
pub fn get_lock_path(project_root: &Path) -> PathBuf {
    project_root.join(".sikil").join("lock.toml")
}

/// Returns `<project_root>/.sikil/skills/`.
pub fn get_project_skills_path(project_root: &Path) -> PathBuf {
    project_root.join(".sikil").join("skills")
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

    fn test_ensure_dir_exists_empty_path() {
        let empty_path = Path::new("");
        // Empty path should either succeed (no-op) or fail gracefully
        // The actual behavior depends on the filesystem
        let result = ensure_dir_exists(empty_path);
        // We don't assert a specific result since behavior may vary
        // Just ensure it doesn't panic
        let _ = result;
    }

    // --- Project root discovery tests ---

    #[test]
    fn test_find_project_root_manifest_at_start() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sikil_dir = root.join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        fs::write(sikil_dir.join("manifest.toml"), "").unwrap();

        let result = find_project_root(root);
        assert_eq!(result, Some(root.to_path_buf()));
    }

    #[test]
    fn test_find_project_root_manifest_walking_up() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let sikil_dir = root.join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        fs::write(sikil_dir.join("manifest.toml"), "").unwrap();

        let deep = root.join("a").join("b").join("c");
        fs::create_dir_all(&deep).unwrap();

        let result = find_project_root(&deep);
        assert_eq!(result, Some(root.to_path_buf()));
    }

    #[test]
    fn test_find_project_root_git_directory_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let git_dir = root.join(".git");
        fs::create_dir_all(&git_dir).unwrap();

        let deep = root.join("subdir");
        fs::create_dir_all(&deep).unwrap();

        let result = find_project_root(&deep);
        assert_eq!(result, Some(root.to_path_buf()));
    }

    #[test]
    fn test_find_project_root_git_file_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // .git as a plain file (worktree marker)
        fs::write(root.join(".git"), "gitdir: /some/where").unwrap();

        let deep = root.join("src");
        fs::create_dir_all(&deep).unwrap();

        let result = find_project_root(&deep);
        assert_eq!(result, Some(root.to_path_buf()));
    }

    #[test]
    fn test_find_project_root_manifest_preferred_over_git() {
        // Manifest at deeper level, .git at shallower level — manifest wins
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // .git at root level
        fs::create_dir_all(root.join(".git")).unwrap();

        // manifest at root/a level
        let mid = root.join("a");
        let sikil_dir = mid.join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        fs::write(sikil_dir.join("manifest.toml"), "").unwrap();

        let deep = mid.join("b").join("c");
        fs::create_dir_all(&deep).unwrap();

        let result = find_project_root(&deep);
        assert_eq!(result, Some(mid.to_path_buf()));
    }

    #[test]
    fn test_find_project_root_manifest_wins_over_git_regardless_of_depth() {
        // .git at shallow level, manifest at deep level — manifest still wins
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // .git at root level
        fs::create_dir_all(root.join(".git")).unwrap();

        // manifest at root/x/y/z level
        let deep_dir = root.join("x").join("y").join("z");
        let sikil_dir = deep_dir.join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        fs::write(sikil_dir.join("manifest.toml"), "").unwrap();

        // start from inside z
        let start = deep_dir.join("sub");
        fs::create_dir_all(&start).unwrap();

        let result = find_project_root(&start);
        // manifest wins even though .git was found first (closer to root)
        assert_eq!(result, Some(deep_dir.to_path_buf()));
    }

    #[test]
    fn test_find_project_root_returns_none_when_no_markers() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let deep = root.join("a").join("b");
        fs::create_dir_all(&deep).unwrap();

        let result = find_project_root(&deep);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_manifest_path() {
        let root = Path::new("/tmp/myproject");
        assert_eq!(
            get_manifest_path(root),
            PathBuf::from("/tmp/myproject/.sikil/manifest.toml")
        );
    }

    #[test]
    fn test_get_lock_path() {
        let root = Path::new("/tmp/myproject");
        assert_eq!(
            get_lock_path(root),
            PathBuf::from("/tmp/myproject/.sikil/lock.toml")
        );
    }

    #[test]
    fn test_get_project_skills_path() {
        let root = Path::new("/tmp/myproject");
        assert_eq!(
            get_project_skills_path(root),
            PathBuf::from("/tmp/myproject/.sikil/skills")
        );
    }
}
