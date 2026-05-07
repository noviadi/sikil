# Skill Unmanagement Spec

## One-Sentence Description

Skill unmanagement converts managed skills back to standalone copies.

## Overview

The `unmanage` command converts managed skills (symlinks pointing into a managed store) back to unmanaged skills (physical directories in agent directories). Both managed-store roots are recognized: the global repo at `~/.sikil/repo/` and the per-project store at `<project_root>/.sikil/skills/`. The source store is selected automatically based on the scope reachable from each symlink (project store wins when both exist for the same skill name and the symlink resolves there). When a project-managed skill is fully unmanaged, the corresponding manifest entry, lockfile entry, and vendored bytes are removed; when a global-managed skill is fully unmanaged, the global repo entry and its sidecar are removed.

## Arguments

**`UnmanageArgs`:**
- `name` (required): Name of the skill to unmanage
- `--agent`: Specific agent to unmanage from
- `--yes`: Skip confirmation prompt
- `--json`: Output in JSON format

## Behavior

- Only works on managed skills (symlinks pointing into a recognized managed store: `~/.sikil/repo/` or any reachable `<project_root>/.sikil/skills/`)
- Removes the symlink and copies content from the relevant managed store to the original location
- Without `--agent`: Unmanages all managed installations of the named skill
- With `--agent`: Unmanages only the specified agent's installation
- When all installations of a project-managed skill are unmanaged: removes the manifest entry, lockfile entry, and `.sikil/skills/<name>/` directory (subject to confirmation)
- When all installations of a global-managed skill are unmanaged: removes `~/.sikil/repo/<name>/` (and its sidecar) (subject to confirmation)
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

1. Parse `--agent` if provided; resolve scope (`--global`/`--project` overrides)
2. Scan all agent directories to find skill by name
3. For each managed installation, classify the source store (project or global) by inspecting the resolved symlink target
4. Verify at least one managed installation exists in the resolved scope
5. Filter to only managed installations (symlinks with targets in a recognized store)
6. If `--agent` specified, filter to that agent only
7. Display what will be unmanaged (with source store annotation per installation)
8. Prompt for confirmation (unless `--yes` or `--json`)
9. For each installation:
   - Remove symlink with `fs::remove_file()`
   - Copy content from the relevant managed store with `copy_skill_dir()`
   - On copy failure: remove partial copy, restore symlink
10. If all managed installations of a project-managed skill were unmanaged:
    - Remove the `[skill.<name>]` entry from `.sikil/manifest.toml` (atomic write)
    - Remove the `[skill.<name>]` entry from `.sikil/lock.toml` (atomic write)
    - Remove `<project_root>/.sikil/skills/<name>/` with `fs::remove_dir_all()`
11. If all managed installations of a global-managed skill were unmanaged:
    - Remove `~/.sikil/repo/<name>/` (including its `.sikil-source.toml`) with `fs::remove_dir_all()`

## Acceptance Criteria

- Unmanaging replaces symlink with physical copy of skill content
- Without `--agent`, all managed installations are unmanaged
- With `--agent`, only that agent's installation is unmanaged
- Symlinks targeting `<project_root>/.sikil/skills/` are recognized as project-managed and copy from that store
- Symlinks targeting `~/.sikil/repo/` are recognized as global-managed and copy from that store
- When all installations of a project-managed skill are unmanaged, the manifest entry, lockfile entry, and vendored bytes are removed
- When all installations of a global-managed skill are unmanaged, the global repo entry (including its sidecar) is removed
- Unmanaging an unmanaged skill (not a symlink) returns `ValidationError`
- Skill not found returns `SkillNotFound` error
- Skill not in any managed store returns `ValidationError`
- Invalid agent name returns `ValidationError`
- Copy failure removes partial copy and restores original symlink
- Manifest/lockfile write failure during cleanup is non-fatal: warning is emitted, vendored bytes remain in place for recovery
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
| `crate::utils::atomic::atomic_write_file` | Atomic manifest/lockfile rewrites when removing entries |
| `crate::utils::paths::get_repo_path` | Get global managed repository path (`~/.sikil/repo/`) |
| `crate::utils::paths::find_project_root` | Discover project root for project-managed skills |
| `crate::utils::paths::get_project_skills_path` | Get `<project_root>/.sikil/skills/` path |
| Manifest layer | per [project-manifest.md](project-manifest.md) | Remove manifest+lockfile entries (project scope) |
| Provenance layer | per [provenance.md](provenance.md) | Sidecar removed implicitly with `~/.sikil/repo/<name>/` (global scope) |
| `fs-err` | Enhanced filesystem operations with better error messages |

## Used By

This command is invoked by the CLI layer:
- `sikil unmanage <name>` - Unmanage all installations
- `sikil unmanage <name> --agent <agent>` - Unmanage from specific agent
