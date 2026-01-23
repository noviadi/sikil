# Atomic Operations Spec

## One-Sentence Description

Atomic operations ensure safe filesystem mutations with rollback capability.

## Overview

**Location:** `src/utils/atomic.rs`

The atomic module provides safe, atomic filesystem operations for managing skill directories, including rollback capabilities and safeguards against partial failures. All filesystem operations use `fs-err` instead of `std::fs` for better error messages.

## Functions

| Function | Purpose |
|----------|---------|
| `copy_skill_dir(src, dest) -> Result<(), SikilError>` | Deep copy excluding `.git` and rejecting symlinks |
| `atomic_move_dir(src, dest) -> Result<(), SikilError>` | Atomic rename; falls back to copy+delete |
| `safe_remove_dir(path, confirmed) -> Result<(), SikilError>` | Removes directory only if `confirmed=true` |

## Behavior Notes

- `copy_skill_dir` uses `WalkDir::follow_links(false)` to detect symlinks
- On copy failure, tracks copied files/dirs and removes them in reverse order (rollback)
- `atomic_move_dir` tries `fs::rename` first (same filesystem), falls back to copy+remove
- Cross-filesystem moves back up existing destination, restore on failure
- `safe_remove_dir` requires explicit confirmation to prevent accidental deletions

## Acceptance Criteria

- `copy_skill_dir` deep copies all files and subdirectories from source to destination
- `copy_skill_dir` excludes `.git` directory from the copy
- `copy_skill_dir` returns `SikilError::SymlinkNotAllowed` if source contains symlinks
- `copy_skill_dir` returns `SikilError::DirectoryNotFound` if source doesn't exist
- `copy_skill_dir` rolls back (removes partial copy) on failure
- `atomic_move_dir` uses `fs::rename` for same-filesystem moves
- `atomic_move_dir` falls back to copy+delete for cross-filesystem moves
- `atomic_move_dir` restores existing destination on failure during cross-filesystem move
- `safe_remove_dir` returns `SikilError::ValidationError` if `confirmed=false`
- `safe_remove_dir` removes directory and all contents if `confirmed=true`

## Error Handling

All functions return `Result<T, SikilError>` with:

| Error Variant | Used By |
|---------------|---------|
| `SikilError::DirectoryNotFound { path }` | All operations when source doesn't exist |
| `SikilError::PermissionDenied { operation, path }` | Copy/move failures |
| `SikilError::ValidationError { reason }` | `safe_remove_dir` without confirmation |
| `SikilError::SymlinkNotAllowed { reason }` | `copy_skill_dir` (symlink in source) |
| `SikilError::PathTraversal { path }` | `copy_skill_dir` (strip_prefix failure) |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `fs-err` | Filesystem operations with better error messages |
| `walkdir` | Recursive directory traversal |
| `tempfile` | Temporary directories for backup during atomic moves |

Internal: `crate::core::errors::SikilError`

## Used By

- **Commands:** Skill installation, updates, and removal
- **Core:** Skill copying and moving operations
