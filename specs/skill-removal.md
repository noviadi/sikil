# Skill Removal Spec

## One-Sentence Description

Skill removal deletes skills from agent directories or the managed repository.

## Overview

The `remove` command deletes skill installations from agent directories, with the option to also delete from the managed repository.

## Arguments

**`RemoveArgs`:**
- `name` (required): Name of the skill to remove
- `--agent`: Comma-separated list of agents to remove from (e.g., `claude-code,windsurf`)
- `--all`: Remove from all agents AND delete from repository
- `--yes`: Skip confirmation prompt
- `--json`: Output in JSON format

## Behavior

- Requires either `--agent` or `--all` (no default behavior)
- With `--agent`: Removes symlinks/directories only from specified agents; repository entry is preserved unless orphaned
- With `--all`: Removes all installations AND deletes the skill from the repository
- Supports both managed skills (symlinks) and unmanaged skills (physical directories)
- Detects orphaned repository entries after `--agent` removal and prompts to delete them

## Confirmation

Uses an interactive confirmation prompt with `[y/N]` format.

**Confirmation is skipped when:**
- `--yes` flag is provided
- `--json` mode is enabled (for non-interactive/scripted use)

**Confirmation behavior:**
- Empty input defaults to "no"
- Accepts `y`, `yes`, `Y` for confirmation
- Accepts `n`, `no`, `N`, or any other input for rejection
- Cancellation returns a `PermissionDenied` error

## Removal Process

1. Validate that `--agent` or `--all` is specified
2. Parse `--agent` string into list of `Agent` values if provided
3. Scan all agent directories to find skill installations by name
4. Filter installations to those matching target agents
5. Display what will be removed
6. Prompt for confirmation (unless `--yes` or `--json`)
7. For each installation:
   - If symlink: call `fs::remove_file()`
   - If directory: call `fs::remove_dir_all()`
8. If `--agent` was used and repository is now orphaned (no remaining installations):
   - Prompt to delete orphaned repository entry
9. If `--all` and skill was managed:
   - Delete skill directory from repository using `safe_remove_dir()`

## Error Conditions

| Condition | Error Type |
|-----------|------------|
| Neither `--agent` nor `--all` specified | `ValidationError` |
| Invalid agent name in `--agent` | `ValidationError` |
| Skill not found by name | `SkillNotFound` |
| Skill not installed for specified agent(s) | `ValidationError` |
| No installations found | `ValidationError` |
| Failed to remove installation | `PermissionDenied` |
| Failed to remove repository entry | `PermissionDenied` |
| User cancels operation | `PermissionDenied` |

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `crate::cli::output::Output` | Formatted output (info, success, warning, error) |
| `crate::core::config::Config` | Agent directory configuration |
| `crate::core::errors::SikilError` | Error types |
| `crate::core::scanner::Scanner` | Scan agent directories to find skill installations |
| `crate::core::skill::Agent` | Agent enum with `from_cli_name()` parsing |
| `crate::utils::atomic::safe_remove_dir` | Safe directory removal |
| `crate::utils::paths::get_repo_path` | Get managed repository path (`~/.sikil/repo/`) |
| `fs-err` | Enhanced filesystem operations with better error messages |

## Used By

This command is invoked by the CLI layer:
- `sikil remove <name> --agent <agents>` - Remove from specific agents
- `sikil remove <name> --all` - Remove from all agents and repository
