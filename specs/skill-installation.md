# Skill Installation Spec

## One-Sentence Description

Skill installation copies skills from local paths or Git repositories to `~/.sikil/repo/<name>/` and creates symlinks from agent skill directories to the managed repository.

## Overview

The install command supports two installation sources: local directories and Git URLs. Both flows validate the source skill (requiring a valid SKILL.md), copy the skill to the managed repository at `~/.sikil/repo/`, and create symlinks to one or more agent skill directories. The implementation includes rollback on partial failure and rejects symlinks in source directories.

## Installation Sources

1. **Local path** (`execute_install_local`)
   - Absolute or relative filesystem paths
   - Relative paths resolved against current working directory
   - Must be an existing directory containing SKILL.md

2. **Git URL** (`execute_install_git`)
   - Short form: `owner/repo` or `owner/repo/path/to/skill`
   - HTTPS URL: `https://github.com/owner/repo.git`
   - HTTPS URL without .git suffix: `https://github.com/owner/repo`

## Git URL Parsing

Implemented in `src/utils/git.rs`:

| Format | Example | Expansion |
|--------|---------|-----------|
| Short form | `owner/repo` | `https://github.com/owner/repo.git` |
| Short with subdir | `owner/repo/skills/my-skill` | Clone repo, extract `skills/my-skill` |
| HTTPS | `https://github.com/owner/repo.git` | Used directly |
| HTTPS without .git | `https://github.com/owner/repo` | Used directly |

**Security validations:**
- Only GitHub.com URLs allowed
- `file://` protocol rejected
- URLs starting with `-` rejected (argument injection protection)
- URLs with whitespace or NUL characters rejected
- Subdirectory paths with `..` rejected (path traversal protection)

## Installation Process

### Local Installation Flow

1. Parse source path (resolve relative to cwd if needed)
2. Validate source exists and is a directory
3. Validate SKILL.md exists and parse metadata to extract skill name
4. Determine target agents from `--to` flag or interactive prompt
5. Check `~/.sikil/repo/<name>` does not already exist
6. Check each agent skill directory for existing skill (dir or symlink)
7. Copy skill to `~/.sikil/repo/<name>/` using `copy_skill_dir` (rejects symlinks)
8. Create symlinks from each agent's skill directory to the repo copy
9. On failure during symlink creation, rollback: remove created symlinks and copied skill

### Git Installation Flow

1. Parse Git URL to extract clone_url, owner, repo, and optional subdirectory
2. Clone repository to temp directory with `--depth=1` (shallow clone)
3. If subdirectory specified, extract it to separate temp location
4. Validate SKILL.md exists and parse metadata
5. Determine target agents
6. Check repo and agent destinations for conflicts
7. Clean up clone (remove `.git/` directory)
8. Copy skill to `~/.sikil/repo/<name>/` using `copy_skill_dir`
9. Create symlinks to agents
10. On failure, rollback and clean up temp directories

## Agent Targeting

The `--to` flag determines which agents receive symlinks:

| Value | Behavior |
|-------|----------|
| Not specified | Interactive prompt (or all enabled in JSON mode) |
| `all` | All enabled agents from config |
| `claude-code` | Single agent by name |
| `claude-code,amp` | Comma-separated list of agents |

Implementation: `parse_agent_selection()` and `prompt_agent_selection()` in `src/commands/mod.rs`.

## Symlink Creation

Implemented in `src/utils/symlink.rs`:

1. Ensure parent directory exists (create if missing)
2. Remove existing symlink at destination if present
3. Create Unix symlink: `std::os::unix::fs::symlink(src, dest)`

**Symlink direction:** `agent_skill_dir/<name>` → `~/.sikil/repo/<name>/`

**Helper functions:**
- `is_symlink(path)` - Check if path is a symlink
- `read_symlink_target(path)` - Read symlink target
- `resolve_realpath(path)` - Canonicalize path following symlinks
- `is_managed_symlink(path)` - Check if symlink points to `~/.sikil/repo/`

## Error Conditions

| Condition | Error Type | Message/Behavior |
|-----------|------------|------------------|
| Source path not found | `DirectoryNotFound` | Path does not exist |
| Source not a directory | `ValidationError` | "source path is not a directory" |
| Missing SKILL.md | `InvalidSkillMd` | "SKILL.md not found" |
| Invalid SKILL.md content | `InvalidSkillMd` | Parsing failure details |
| No agents selected | `ValidationError` | "no agents selected" |
| Skill already in repo | `AlreadyExists` | "skill '<name>' in repository" |
| Destination is physical dir | `AlreadyExists` | Suggests `sikil adopt` |
| Destination is symlink | `AlreadyExists` | Suggests `sikil sync` |
| Source contains symlinks | `SymlinkNotAllowed` | Symlinks not permitted in skills |
| Git not installed | `GitError` | "git is not installed" |
| Clone failure | `GitError` | stderr from git |
| Subdirectory not found | `DirectoryNotFound` | Path within clone not found |
| Path traversal attempt | `PathTraversal` | Subdirectory contains `..` |
| Permission denied | `PermissionDenied` | Cannot create directories |

**Rollback behavior:** On partial failure during symlink creation, all created symlinks are removed and the copied skill directory is deleted.

## Dependencies

| Component | Location | Purpose |
|-----------|----------|---------|
| `Config` | `src/core/config.rs` | Agent configuration and paths |
| `parse_skill_md` | `src/core/parser.rs` | Extract skill name from SKILL.md |
| `copy_skill_dir` | `src/utils/atomic.rs` | Atomic copy rejecting symlinks |
| `create_symlink` | `src/utils/symlink.rs` | Unix symlink creation |
| `parse_git_url` | `src/utils/git.rs` | Git URL parsing |
| `clone_repo` | `src/utils/git.rs` | Git clone execution |
| `extract_subdirectory` | `src/utils/git.rs` | Subdirectory extraction |
| `cleanup_clone` | `src/utils/git.rs` | Remove .git directory |
| `get_repo_path` | `src/utils/paths.rs` | Returns `~/.sikil/repo/` |
| `ensure_dir_exists` | `src/utils/paths.rs` | Create directories |

## Used By

| Command | Usage |
|---------|-------|
| `sikil install <path>` | Install from local directory |
| `sikil install <git-url>` | Install from Git repository |
