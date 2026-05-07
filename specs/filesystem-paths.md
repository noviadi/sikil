# Filesystem Paths Spec

## One-Sentence Description

Filesystem paths define Sikil's on-disk directory layout.

## Overview

**Location:** `src/utils/paths.rs`

The paths module provides utilities for path expansion and resolving standard Sikil directories. All filesystem operations use `fs-err` instead of `std::fs` for better error messages.

## Functions

### Global Paths

| Function | Purpose |
|----------|---------|
| `expand_path(path: &str) -> PathBuf` | Expands `~` and `$VAR` using `shellexpand` |
| `get_repo_path() -> PathBuf` | Returns `~/.sikil/repo/` |
| `get_config_path() -> PathBuf` | Returns `~/.sikil/config.toml` |
| `get_cache_path() -> PathBuf` | Returns `~/.sikil/cache.json` |
| `ensure_dir_exists(path: &Path) -> Result<(), std::io::Error>` | Creates directory with parents if needed |

All global `get_*` functions use `directories::UserDirs` to resolve the home directory and panic on home directory lookup failure.

### Project Paths

| Function | Purpose |
|----------|---------|
| `find_project_root(start: &Path) -> Option<PathBuf>` | Walks upward from `start`; returns first directory containing `.sikil/manifest.toml` (preferred) or `.git` (fallback); returns `None` if reached filesystem root with no marker |
| `get_project_root() -> Option<PathBuf>` | Convenience wrapper around `find_project_root(env::current_dir()?)` |
| `get_manifest_path(project_root: &Path) -> PathBuf` | Returns `<project_root>/.sikil/manifest.toml` |
| `get_lock_path(project_root: &Path) -> PathBuf` | Returns `<project_root>/.sikil/lock.toml` |
| `get_project_skills_path(project_root: &Path) -> PathBuf` | Returns `<project_root>/.sikil/skills/` |

Project-root discovery is **non-fatal**: returning `None` indicates the operation should run in global scope (or error with `OutsideProject` if `--project` was passed). A `.git` *file* (worktree marker) is treated equivalently to a `.git/` *directory*.

## Acceptance Criteria

- `expand_path` expands `~` to the user's home directory
- `expand_path` expands environment variables like `$HOME` in paths
- `get_repo_path` returns `~/.sikil/repo/` expanded to absolute path
- `get_config_path` returns `~/.sikil/config.toml` expanded to absolute path
- `get_cache_path` returns `~/.sikil/cache.json` expanded to absolute path
- `ensure_dir_exists` creates parent directories if they don't exist
- `ensure_dir_exists` succeeds silently if directory already exists
- `get_*` functions for global paths panic if home directory cannot be determined
- `find_project_root` returns the directory containing `.sikil/manifest.toml` when present, walking upward from the start path
- `find_project_root` returns the directory containing `.git` (file or directory) when no manifest is found higher up
- `find_project_root` prefers `.sikil/manifest.toml` over `.git` when both are present at different levels (manifest wins regardless of depth)
- `find_project_root` returns `None` when neither marker is found before reaching the filesystem root
- `find_project_root` treats a `.git` file (worktree marker) equivalently to a `.git/` directory
- `get_manifest_path` returns `<project_root>/.sikil/manifest.toml`
- `get_lock_path` returns `<project_root>/.sikil/lock.toml`
- `get_project_skills_path` returns `<project_root>/.sikil/skills/`

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

- **Commands:** Any command needing path resolution; `init`, `install`, `add`, `update`, `remove` consume project-root discovery
- **Core:** All modules needing Sikil directory locations
- **Symlink utilities:** For `is_managed_symlink` check (accepts both global repo and per-project `.sikil/skills/` as managed targets — see [symlink-operations.md](symlink-operations.md))
- **Manifest layer:** `get_manifest_path`, `get_lock_path`, `get_project_skills_path` consumed by [project-manifest.md](project-manifest.md)
