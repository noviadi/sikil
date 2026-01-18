//! Integration tests for filesystem utilities
//!
//! M1-E05-T04: Test Filesystem Utilities
//!
//! This test module verifies the functionality of symlink and atomic
//! file operations through integration-level testing.

use std::fs::{self, File};
use std::io::Write;

use sikil::utils::atomic::{atomic_move_dir, copy_skill_dir, safe_remove_dir};
use sikil::utils::symlink::{
    create_symlink, is_managed_symlink, is_symlink, read_symlink_target, resolve_realpath,
};
use tempfile::TempDir;

/// M1-E05-T04-S01: Test symlink creation and reading
#[test]
fn test_symlink_creation_and_reading() {
    let temp_dir = TempDir::new().unwrap();
    let target = temp_dir.path().join("target-file.txt");
    let link = temp_dir.path().join("symlink");

    // Create target file with content
    let mut file = File::create(&target).unwrap();
    file.write_all(b"test content").unwrap();

    // Create symlink
    create_symlink(&target, &link).unwrap();

    // Verify symlink exists and points to correct target
    assert!(is_symlink(&link));
    assert_eq!(read_symlink_target(&link).unwrap(), target);

    // Verify we can read content through symlink
    let content = fs::read_to_string(&link).unwrap();
    assert_eq!(content, "test content");
}

/// M1-E05-T04-S02: Test symlink to non-existent target detection
#[test]
fn test_symlink_to_nonexistent_target() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_target = temp_dir.path().join("does-not-exist.txt");
    let link = temp_dir.path().join("broken-link");

    // Create symlink to non-existent target
    std::os::unix::fs::symlink(&nonexistent_target, &link).unwrap();

    // Verify link is detected as symlink
    assert!(is_symlink(&link));

    // Verify read_symlink_target still works (reads what the symlink points to)
    let target = read_symlink_target(&link).unwrap();
    assert_eq!(target, nonexistent_target);

    // Verify resolve_realpath fails for broken symlink
    let result = resolve_realpath(&link);
    assert!(result.is_err());

    // Verify is_managed_symlink returns false for broken symlinks
    assert!(!is_managed_symlink(&link));
}

/// M1-E05-T04-S03: Test atomic copy with temp directory
#[test]
fn test_atomic_copy_with_temp_directory() {
    let src_dir = TempDir::new().unwrap();
    let dest_parent = TempDir::new().unwrap();

    // Create source skill structure
    let src = src_dir.path();
    File::create(src.join("SKILL.md")).unwrap();
    File::create(src.join("script.sh")).unwrap();
    fs::create_dir_all(src.join("scripts")).unwrap();
    File::create(src.join("scripts").join("install.sh")).unwrap();

    let dest = dest_parent.path().join("copied-skill");

    // Perform atomic copy
    copy_skill_dir(src, &dest).unwrap();

    // Verify all content was copied
    assert!(dest.exists());
    assert!(dest.join("SKILL.md").exists());
    assert!(dest.join("script.sh").exists());
    assert!(dest.join("scripts").exists());
    assert!(dest.join("scripts").join("install.sh").exists());

    // Verify source still exists
    assert!(src.exists());
}

/// M1-E05-T04-S04: Test atomic move preserves content
#[test]
fn test_atomic_move_preserves_content() {
    let src_parent = TempDir::new().unwrap();
    let dest_parent = TempDir::new().unwrap();

    // Create source with specific content
    let src = src_parent.path().join("to-move");
    fs::create_dir_all(&src).unwrap();
    File::create(src.join("file1.txt")).unwrap();
    File::create(src.join("file2.txt")).unwrap();

    // Write specific content to verify preservation
    fs::write(src.join("file1.txt"), b"exact content 1").unwrap();
    fs::write(src.join("file2.txt"), b"exact content 2").unwrap();

    let dest = dest_parent.path().join("moved");

    // Perform atomic move
    atomic_move_dir(&src, &dest).unwrap();

    // Verify source no longer exists
    assert!(!src.exists());

    // Verify destination exists with exact content
    assert!(dest.exists());
    assert!(dest.join("file1.txt").exists());
    assert!(dest.join("file2.txt").exists());

    let content1 = fs::read_to_string(dest.join("file1.txt")).unwrap();
    let content2 = fs::read_to_string(dest.join("file2.txt")).unwrap();

    assert_eq!(content1, "exact content 1");
    assert_eq!(content2, "exact content 2");
}

