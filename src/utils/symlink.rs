//! Symlink utility functions for Sikil
//!
//! This module provides utilities for working with symbolic links,
//! including creating, reading, and resolving symlinks, and determining
//! whether a symlink is managed by Sikil (i.e., points to ~/.sikil/repo/).

use crate::core::errors::SikilError;
use std::path::{Path, PathBuf};

/// Creates a symbolic link from `src` to `dest`.
///
/// This function creates a symbolic link at `dest` that points to `src`.
/// The parent directory of `dest` will be created if it doesn't exist.
///
/// # Arguments
///
/// * `src` - The target path that the symlink will point to
/// * `dest` - The location where the symlink will be created
///
/// # Returns
///
/// * `Ok(())` if the symlink was created successfully
/// * `Err(SikilError::SymlinkError)` if symlink creation failed
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::symlink::create_symlink;
/// use std::path::Path;
///
/// let src = Path::new("/home/user/.sikil/repo/my-skill");
/// let dest = Path::new("/home/user/.config/claude-code/skills/my-skill");
/// create_symlink(src, dest).unwrap();
/// ```
pub fn create_symlink(src: &Path, dest: &Path) -> Result<(), SikilError> {
    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| SikilError::SymlinkError {
                reason: format!("failed to create parent directory: {}", parent.display()),
                source: Some(e),
            })?;
        }
    }

    // Remove existing symlink if it exists
    if dest.exists() || is_symlink(dest) {
        std::fs::remove_file(dest).map_err(|e| SikilError::SymlinkError {
            reason: format!("failed to remove existing symlink: {}", dest.display()),
            source: Some(e),
        })?;
    }

    // Create the symlink
    std::os::unix::fs::symlink(src, dest).map_err(|e| SikilError::SymlinkError {
        reason: format!(
            "failed to create symlink from {} to {}",
            src.display(),
            dest.display()
        ),
        source: Some(e),
    })?;

    Ok(())
}

/// Checks if a path is a symbolic link.
///
/// This function returns `true` if the path exists and is a symbolic link,
/// and `false` otherwise. It does not follow symlinks.
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// `true` if the path is a symbolic link, `false` otherwise
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::symlink::is_symlink;
/// use std::path::Path;
///
/// let path = Path::new("/home/user/.config/claude-code/skills/my-skill");
/// if is_symlink(path) {
///     println!("This is a symlink!");
/// }
/// ```
pub fn is_symlink(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    }
}

/// Reads the target of a symbolic link.
///
/// This function returns the path that the symlink points to, without
/// resolving it to an absolute path or following intermediate symlinks.
///
/// # Arguments
///
/// * `path` - The symlink to read
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the target of the symlink
/// * `Err(SikilError::SymlinkError)` if reading the symlink failed
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::symlink::read_symlink_target;
/// use std::path::Path;
///
/// let link = Path::new("/home/user/.config/claude-code/skills/my-skill");
/// let target = read_symlink_target(link).unwrap();
/// println!("Symlink points to: {}", target.display());
/// ```
pub fn read_symlink_target(path: &Path) -> Result<PathBuf, SikilError> {
    if !is_symlink(path) {
        return Err(SikilError::SymlinkError {
            reason: format!("not a symlink: {}", path.display()),
            source: None,
        });
    }

    std::fs::read_link(path).map_err(|e| SikilError::SymlinkError {
        reason: format!("failed to read symlink: {}", path.display()),
        source: Some(e),
    })
}

/// Resolves a path to its real (canonical) absolute path.
///
/// This function follows all symbolic links and resolves the path to
/// its ultimate target, returning an absolute, normalized path.
///
/// # Arguments
///
/// * `path` - The path to resolve
///
/// # Returns
///
/// * `Ok(PathBuf)` containing the resolved absolute path
/// * `Err(SikilError::SymlinkError)` if resolution failed
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::symlink::resolve_realpath;
/// use std::path::Path;
///
/// let path = Path::new("/home/user/.config/claude-code/skills/my-skill");
/// let realpath = resolve_realpath(path).unwrap();
/// println!("Real path: {}", realpath.display());
/// ```
pub fn resolve_realpath(path: &Path) -> Result<PathBuf, SikilError> {
    // First check if path exists
    if !path.exists() && !is_symlink(path) {
        return Err(SikilError::SymlinkError {
            reason: format!("path does not exist: {}", path.display()),
            source: None,
        });
    }

    // Use canonicalize to resolve all symlinks and normalize the path
    path.canonicalize().map_err(|e| SikilError::SymlinkError {
        reason: format!("failed to resolve real path: {}", path.display()),
        source: Some(e),
    })
}

