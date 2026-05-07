# Skill Synchronization Spec

## One-Sentence Description

Skill synchronization creates symlinks for managed skills in agents where they are missing.

## Overview

The sync command links managed skills from a managed store to agent skill directories. The source store and target agent directories are scope-dependent: in **project scope**, the source is `<project_root>/.sikil/skills/<name>/` and targets are each agent's workspace path under the project root; in **global scope**, the source is `~/.sikil/repo/<name>/` and targets are each agent's global path. Sync identifies which target agents are missing the symlink and creates them; existing managed symlinks are left untouched.

## Sync Scope

- **Single skill**: `sikil sync <skill-name>` syncs one specific skill
- **All skills**: `sikil sync --all` syncs every managed skill in the resolved store (project store in project scope; global repo in global scope)
- A skill is considered "managed" if it exists as a directory in the chosen store and contains a `SKILL.md` file
- Hidden directories (starting with `.`) are skipped when using `--all`

## Source Store Selection

| Scope | Source | Target agent paths |
|-------|--------|--------------------|
| Project (default in project root) | `<project_root>/.sikil/skills/<name>/` | `<project_root>/<workspace_path>/<name>` per agent |
| Global (default outside project, or `--global`) | `~/.sikil/repo/<name>/` | `<global_path>/<name>` per agent |

`--project`/`--global` flags override scope discovery. In project scope, target symlinks are **relative** (`../../.sikil/skills/<name>`); in global scope, target symlinks are **absolute**.

## Agent Targeting

- **Default**: When no `--to` flag is provided, syncs to all enabled agents (`parse_agent_selection(Some("all"), config)`); in project scope, the manifest's per-skill `agents` field (if present) further restricts the default per [agent-targeting.md](agent-targeting.md)
- **Specific agents**: Use `--to <agent>` to target specific agents
- Agent paths are resolved from `Config::get_agent()` using `agent_config.global_path` (global scope) or `agent_config.workspace_path` anchored at project root (project scope)

## Sync Process

1. **Validate input**: Either `--all` or a skill name must be provided
2. **Resolve scope**: project (default in project root) or global; `--global`/`--project` flags override
3. **Locate skill in store**: Find skill directory at `<store_path>/<skill-name>`
4. **Validate skill**: Verify `SKILL.md` exists in the skill directory
5. **Determine target agents**: Parse `--to` flag, fall back to manifest `agents` field (project scope) or all enabled agents
6. **Check each agent**:
   - If symlink exists → mark as "already synced"
   - If physical directory exists → fail with adopt suggestion
   - If nothing exists → mark as "missing"
7. **Create symlinks**: For each missing agent, create symlink from `<agent_path>/<skill-name>` → `<store_path>/<skill-name>` (relative target in project scope, absolute in global)
8. **Ensure agent directory exists**: Creates agent skill directory if needed via `ensure_dir_exists()`

## Skip Conditions

| Condition | Behavior |
|-----------|----------|
| Symlink already exists at agent path | Skip, mark as "already synced" |
| Physical directory exists (not symlink) | Error with message: "use `sikil adopt` to manage it" |
| Agent directory creation fails | Warning, continue to next agent |
| Symlink creation fails | Warning, continue to next agent |
| All agents already synced | Success with info message, no action taken |

## Dry Run

Not supported. The implementation has no `--dry-run` flag in `SyncArgs`.

## Acceptance Criteria

- `sikil sync <name>` creates symlinks for the named skill in all target agents
- `sikil sync --all` syncs every managed skill in the resolved store
- In project scope, source is `<project_root>/.sikil/skills/<name>/` and symlinks are relative
- In global scope, source is `~/.sikil/repo/<name>/` and symlinks are absolute
- Default `--to` targets all enabled agents (further restricted by manifest `agents` field in project scope)
- Agents already having the skill (symlink exists) are skipped
- Physical directory at agent path returns error suggesting `sikil adopt`
- Skill not found in resolved store returns error
- Skill missing `SKILL.md` returns validation error
- Hidden directories (starting with `.`) are skipped when using `--all`
- Agent skill directory is created if it doesn't exist
- Neither skill name nor `--all` provided returns `ValidationError`
- `--project` outside any project root returns `OutsideProject` error

## Dependencies

- `crate::core::config::Config` - Agent configuration and paths
- `crate::utils::paths::{ensure_dir_exists, get_repo_path, find_project_root, get_project_skills_path}` - Directory, repo, and project store path utilities
- `crate::utils::symlink::{create_symlink, is_symlink}` - Symlink operations
- `crate::commands::parse_agent_selection` - Agent targeting logic
- Manifest layer per [project-manifest.md](project-manifest.md) - Reads manifest `agents` field in project scope

## Used By

- CLI `sync` subcommand
- Called after `install` command to propagate new skills to agents
