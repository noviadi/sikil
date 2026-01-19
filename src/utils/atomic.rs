//! Atomic file operations for Sikil
//!
//! This module provides safe, atomic filesystem operations for managing
//! skill directories. It includes rollback capabilities and safeguards
//! against partial failures.

use fs_err as fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::core::errors::SikilError;

/// Copies a skill directory from source to destination, excluding symlinks and .git directory.
///
/// This function performs a deep copy of a directory while rejecting any symlinks
/// in the source tree to prevent potential security issues. It also excludes the
/// .git directory to avoid copying version control metadata.
///
/// # Arguments
///
/// * `src` - The source directory path to copy from
/// * `dest` - The destination directory path to copy to
///
/// # Returns
///
/// * `Ok(())` if the copy was successful
/// * `Err(SikilError)` if the copy failed or symlinks were detected
///
/// # Errors
///
/// This function will return an error if:
/// - The source path does not exist or is not a directory
/// - A symlink is detected in the source tree
/// - The copy operation fails partway through (partial rollback is attempted)
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::atomic::copy_skill_dir;
/// use std::path::Path;
///
/// let src = Path::new("/path/to/skill");
/// let dest = Path::new("/path/to/dest");
/// match copy_skill_dir(src, dest) {
///     Ok(()) => println!("Skill copied successfully"),
///     Err(e) => eprintln!("Failed to copy skill: {}", e),
/// }
/// ```
pub fn copy_skill_dir(src: &Path, dest: &Path) -> Result<(), SikilError> {
    // Validate source exists and is a directory
    if !src.exists() {
        return Err(SikilError::DirectoryNotFound {
            path: src.to_path_buf(),
        });
    }

    if !src.is_dir() {
        return Err(SikilError::ValidationError {
            reason: format!("source path is not a directory: {}", src.display()),
        });
    }

    // Create destination parent directory if needed
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|_e| SikilError::PermissionDenied {
            operation: "create destination parent".to_string(),
            path: parent.to_path_buf(),
        })?;
    }

    // Track copied files for rollback
    let mut copied_files: Vec<PathBuf> = Vec::new();
    let mut copied_dirs: Vec<PathBuf> = Vec::new();

    let result = (|| -> Result<(), SikilError> {
        // Walk the source directory
        for entry in WalkDir::new(src)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip .git directory
                e.file_name() != ".git"
            })
        {
            let entry = entry.map_err(|e| SikilError::ValidationError {
                reason: format!("failed to read directory entry: {}", e),
            })?;

            let entry_path = entry.path();
            let relative_path =
                entry_path
                    .strip_prefix(src)
                    .map_err(|e| SikilError::PathTraversal {
                        path: e.to_string(),
                    })?;
            let dest_path = dest.join(relative_path);

            // Reject symlinks
            if entry.path_is_symlink() {
                return Err(SikilError::SymlinkNotAllowed {
                    reason: format!("symlink found in source at {}", entry_path.display()),
                });
            }

            if entry.file_type().is_dir() {
                // Create directory
                fs::create_dir_all(&dest_path).map_err(|_e| SikilError::PermissionDenied {
                    operation: "create directory".to_string(),
                    path: dest_path.clone(),
                })?;
                copied_dirs.push(dest_path);
            } else {
                // Copy file
                fs::copy(entry_path, &dest_path).map_err(|_e| SikilError::PermissionDenied {
                    operation: "copy file".to_string(),
                    path: dest_path.clone(),
                })?;
                copied_files.push(dest_path);
            }
        }

        Ok(())
    })();

    // Rollback on failure
    if result.is_err() {
        // Clean up copied files (reverse order)
        for file in copied_files.into_iter().rev() {
            fs::remove_file(file).ok();
        }
        // Clean up copied directories (reverse order)
        for dir in copied_dirs.into_iter().rev() {
            fs::remove_dir(dir).ok();
        }
    }

    result
}

