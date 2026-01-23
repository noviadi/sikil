# Filesystem Paths Spec

## One-Sentence Description

Filesystem paths define Sikil's on-disk directory layout.

## Overview

**Location:** `src/utils/paths.rs`

The paths module provides utilities for path expansion and resolving standard Sikil directories. All filesystem operations use `fs-err` instead of `std::fs` for better error messages.

## Functions

| Function | Purpose |
|----------|---------|
| `expand_path(path: &str) -> PathBuf` | Expands `~` and `$VAR` using `shellexpand` |
| `get_repo_path() -> PathBuf` | Returns `~/.sikil/repo/` |
| `get_config_path() -> PathBuf` | Returns `~/.sikil/config.toml` |
| `get_cache_path() -> PathBuf` | Returns `~/.sikil/cache.sqlite` |
| `ensure_dir_exists(path: &Path) -> Result<(), std::io::Error>` | Creates directory with parents if needed |

All `get_*` functions use `directories::UserDirs` to resolve the home directory and panic on home directory lookup failure.

## Acceptance Criteria

- `expand_path` expands `~` to the user's home directory
- `expand_path` expands environment variables like `$HOME` in paths
- `get_repo_path` returns `~/.sikil/repo/` expanded to absolute path
- `get_config_path` returns `~/.sikil/config.toml` expanded to absolute path
- `get_cache_path` returns `~/.sikil/cache.sqlite` expanded to absolute path
- `ensure_dir_exists` creates parent directories if they don't exist
- `ensure_dir_exists` succeeds silently if directory already exists
- `get_*` functions panic if home directory cannot be determined

## Error Handling

- `ensure_dir_exists` returns `std::io::Error`
- `get_*` functions panic on home directory lookup failure

## Dependencies

| Crate | Purpose |
|-------|---------|
| `fs-err` | Filesystem operations with better error messages |
| `shellexpand` | Tilde and environment variable expansion |
| `directories` | Cross-platform home directory lookup |

## Used By

- **Commands:** Any command needing path resolution
- **Core:** All modules needing Sikil directory locations
- **Symlink utilities:** For `is_managed_symlink` check
