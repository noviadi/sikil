# Symlink Operations Spec

## One-Sentence Description

Symlink operations manage symbolic links between managed repository and agent directories.

## Overview

**Location:** `src/utils/symlink.rs`

The symlink module provides utilities for creating, reading, and resolving symbolic links, and determining whether a symlink is managed by Sikil.

**Note:** This module is Unix-only, using `std::os::unix::fs::symlink`.

## Functions

| Function | Purpose |
|----------|---------|
| `create_symlink(src, dest) -> Result<(), SikilError>` | Creates symlink; creates parent dirs; replaces existing |
| `is_symlink(path) -> bool` | Checks if path is symlink using `symlink_metadata` |
| `read_symlink_target(path) -> Result<PathBuf, SikilError>` | Returns symlink target (not resolved) |
| `resolve_realpath(path) -> Result<PathBuf, SikilError>` | Canonicalizes path, following all symlinks |
| `is_managed_symlink(path) -> bool` | Returns true if symlink target is under `~/.sikil/repo/` |

## Behavior Notes

- `create_symlink` removes existing file/symlink at dest before creating
- `is_managed_symlink` returns false for broken symlinks
- `read_symlink_target` returns error if path is not a symlink

## Error Handling

All functions return `Result<T, SikilError>` with:

| Error Variant | Used By |
|---------------|---------|
| `SikilError::SymlinkError { reason, source }` | All symlink operations |

## Dependencies

| Crate | Purpose |
|-------|---------|
| `std::os::unix::fs` | Unix symlink creation |

Internal: `crate::core::errors::SikilError`, `crate::utils::paths::get_repo_path`

## Used By

- **Commands:** Skill installation/removal
- **Core:** Skill linking to agent directories