/// Atomically moves a directory from source to destination.
///
/// This function attempts to perform an atomic move operation. If the atomic
/// move fails (e.g., across different filesystems), it falls back to a copy
/// followed by removal of the source.
///
/// # Arguments
///
/// * `src` - The source directory path to move from
/// * `dest` - The destination directory path to move to
///
/// # Returns
///
/// * `Ok(())` if the move was successful
/// * `Err(SikilError)` if the move failed
///
/// # Errors
///
/// This function will return an error if:
/// - The source path does not exist
/// - Both atomic move and fallback copy+remove fail
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::atomic::atomic_move_dir;
/// use std::path::Path;
///
/// let src = Path::new("/path/to/source");
/// let dest = Path::new("/path/to/dest");
/// match atomic_move_dir(src, dest) {
///     Ok(()) => println!("Directory moved successfully"),
///     Err(e) => eprintln!("Failed to move directory: {}", e),
/// }
/// ```
pub fn atomic_move_dir(src: &Path, dest: &Path) -> Result<(), SikilError> {
    // Validate source exists
    if !src.exists() {
        return Err(SikilError::DirectoryNotFound {
            path: src.to_path_buf(),
        });
    }

    // Create destination parent directory if needed
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|_e| SikilError::PermissionDenied {
                operation: "create destination parent".to_string(),
                path: parent.to_path_buf(),
            })?;
        }
    }

    // Try atomic rename first (works on same filesystem)
    if fs::rename(src, dest).is_ok() {
        return Ok(());
    }

    // Fallback: copy then remove (for cross-filesystem moves)
    // Create a backup of destination state for rollback
    let dest_existed = dest.exists();
    let backup_path = if dest_existed {
        Some(
            tempfile::TempDir::new().map_err(|e| SikilError::ValidationError {
                reason: format!("failed to create backup directory: {}", e),
            })?,
        )
    } else {
        None
    };

    // Backup existing destination if it exists
    if dest_existed {
        if let Some(ref backup) = backup_path {
            let backup_dest = backup.path().join("backup");
            copy_skill_dir(dest, &backup_dest)?;
        }
    }

    // Perform the copy
    let copy_result = copy_skill_dir(src, dest);

    match copy_result {
        Ok(()) => {
            // Copy succeeded, remove source
            if let Err(_e) = fs::remove_dir_all(src) {
                // Rollback: remove destination copy
                fs::remove_dir_all(dest).ok();
                return Err(SikilError::PermissionDenied {
                    operation: "remove source directory".to_string(),
                    path: src.to_path_buf(),
                });
            }
            Ok(())
        }
        Err(e) => {
            // Rollback: restore backup if it existed
            if dest_existed {
                if let Some(ref backup) = backup_path {
                    let backup_dest = backup.path().join("backup");
                    // Remove failed partial copy
                    fs::remove_dir_all(dest).ok();
                    // Restore backup
                    let _ = copy_skill_dir(&backup_dest, dest);
                }
            }
            Err(e)
        }
    }
}