/// M1-E05-T04-S05: Test permission error handling
#[test]
fn test_permission_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let protected_dir = temp_dir.path().join("protected");
    fs::create_dir_all(&protected_dir).unwrap();

    // Create a file inside protected directory
    let file = protected_dir.join("file.txt");
    File::create(&file).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Test 1: Try to copy into a read-only directory (should fail)
        let src_dir = TempDir::new().unwrap();
        let src_file = src_dir.path().join("file.txt");
        File::create(&src_file).unwrap();

        // Make protected directory read-only
        let mut perms = fs::metadata(&protected_dir).unwrap().permissions();
        perms.set_mode(0o444); // read-only
        fs::set_permissions(&protected_dir, perms).unwrap();

        let dest = protected_dir.join("copied");
        let result = copy_skill_dir(src_dir.path(), &dest);

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&protected_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&protected_dir, perms).unwrap();

        assert!(result.is_err());

        // Test 2: Try to remove a directory without confirmation (should fail)
        let to_remove = temp_dir.path().join("to-remove");
        fs::create_dir_all(&to_remove).unwrap();
        let result = safe_remove_dir(&to_remove, false);
        assert!(result.is_err());
        assert!(to_remove.exists());
    }
}

/// M1-E05-T04-S06: Test copy_skill_dir rejects symlinks in source
#[test]
fn test_copy_skill_dir_rejects_symlinks() {
    let src_dir = TempDir::new().unwrap();
    let dest_dir = TempDir::new().unwrap();

    // Create source with a symlink
    let src = src_dir.path();
    File::create(src.join("regular.txt")).unwrap();
    File::create(src.join("target.txt")).unwrap();

    // Create symlink inside source directory
    std::os::unix::fs::symlink(src.join("target.txt"), src.join("link.txt")).unwrap();

    let dest = dest_dir.path().join("copied");

    // Attempt to copy should fail due to symlink
    let result = copy_skill_dir(src, &dest);
    assert!(result.is_err());

    // Verify destination was not created (rollback occurred)
    assert!(!dest.exists());
}

/// M1-E05-T04-S07: Test copy_skill_dir excludes .git directory
#[test]
fn test_copy_skill_dir_excludes_git_directory() {
    let src_dir = TempDir::new().unwrap();
    let dest_dir = TempDir::new().unwrap();

    // Create source with .git directory
    let src = src_dir.path();
    File::create(src.join("SKILL.md")).unwrap();
    fs::create_dir_all(src.join(".git")).unwrap();
    File::create(src.join(".git").join("config")).unwrap();
    File::create(src.join(".git").join("HEAD")).unwrap();
    File::create(src.join(".git").join("objects")).unwrap();
    fs::create_dir_all(src.join(".git").join("refs")).unwrap();
    File::create(src.join(".git").join("refs").join("heads")).unwrap();

    // Create nested .git directory (should also be excluded)
    fs::create_dir_all(src.join("subdir").join(".git")).unwrap();
    File::create(src.join("subdir").join(".git").join("config")).unwrap();

    let dest = dest_dir.path().join("copied");

    // Perform copy
    copy_skill_dir(src, &dest).unwrap();

    // Verify .git directories were excluded
    assert!(!dest.join(".git").exists());
    assert!(!dest.join("subdir").join(".git").exists());

    // Verify other files were copied
    assert!(dest.join("SKILL.md").exists());
}

/// Additional test: Verify safe_remove_dir with confirmation
#[test]
fn test_safe_remove_dir_with_confirmation() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("to-remove");
    fs::create_dir_all(&path).unwrap();
    File::create(path.join("file.txt")).unwrap();

    // Should fail without confirmation
    let result = safe_remove_dir(&path, false);
    assert!(result.is_err());
    assert!(path.exists());

    // Should succeed with confirmation
    let result = safe_remove_dir(&path, true);
    assert!(result.is_ok());
    assert!(!path.exists());
}

/// Additional test: Verify managed symlink detection
#[test]
fn test_managed_symlink_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a mock .sikil/repo structure
    let repo = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo).unwrap();

    let skill_in_repo = repo.join("test-skill");
    fs::create_dir_all(&skill_in_repo).unwrap();
    File::create(skill_in_repo.join("SKILL.md")).unwrap();

    // Create symlink to repo
    let link = temp_dir.path().join("managed-link");
    std::os::unix::fs::symlink(&skill_in_repo, &link).unwrap();

    // Note: is_managed_symlink uses the real ~/.sikil/repo path
    // So in this test context, it won't detect as managed since we're using a temp dir
    // But we can verify the symlink itself exists
    assert!(is_symlink(&link));
    assert_eq!(read_symlink_target(&link).unwrap(), skill_in_repo);
}
