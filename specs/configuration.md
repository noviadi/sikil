# Configuration Spec

## One-Sentence Description

Configuration manages user preferences and agent path overrides stored in TOML format.

## Overview

The configuration system provides persistent storage for agent-specific settings including enabled status and custom installation paths. It uses TOML format with sensible defaults for all supported agents, allowing users to override paths or disable specific agents.

## Config File Location

- Path: `~/.sikil/config.toml`
- Resolved via `get_config_path()` in `src/utils/paths.rs`
- Directory created automatically when editing or setting values

## Config Structure

```toml
[agents.<agent-name>]
enabled = true|false
global_path = "<path>"
workspace_path = "<path>"
```

### AgentConfig Fields

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | bool | Whether this agent is active |
| `global_path` | PathBuf | Global installation path (supports `~` expansion) |
| `workspace_path` | PathBuf | Workspace-relative installation path |

### Default Agents

| Agent | Global Path | Workspace Path |
|-------|-------------|----------------|
| claude-code | `~/.claude/skills` | `.claude/skills` |
| windsurf | `~/.codeium/windsurf/skills` | `.windsurf/skills` |
| opencode | `~/.config/opencode/skill` | `.opencode/skill` |
| kilocode | `~/.kilocode/skills` | `.kilocode/skills` |
| amp | `~/.config/agents/skills` | `.agents/skills` |

## Loading Behavior

1. If config file does not exist → returns `Config::default()` with all agents enabled
2. If file exists → parses TOML and returns the loaded config (does NOT merge with defaults)
3. Path expansion (`~` to home) performed via `expand_paths()` using `shellexpand::tilde`
4. File size limit: 1MB maximum

## Config Commands

### Show (`sikil config`)

- Displays current configuration with default/custom indicators
- Compares loaded config against defaults to show `(default)` or `(custom)` markers
- Supports `--json` flag for machine-readable output

### Edit (`sikil config --edit`)

- Creates default config file if missing
- Opens config in `$EDITOR` (falls back to `vi`)
- Validates config after editing

### Set (`sikil config --set <key> <value>`)

- Key format: `agents.<agent>.<field>`
- Valid fields: `enabled`, `global_path`, `workspace_path`
- Creates config file with defaults if missing before applying change

## Validation

- `#[serde(deny_unknown_fields)]` rejects unknown TOML keys
- File size must be ≤1MB (`ConfigError::ConfigTooLarge`)
- Invalid TOML returns `ConfigError::InvalidToml`
- Post-edit validation ensures config remains parseable
- Set command validates key format and field names

## Acceptance Criteria

- Missing config file returns `Config::default()` with all agents enabled
- Existing config file is parsed and returned without merging with defaults
- Config file exceeding 1MB returns `ConfigError::ConfigTooLarge`
- Unknown TOML keys return `ConfigError::InvalidToml`
- Tilde (`~`) in paths is expanded to home directory
- `sikil config` displays `(default)` or `(custom)` markers for each setting
- `sikil config --json` outputs configuration as valid JSON to stdout
- `sikil config --edit` creates default config if file is missing
- `sikil config --edit` opens `$EDITOR` (or `vi` if unset)
- `sikil config --edit` validates config after editor closes
- `sikil config --set agents.<agent>.enabled false` disables the specified agent
- `sikil config --set` with invalid key format prints error and exits non-zero
- `sikil config --set` creates config file with defaults before applying change if missing

## Dependencies

- `toml` - TOML parsing and serialization
- `shellexpand` - Path tilde expansion
- `serde` - Serialization/deserialization
- `directories` - Home directory resolution
- `crate::core::errors::ConfigError` - Error types
- `crate::utils::paths::get_config_path` - Path resolution

## Used By

- `src/commands/config.rs` - Config CLI commands
- Other commands that need agent paths for skill installation/scanning