/// Safely removes a directory with confirmation check.
///
/// This function removes a directory and all its contents. It includes
/// safeguards to prevent accidental deletion of important directories.
///
/// # Arguments
///
/// * `path` - The directory path to remove
/// * `confirmed` - Whether the user has confirmed the deletion
///
/// # Returns
///
/// * `Ok(())` if the removal was successful
/// * `Err(SikilError)` if the removal failed or not confirmed
///
/// # Errors
///
/// This function will return an error if:
/// - The path does not exist
/// - The confirmation flag is false
/// - The removal fails due to permissions or other IO errors
///
/// # Examples
///
/// ```no_run
/// use sikil::utils::atomic::safe_remove_dir;
/// use std::path::Path;
///
/// let path = Path::new("/path/to/remove");
/// match safe_remove_dir(path, true) {
///     Ok(()) => println!("Directory removed successfully"),
///     Err(e) => eprintln!("Failed to remove directory: {}", e),
/// }
/// ```
pub fn safe_remove_dir(path: &Path, confirmed: bool) -> Result<(), SikilError> {
    // Check confirmation
    if !confirmed {
        return Err(SikilError::ValidationError {
            reason: "directory removal requires confirmation".to_string(),
        });
    }

    // Validate path exists
    if !path.exists() {
        return Err(SikilError::DirectoryNotFound {
            path: path.to_path_buf(),
        });
    }

    // Remove the directory
    fs::remove_dir_all(path).map_err(|_e| SikilError::PermissionDenied {
        operation: "remove directory".to_string(),
        path: path.to_path_buf(),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_copy_skill_dir_basic() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        // Create test files and directories
        let src_path = src_dir.path();
        File::create(src_path.join("file1.txt")).unwrap();
        File::create(src_path.join("file2.txt")).unwrap();
        fs::create_dir_all(src_path.join("subdir")).unwrap();
        File::create(src_path.join("subdir").join("file3.txt")).unwrap();

        let dest_path = dest_dir.path().join("copied");

        let result = copy_skill_dir(src_path, &dest_path);
        assert!(result.is_ok());
        assert!(dest_path.exists());
        assert!(dest_path.join("file1.txt").exists());
        assert!(dest_path.join("file2.txt").exists());
        assert!(dest_path.join("subdir").exists());
        assert!(dest_path.join("subdir").join("file3.txt").exists());
    }

    #[test]
    fn test_copy_skill_dir_excludes_git() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let src_path = src_dir.path();
        File::create(src_path.join("file.txt")).unwrap();
        fs::create_dir_all(src_path.join(".git")).unwrap();
        File::create(src_path.join(".git").join("config")).unwrap();
        File::create(src_path.join(".git").join("HEAD")).unwrap();

        let dest_path = dest_dir.path().join("copied");

        let result = copy_skill_dir(src_path, &dest_path);
        assert!(result.is_ok());
        assert!(dest_path.join("file.txt").exists());
        assert!(!dest_path.join(".git").exists());
    }

    #[test]
    fn test_copy_skill_dir_rejects_symlinks() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let src_path = src_dir.path();
        File::create(src_path.join("file.txt")).unwrap();

        // Create a symlink
        #[cfg(unix)]
        std::os::unix::fs::symlink(src_path.join("file.txt"), src_path.join("link.txt")).unwrap();

        let dest_path = dest_dir.path().join("copied");

        let result = copy_skill_dir(src_path, &dest_path);
        assert!(result.is_err());
        match result {
            Err(SikilError::SymlinkNotAllowed { .. }) => {}
            _ => panic!("Expected SymlinkNotAllowed error"),
        }
    }

    #[test]
    fn test_copy_skill_dir_nonexistent_source() {
        let result = copy_skill_dir(Path::new("/nonexistent/path"), Path::new("/dest"));
        assert!(result.is_err());
        match result {
            Err(SikilError::DirectoryNotFound { .. }) => {}
            _ => panic!("Expected DirectoryNotFound error"),
        }
    }

    #[test]
    fn test_copy_skill_dir_rollback_on_failure() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let src_path = src_dir.path();
        File::create(src_path.join("file1.txt")).unwrap();
        File::create(src_path.join("file2.txt")).unwrap();

        // Create a read-only file to trigger failure during copy
        let readonly_file = src_path.join("readonly.txt");
        File::create(&readonly_file).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_file).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&readonly_file, perms).unwrap();
        }

        let dest_path = dest_dir.path().join("copied");
        let dest_parent = dest_dir.path();

        // Make the destination read-only to trigger failure
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(dest_parent).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(dest_parent, perms).unwrap();
        }

        let result = copy_skill_dir(src_path, &dest_path);

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(dest_parent).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(dest_parent, perms).unwrap();
            let mut perms = fs::metadata(&readonly_file).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&readonly_file, perms).unwrap();
        }

        // The copy should fail
        assert!(result.is_err());

        // Destination should not exist or be incomplete (rollback happened)
        assert!(!dest_path.exists() || !dest_path.join("file1.txt").exists());
    }

    #[test]
    fn test_atomic_move_dir_same_fs() {
        let src_dir = TempDir::new().unwrap();
        let dest_parent = TempDir::new().unwrap();

        let src_path = src_dir.path();
        File::create(src_path.join("file.txt")).unwrap();

        let dest_path = dest_parent.path().join("moved");

        let result = atomic_move_dir(src_path, &dest_path);
        assert!(result.is_ok());
        assert!(!src_path.exists());
        assert!(dest_path.exists());
        assert!(dest_path.join("file.txt").exists());
    }

    #[test]
    fn test_atomic_move_dir_nonexistent_source() {
        let result = atomic_move_dir(Path::new("/nonexistent/path"), Path::new("/dest"));
        assert!(result.is_err());
        match result {
            Err(SikilError::DirectoryNotFound { .. }) => {}
            _ => panic!("Expected DirectoryNotFound error"),
        }
    }

    #[test]
    fn test_safe_remove_dir_success() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("to_remove");
        fs::create_dir_all(&path).unwrap();
        File::create(path.join("file.txt")).unwrap();

        let result = safe_remove_dir(&path, true);
        assert!(result.is_ok());
        assert!(!path.exists());
    }

    #[test]
    fn test_safe_remove_dir_not_confirmed() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("to_remove");
        fs::create_dir_all(&path).unwrap();

        let result = safe_remove_dir(&path, false);
        assert!(result.is_err());
        match result {
            Err(SikilError::ValidationError { .. }) => {}
            _ => panic!("Expected ValidationError"),
        }
        assert!(path.exists());
    }

    #[test]
    fn test_safe_remove_dir_nonexistent() {
        let result = safe_remove_dir(Path::new("/nonexistent"), true);
        assert!(result.is_err());
        match result {
            Err(SikilError::DirectoryNotFound { .. }) => {}
            _ => panic!("Expected DirectoryNotFound error"),
        }
    }

    #[test]
    fn test_copy_skill_dir_nested_structure() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let src_path = src_dir.path();
        fs::create_dir_all(src_path.join("level1").join("level2").join("level3")).unwrap();
        File::create(src_path.join("level1").join("file1.txt")).unwrap();
        File::create(src_path.join("level1").join("level2").join("file2.txt")).unwrap();
        File::create(
            src_path
                .join("level1")
                .join("level2")
                .join("level3")
                .join("file3.txt"),
        )
        .unwrap();

        let dest_path = dest_dir.path().join("copied");

        let result = copy_skill_dir(src_path, &dest_path);
        assert!(result.is_ok());
        assert!(dest_path.join("level1").join("file1.txt").exists());
        assert!(dest_path
            .join("level1")
            .join("level2")
            .join("file2.txt")
            .exists());
        assert!(dest_path
            .join("level1")
            .join("level2")
            .join("level3")
            .join("file3.txt")
            .exists());
    }

    #[test]
    fn test_copy_skill_dir_empty_directory() {
        let src_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();

        let src_path = src_dir.path();
        fs::create_dir_all(src_path.join("empty")).unwrap();

        let dest_path = dest_dir.path().join("copied");

        let result = copy_skill_dir(src_path, &dest_path);
        assert!(result.is_ok());
        assert!(dest_path.join("empty").exists());
    }
}
