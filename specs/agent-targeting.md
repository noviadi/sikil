# Agent Targeting Spec

## One-Sentence Description

Agent targeting selects enabled agents for multi-agent operations.

## Overview

The agent targeting module parses and validates agent selections from command-line arguments, supporting the `--to` flag with comma-separated lists, the "all" keyword, and interactive prompts when no selection is provided. It ensures only enabled agents from configuration are targeted.

## Implementation

Location: `src/commands/agent_selection.rs`

### `parse_agent_selection()`

Parses the `--to` flag value and returns a vector of validated `Agent` values.

| Input | Behavior |
|-------|----------|
| `None` | Returns empty vec (triggers interactive prompt) |
| `"all"` (case-insensitive) | Returns all enabled agents from config |
| Single agent name | Validates and returns single agent |
| Comma-separated list | Parses, trims whitespace, validates each agent |

**Validation steps:**
1. Parse agent name via `Agent::from_cli_name()`
2. Check agent exists in config
3. Verify agent is enabled (not disabled)
4. Reject if final list is empty

### `prompt_agent_selection()`

Interactive terminal prompt for agent selection when `--to` is not provided.

**Display format:**
```
Select agents to install to:
  1. claude-code
  2. windsurf
  3. amp
  a. All enabled agents

Enter selection (e.g., '1', '1,2', 'a'):
```

**Input handling:**
| Input | Result |
|-------|--------|
| `a` | All enabled agents |
| `1` | First agent in list |
| `1,2,3` | Multiple agents by index |
| Empty | Error: "no selection made" |
| Invalid number | Error: "invalid selection format" |
| Out of range | Error: "invalid selection N" |

### `list_valid_agents()`

Helper function that returns comma-separated string of enabled agent names for error messages.

Used in: Unknown agent error message to show valid options.

### `get_all_enabled_agents()`

Private helper that filters config for enabled agents.

Returns error if no agents are enabled: "no enabled agents in configuration".

## Error Messages

| Condition | Error Type | Message |
|-----------|------------|---------|
| Unknown agent name | `ValidationError` | "unknown agent 'X'. Valid agents: ..." |
| Disabled agent | `ValidationError` | "agent 'X' is disabled in configuration" |
| Empty selection | `ValidationError` | "no valid agents specified" |
| No enabled agents | `ValidationError` | "no enabled agents in configuration" |
| Invalid prompt input | `ValidationError` | "invalid selection format..." |
| Selection out of range | `ValidationError` | "invalid selection N. Must be between 1 and M" |
| Empty prompt input | `ValidationError` | "no selection made" |

## JSON Mode Behavior

When `--json` flag is active:
- Interactive prompt is skipped entirely
- Commands default to all enabled agents if `--to` not specified
- Callers use `parse_agent_selection(Some("all"), config)` directly

This ensures non-interactive operation for scripting and automation.

## Dependencies

| Component | Location | Purpose |
|-----------|----------|---------|
| `Config` | `src/core/config.rs` | Read agent configurations and enabled status |
| `Agent` | `src/core/skill.rs` | Agent enum with `from_cli_name()` method |
| `SikilError` | `src/core/errors.rs` | Error types for validation failures |

## Used By

| Command | Usage |
|---------|-------|
| `sikil install` | Determine target agents for symlink creation |
| `sikil sync` | Select agents for synchronization |

## Example Usage

```rust
// Parse explicit selection
let agents = parse_agent_selection(Some("claude-code,amp"), &config)?;

// Use all enabled agents (JSON mode)
let agents = parse_agent_selection(Some("all"), &config)?;

// Trigger interactive prompt
let agents = if to_flag.is_some() {
    parse_agent_selection(to_flag, &config)?
} else if json_mode {
    parse_agent_selection(Some("all"), &config)?
} else {
    prompt_agent_selection(&config)?
};
```
