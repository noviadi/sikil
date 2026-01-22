# Skill Synchronization Spec

## One-Sentence Description

Skill synchronization creates symlinks for managed skills in agents where they are missing.

## Overview

The sync command links managed skills from the central repository to agent skill directories. It identifies which agents are missing a skill and creates symlinks to the repo copy, ensuring all targeted agents have access to the skill.

## Sync Scope

- **Single skill**: `sikil sync <skill-name>` syncs one specific skill
- **All skills**: `sikil sync --all` syncs every managed skill in the repository
- A skill is considered "managed" if it exists as a directory in the repo path and contains a `SKILL.md` file
- Hidden directories (starting with `.`) are skipped when using `--all`

## Agent Targeting

- **Default**: When no `--to` flag is provided, syncs to all enabled agents (`parse_agent_selection(Some("all"), config)`)
- **Specific agents**: Use `--to <agent>` to target specific agents
- Agent paths are resolved from `Config::get_agent()` using `agent_config.global_path`

## Sync Process

1. **Validate input**: Either `--all` or a skill name must be provided
2. **Locate skill in repo**: Find skill directory at `repo_path/<skill-name>`
3. **Validate skill**: Verify `SKILL.md` exists in the skill directory
4. **Determine target agents**: Parse `--to` flag or default to all enabled agents
5. **Check each agent**:
   - If symlink exists → mark as "already synced"
   - If physical directory exists → fail with adopt suggestion
   - If nothing exists → mark as "missing"
6. **Create symlinks**: For each missing agent, create symlink from `agent_path/<skill-name>` → `repo_path/<skill-name>`
7. **Ensure agent directory exists**: Creates agent skill directory if needed via `ensure_dir_exists()`

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

## Dependencies

- `crate::core::config::Config` - Agent configuration and paths
- `crate::utils::paths::{ensure_dir_exists, get_repo_path}` - Directory and repo path utilities
- `crate::utils::symlink::{create_symlink, is_symlink}` - Symlink operations
- `crate::commands::parse_agent_selection` - Agent targeting logic

## Used By

- CLI `sync` subcommand
- Called after `install` command to propagate new skills to agents
