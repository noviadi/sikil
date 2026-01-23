# Skill Unmanagement Spec

## One-Sentence Description

Skill unmanagement converts managed skills back to standalone copies.

## Overview

The `unmanage` command converts managed skills (symlinks pointing to `~/.sikil/repo/`) back to unmanaged skills (physical directories in agent directories).

## Arguments

**`UnmanageArgs`:**
- `name` (required): Name of the skill to unmanage
- `--agent`: Specific agent to unmanage from
- `--yes`: Skip confirmation prompt
- `--json`: Output in JSON format

## Behavior

- Only works on managed skills (symlinks pointing to `~/.sikil/repo/`)
- Removes the symlink and copies content from repository to the original location
- Without `--agent`: Unmanages all managed installations
- With `--agent`: Unmanages only the specified agent's installation
- When all installations are unmanaged, automatically deletes the skill from the repository
- If copy fails, attempts to restore the original symlink

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

## Unmanage Process

1. Parse `--agent` if provided
2. Scan all agent directories to find skill by name
3. Verify skill is managed (exists in `~/.sikil/repo/`)
4. Filter to only managed installations (symlinks with targets)
5. If `--agent` specified, filter to that agent only
6. Display what will be unmanaged
7. Prompt for confirmation (unless `--yes` or `--json`)
8. For each installation:
   - Remove symlink with `fs::remove_file()`
   - Copy content from repository with `copy_skill_dir()`
   - On copy failure: remove partial copy, restore symlink
9. If all managed installations were unmanaged:
   - Delete skill from repository with `fs::remove_dir_all()`

## Acceptance Criteria

- Unmanaging replaces symlink with physical copy of skill content
- Without `--agent`, all managed installations are unmanaged
- With `--agent`, only that agent's installation is unmanaged
- When all installations are unmanaged, repository entry is deleted
- Unmanaging an unmanaged skill (not a symlink) returns `ValidationError`
- Skill not found returns `SkillNotFound` error
- Skill not in repository returns `ValidationError`
- Invalid agent name returns `ValidationError`
- Copy failure removes partial copy and restores original symlink
- Confirmation prompt shows `[y/N]` format with default "no"
- `--yes` flag skips confirmation prompt
- `--json` mode skips confirmation prompt
- User canceling confirmation returns `PermissionDenied` error

## Error Conditions

| Condition | Error Type |
|-----------|------------|
| Invalid agent name | `ValidationError` |
| Skill not found | `SkillNotFound` |
| Skill is not managed | `ValidationError` |
| Skill not found in repository | `ValidationError` |
| No managed installations found | `ValidationError` |
| Skill not managed for specified agent | `ValidationError` |
| Failed to remove symlink | `SymlinkError` |
| Failed to copy skill content | `SymlinkError` |
| Failed to remove from repository | `PermissionDenied` |
| User cancels operation | `PermissionDenied` |

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `crate::cli::output::Output` | Formatted output (info, success, warning, error) |
| `crate::core::config::Config` | Agent directory configuration |
| `crate::core::errors::SikilError` | Error types |
| `crate::core::scanner::Scanner` | Scan agent directories to find skill installations |
| `crate::core::skill::Agent` | Agent enum with `from_cli_name()` parsing |
| `crate::utils::atomic::copy_skill_dir` | Copy skill directory |
| `crate::utils::paths::get_repo_path` | Get managed repository path (`~/.sikil/repo/`) |
| `fs-err` | Enhanced filesystem operations with better error messages |

## Used By

This command is invoked by the CLI layer:
- `sikil unmanage <name>` - Unmanage all installations
- `sikil unmanage <name> --agent <agent>` - Unmanage from specific agent
