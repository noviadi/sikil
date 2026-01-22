# Skill Discovery Spec

## One-Sentence Description

Skill discovery displays installed skills and their details across all agent directories.

## Overview

The skill discovery system scans configured agent directories for installed skills and presents them in both human-readable and JSON formats. It consists of two commands: `list` for viewing all skills with optional filtering, and `show` for detailed information about a specific skill.

## List Command

The `list` command (`src/commands/list.rs`) scans all configured agents and displays installed skills:

- Uses `Scanner` to scan all agent directories for skills
- Groups skills by managed/unmanaged status
- Sorts skills alphabetically by name within each group
- Detects conflicts using `conflicts::detect_conflicts()`
- Shows disabled default agents when no skills are found
- Outputs a tabular format with columns: NAME, DESCRIPTION, AGENTS
- Displays status indicators: `✓` for managed, `?` for unmanaged
- Shows directory name if different from skill name (occurs when SKILL.md `name` field differs from containing directory)
- Prints conflict details and recommendations when conflicts exist

## Show Command

The `show` command (`src/commands/show.rs`) displays detailed information about a specific skill:

- Looks up skill by name from scan results
- Returns `SkillNotFound` error if skill doesn't exist
- Displays metadata: name, description, version, author, license
- Shows managed status and canonical path (repo path for managed, first installation for unmanaged)
- Lists all installations with: agent, path, scope, symlink status, symlink target
- Includes file tree info: has_skill_md, has_scripts_dir, has_references_dir, file_count
- Calculates and displays total size in bytes (with human-readable formatting)
- Excludes `.git` directory from file counts

## Filtering

The list command supports these filters (applied via `apply_filters()`):

| Filter | Field | Behavior |
|--------|-------|----------|
| `--agent <name>` | `agent_filter` | Show only skills installed for the specified agent |
| `--managed` | `managed_only` | Show only managed skills (sikil-controlled) |
| `--unmanaged` | `unmanaged_only` | Show only unmanaged skills |
| `--conflicts` | `conflicts_only` | Show only skills with detected conflicts |
| `--duplicates` | `duplicates_only` | Alias for `--conflicts` - shows skills in conflict set |

Filters can be combined (e.g., `--agent claude-code --managed`).

## Cache Control

Both `list` and `show` commands support the `--no-cache` flag:

| Flag | Field | Behavior |
|------|-------|----------|
| `--no-cache` | `no_cache` | Bypass cache and force fresh scan of agent directories |

When enabled, commands use `Scanner::without_cache()` instead of `Scanner::new()`.

## Disabled Agent Warning

When the list command finds no skills and no filters are applied, it checks for disabled default agents using `get_disabled_default_agents()`:

1. Compares current config against `Config::default()` to find agents that are disabled but enabled by default
2. Displays a warning listing disabled agents with their global paths
3. Provides guidance: `sikil config set agents.<agent>.enabled true`

This behavior is suppressed in JSON mode.

## Output Formats

### Human-Readable (default)

**List command:**
- Header with summary: "Found N skill(s) (M managed, U unmanaged) - conflicts summary"
- Table with NAME, DESCRIPTION (truncated to 50 chars), AGENTS columns
- Color-coded rows: green for managed, yellow/warning for unmanaged
- Conflict details and numbered recommendations at bottom

**Show command:**
- Section-based display with name, description, version, author, license
- Managed status with canonical path
- Installations list with agent, scope, path, symlink info
- File structure indicators and size

### JSON (`--json` flag)

**List command output structure:**
```json
[{
  "name": "string",
  "directory_name": "string (optional, present when directory name differs from skill name)",
  "description": "string",
  "managed": boolean,
  "installations": [{
    "agent": "string",
    "scope": "global|workspace"
  }]
}]
```

**Show command output structure:**
```json
{
  "name": "string",
  "directory_name": "string (optional, present when directory name differs from skill name)",
  "description": "string",
  "version": "string (optional)",
  "author": "string (optional)",
  "license": "string (optional)",
  "managed": boolean,
  "canonical_path": "string (optional)",
  "installations": [{
    "agent": "string",
    "path": "string",
    "scope": "global|workspace",
    "is_symlink": boolean,
    "symlink_target": "string (optional)"
  }],
  "file_tree": {
    "has_skill_md": boolean,
    "has_scripts_dir": boolean,
    "has_references_dir": boolean,
    "file_count": number
  },
  "total_size_bytes": number
}
```

Optional fields use `#[serde(skip_serializing_if = "Option::is_none")]`.

## Summary Statistics

The list command displays:
- Total skill count
- Managed skill count
- Unmanaged skill count
- Conflicts summary (from `format_conflicts_summary()`)

## Dependencies

| Dependency | Usage |
|------------|-------|
| `crate::cli::output::Output` | Formatted printing (info, success, warning, JSON) |
| `crate::core::config::Config` | Agent configuration and paths |
| `crate::core::scanner::Scanner` | Scanning agent directories for skills |
| `crate::core::skill::{Agent, Scope, Skill}` | Skill data structures |
| `crate::core::conflicts` | Conflict detection and formatting |
| `crate::core::errors::SikilError` | Error types (SkillNotFound) |
| `anyhow::Result` | Error handling |
| `serde::Serialize` | JSON serialization for output structs |
| `fs-err` | File system operations in show command |

## Used By

- CLI parser routes `sikil list` to `execute_list()`
- CLI parser routes `sikil show <name>` to `execute_show()`
- Both commands share the `Scanner` for skill discovery
- Output structs are designed for scripting/automation via `--json`
