# Git Operations Spec

## One-Sentence Description

Git operations retrieve skills from GitHub repositories.

## Overview

The `src/utils/git.rs` module provides secure Git URL parsing, repository cloning, subdirectory extraction, and cleanup utilities. It is used by the install command to fetch skills from remote Git repositories.

## URL Parsing

The `parse_git_url()` function accepts two formats:

| Format | Example | Expansion |
|--------|---------|-----------|
| Short form | `owner/repo` | `https://github.com/owner/repo.git` |
| Short form with subdirectory | `owner/repo/path/to/skill` | Clone `https://github.com/owner/repo.git`, extract `path/to/skill` |
| HTTPS URL | `https://github.com/owner/repo.git` | Used as-is |
| HTTPS without .git | `https://github.com/owner/repo` | Used as-is |

Returns a `ParsedGitUrl` struct containing:
- `clone_url`: Full HTTPS URL to clone
- `owner`: Repository owner/username
- `repo`: Repository name
- `subdirectory`: Optional path within the repository

## Clone Process

The `clone_repo()` function executes:

```
git clone -c protocol.file.allow=never --depth=1 -- <url> <dest>
```

Key behaviors:
- Checks `git --version` first to verify git is installed
- Uses `std::process::Command` with array arguments (no shell interpolation)
- Uses `--depth=1` for shallow clone (bandwidth optimization)
- Uses `--` separator before URL to prevent option injection
- Uses `-c protocol.file.allow=never` to block file:// protocol at git level
- Returns `SikilError::GitError` on failure with stderr output

## Subdirectory Extraction

The `extract_subdirectory()` function copies a subdirectory from a cloned repo to a temporary location:

1. **Validation**: Rejects paths containing `..` or absolute paths
2. **Path resolution**: Uses `canonicalize()` to resolve symlinks and verify the subdirectory is within clone root
3. **Extraction**: Creates a temporary directory via `tempfile::tempdir()` and copies contents recursively
4. **Ownership**: Calls `keep()` on the temp directory so caller manages cleanup

The `copy_dir_recursive()` helper copies files, directories, and symlinks.

## Security Considerations

**URL Validation** (`parse_git_url`):
- GitHub-only: Rejects non-GitHub.com URLs
- Rejects `file://` protocol
- Rejects URLs with whitespace or NUL characters
- Rejects URLs starting with `-` (argument injection protection)
- Validates owner/repo are non-empty

**Clone Security** (`clone_repo`):
- No shell execution (array-based `Command`)
- `--` separator prevents URL being interpreted as git option
- `-c protocol.file.allow=never` blocks file:// even if validation is bypassed

**Path Traversal Prevention** (`extract_subdirectory`):
- Rejects paths containing `..`
- Rejects absolute paths
- Canonicalizes and verifies subdirectory is within clone root

## Error Handling

| Error Type | Trigger |
|------------|---------|
| `SikilError::InvalidGitUrl` | Invalid URL format, non-GitHub host, invalid characters |
| `SikilError::GitError` | Git not installed, clone failure, temp dir creation failure |
| `SikilError::DirectoryNotFound` | Subdirectory doesn't exist in cloned repo |
| `SikilError::PathTraversal` | Path contains `..`, is absolute, or escapes clone root |

## Dependencies

- `std::process::Command` for git execution
- `tempfile` crate for temporary directories
- `crate::core::errors::SikilError` for error types

## Used By

- `src/commands/install.rs`: `execute_install_git()` uses `parse_git_url()`, `clone_repo()`, `extract_subdirectory()`, and `cleanup_clone()` to install skills from Git repositories
