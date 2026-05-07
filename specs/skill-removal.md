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
- With `--agent`: Removes symlinks/directories only from specified agents; canonical store entry is preserved unless orphaned
- With `--all`: Removes all installations AND deletes the skill from the canonical store (project store and/or global repo, depending on scope of each installation)
- Supports both managed skills (symlinks) and unmanaged skills (physical directories)
- In project scope, `--all` also removes the `[skill.<name>]` entry from `.sikil/manifest.toml` and `.sikil/lock.toml` and deletes `<project_root>/.sikil/skills/<name>/`
- In global scope, `--all` removes `~/.sikil/repo/<name>/` (including its `.sikil-source.toml` sidecar)
- Detects orphaned canonical-store entries after `--agent` removal and prompts to delete them; project-managed orphans clean manifest+lockfile entries, global-managed orphans clean the sidecar implicitly with the directory

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
3. Resolve scope (project default in project root; `--global`/`--project` overrides)
4. Scan all agent directories to find skill installations by name (per [skill-scanner.md](skill-scanner.md))
5. Filter installations to those matching target agents
6. Display what will be removed (with scope annotation per installation)
7. Prompt for confirmation (unless `--yes` or `--json`)
8. For each installation:
   - If symlink: call `fs::remove_file()`
   - If directory: call `fs::remove_dir_all()`
9. If `--agent` was used and a canonical-store entry is now orphaned:
   - Prompt to delete; on confirmation, perform the appropriate cleanup below
10. If `--all` (or after orphan-cleanup confirmation):
    - **Project-managed skill:** atomically rewrite `.sikil/manifest.toml` and `.sikil/lock.toml` removing the `[skill.<name>]` entry; remove `<project_root>/.sikil/skills/<name>/` with `safe_remove_dir(_, true)`
    - **Global-managed skill:** remove `~/.sikil/repo/<name>/` (including its `.sikil-source.toml` sidecar) with `safe_remove_dir(_, true)`

## Acceptance Criteria

- `--all` removes skill from all agents AND deletes from the canonical store
- `--agent` removes only from specified agents; canonical-store entry is preserved unless orphaned
- Both managed (symlinks) and unmanaged (directories) skills can be removed
- In project scope, `--all` removes the `[skill.<name>]` manifest entry, the lockfile entry, and the vendored bytes at `<project_root>/.sikil/skills/<name>/`
- In global scope, `--all` removes `~/.sikil/repo/<name>/` including its `.sikil-source.toml` sidecar
- Manifest and lockfile rewrites use atomic temp + rename; a crash mid-write leaves the prior file intact
- Missing both `--agent` and `--all` returns `ValidationError`
- Invalid agent name returns `ValidationError`
- Skill not found returns `SkillNotFound` error
- Confirmation prompt shows `[y/N]` format with default "no"
- `--yes` flag skips confirmation prompt
- `--json` mode skips confirmation prompt
- Empty input at confirmation prompt cancels operation
- User entering `n` or `N` cancels operation with `PermissionDenied` error
- Orphaned canonical-store entry after `--agent` removal triggers deletion prompt
- Orphan cleanup in project scope removes manifest and lockfile entries

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
| `crate::utils::atomic::atomic_write_file` | Atomic manifest/lockfile rewrites when removing entries |
| `crate::utils::paths::get_repo_path` | Get global managed repository path (`~/.sikil/repo/`) |
| `crate::utils::paths::find_project_root` | Discover project root for project-managed skills |
| `crate::utils::paths::get_project_skills_path` | Get `<project_root>/.sikil/skills/` path |
| Manifest layer | per [project-manifest.md](project-manifest.md) | Manifest+lockfile entry removal (project scope) |
| `fs-err` | Enhanced filesystem operations with better error messages |

## Used By

This command is invoked by the CLI layer:
- `sikil remove <name> --agent <agents>` - Remove from specific agents
- `sikil remove <name> --all` - Remove from all agents and repository