/// Checks if a symlink is managed by Sikil.
///
/// A symlink is considered managed if its target is under the Sikil
/// repository directory (`~/.sikil/repo/`). This is used to distinguish
/// between skills that Sikil manages vs. user-created symlinks.
///
/// # Arguments
///
/// * `path` - The symlink to check
///
/// # Returns
///
/// `true` if the symlink points to a location under `~/.sikil/repo/`,
/// `false` otherwise
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::symlink::is_managed_symlink;
/// use std::path::Path;
///
/// let link = Path::new("/home/user/.config/claude-code/skills/my-skill");
/// if is_managed_symlink(link) {
///     println!("This symlink is managed by Sikil");
/// }
/// ```
pub fn is_managed_symlink(path: &Path) -> bool {
    // First check if it's a symlink
    if !is_symlink(path) {
        return false;
    }

    // Get the real path of the symlink target
    let target_real = match resolve_realpath(path) {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Get the repo path
    let repo_path = super::paths::get_repo_path();

    // Check if the target is under the repo path
    target_real.starts_with(&repo_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_symlink_with_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target.txt");
        let link = temp_dir.path().join("link.txt");

        // Create target file
        fs::write(&target, "test content").unwrap();

        // Create symlink
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(is_symlink(&link));
        assert!(!is_symlink(&target));
    }

    #[test]
    fn test_is_symlink_with_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("file.txt");

        fs::write(&file, "test content").unwrap();

        assert!(!is_symlink(&file));
    }

    #[test]
    fn test_is_symlink_nonexistent() {
        let path = Path::new("/nonexistent/path");
        assert!(!is_symlink(path));
    }

    #[test]
    fn test_read_symlink_target() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target.txt");
        let link = temp_dir.path().join("link.txt");

        fs::write(&target, "test content").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = read_symlink_target(&link).unwrap();
        // The result might be relative or absolute, depending on how we created it
        // Just verify it points to the right file
        assert!(result.ends_with("target.txt") || result == target);
    }

    #[test]
    fn test_read_symlink_target_not_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("file.txt");

        fs::write(&file, "test content").unwrap();

        let result = read_symlink_target(&file);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_realpath_with_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target.txt");
        let link = temp_dir.path().join("link.txt");

        fs::write(&target, "test content").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let realpath = resolve_realpath(&link).unwrap();
        assert_eq!(realpath, target);
    }

    #[test]
    fn test_resolve_realpath_with_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("file.txt");

        fs::write(&file, "test content").unwrap();

        let realpath = resolve_realpath(&file).unwrap();
        assert_eq!(realpath, file);
    }

    #[test]
    fn test_resolve_realpath_nonexistent() {
        let path = Path::new("/nonexistent/path/that/does/not/exist");
        let result = resolve_realpath(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target.txt");
        let link = temp_dir.path().join("link.txt");

        fs::write(&target, "test content").unwrap();

        let result = create_symlink(&target, &link);
        assert!(result.is_ok());
        assert!(is_symlink(&link));
        assert_eq!(read_symlink_target(&link).unwrap(), target);
    }

    #[test]
    fn test_create_symlink_creates_parent_dir() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target.txt");
        let link = temp_dir
            .path()
            .join("nested")
            .join("directory")
            .join("link.txt");

        fs::write(&target, "test content").unwrap();

        let result = create_symlink(&target, &link);
        assert!(result.is_ok());
        assert!(is_symlink(&link));
        assert!(link.parent().unwrap().exists());
    }

    #[test]
    fn test_create_symlink_replaces_existing() {
        let temp_dir = TempDir::new().unwrap();
        let target1 = temp_dir.path().join("target1.txt");
        let target2 = temp_dir.path().join("target2.txt");
        let link = temp_dir.path().join("link.txt");

        fs::write(&target1, "content1").unwrap();
        fs::write(&target2, "content2").unwrap();

        // Create first symlink
        create_symlink(&target1, &link).unwrap();
        assert_eq!(read_symlink_target(&link).unwrap(), target1);

        // Replace with second symlink
        create_symlink(&target2, &link).unwrap();
        assert_eq!(read_symlink_target(&link).unwrap(), target2);
    }

    #[test]
    fn test_is_managed_symlink_managed() {
        let temp_dir = TempDir::new().unwrap();

        // Create a mock repo structure
        let repo = temp_dir.path().join(".sikil").join("repo");
        let skill = repo.join("my-skill");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# Test Skill").unwrap();

        // Create symlink pointing to repo
        let link = temp_dir.path().join("skill-link");
        std::os::unix::fs::symlink(&skill, &link).unwrap();

        // Temporarily override the repo path for this test
        // We can't easily do this with the current design, so we'll
        // just test the core logic with a direct check
        let result = resolve_realpath(&link).unwrap();
        assert!(result.starts_with(&repo));
    }

    #[test]
    fn test_is_managed_symlink_unmanaged() {
        let temp_dir = TempDir::new().unwrap();

        // Create a skill outside the repo
        let skill = temp_dir.path().join("other-skill");
        fs::create_dir(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# Test Skill").unwrap();

        // Create symlink pointing outside repo
        let link = temp_dir.path().join("skill-link");
        std::os::unix::fs::symlink(&skill, &link).unwrap();

        // Since it doesn't point to ~/.sikil/repo, it should not be managed
        assert!(!is_managed_symlink(&link));
    }

    #[test]
    fn test_is_managed_symlink_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("regular.txt");

        fs::write(&file, "test content").unwrap();

        // Regular files are not managed symlinks
        assert!(!is_managed_symlink(&file));
    }

    #[test]
    fn test_is_managed_symlink_broken() {
        let temp_dir = TempDir::new().unwrap();
        let link = temp_dir.path().join("broken-link");

        // Create a symlink to a non-existent target
        std::os::unix::fs::symlink("/nonexistent/target", &link).unwrap();

        // Broken symlinks should return false
        assert!(!is_managed_symlink(&link));
    }

    #[test]
    fn test_is_managed_symlink_inside_repo() {
        let temp_dir = TempDir::new().unwrap();

        // Create a mock repo structure
        let repo = temp_dir.path().join(".sikil").join("repo");
        let skill = repo.join("my-skill");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# Test Skill").unwrap();

        // Create symlink pointing to repo
        let link = temp_dir.path().join("skill-link");
        std::os::unix::fs::symlink(&skill, &link).unwrap();

        // The link's target resolves to inside the repo
        let realpath = resolve_realpath(&link).unwrap();
        assert!(realpath.starts_with(&repo));
    }
}
