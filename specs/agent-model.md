# Agent Model Spec

## One-Sentence Description

The agent model defines supported AI coding agents.

## Overview

The agent model is implemented primarily through the `Agent` enum in `src/core/skill.rs` and the `AgentConfig` struct in `src/core/config.rs`. It provides a type-safe representation of supported AI coding agents, their CLI names, and configurable paths for global and workspace-scoped skill installations.

## Supported Agents

| Agent       | CLI Name      | Description            |
|-------------|---------------|------------------------|
| ClaudeCode  | `claude-code` | Claude Code (Anthropic)|
| Windsurf    | `windsurf`    | Windsurf (Codeium)     |
| OpenCode    | `opencode`    | OpenCode               |
| KiloCode    | `kilocode`    | KiloCode               |
| Amp         | `amp`         | Amp                    |

The `Agent::all()` method returns all variants as a static slice for iteration.

## Path Configurations

Each agent has configurable global and workspace paths via `AgentConfig`:

| Agent       | Global Path                     | Workspace Path       |
|-------------|--------------------------------|----------------------|
| claude-code | `~/.claude/skills`             | `.claude/skills`     |
| windsurf    | `~/.codeium/windsurf/skills`   | `.windsurf/skills`   |
| opencode    | `~/.config/opencode/skill`     | `.opencode/skill`    |
| kilocode    | `~/.kilocode/skills`           | `.kilocode/skills`   |
| amp         | `~/.config/agents/skills`      | `.agents/skills`     |

- Paths support tilde (`~`) expansion via `shellexpand`
- Custom paths can be configured in `~/.sikil/config.toml`
- The `enabled` flag allows disabling specific agents

## Scopes

The `Scope` enum defines two installation scopes:

| Scope     | Description                                    | Path Type       |
|-----------|------------------------------------------------|-----------------|
| `Global`  | User-wide installation (e.g., `~/.claude/skills/`) | Absolute        |
| `Workspace` | Project-local installation (e.g., `.claude/skills/`) | Relative to workspace |

## Agent Selection

- **Parsing**: `Agent::from_cli_name(&str) -> Option<Agent>` parses CLI names
- **Display**: `Agent::cli_name() -> &str` and `Display` trait for output
- **Filtering**: `AgentConfig.enabled` controls whether an agent is included in operations
- **Iteration**: `Agent::all()` provides all variants; config filtering uses `config.agents.values().filter(|a| a.enabled)`

## Dependencies

- `serde`: Serialization with `rename_all = "kebab-case"` for Agent enum
- `shellexpand`: Path expansion for `~` and environment variables
- `directories`: Home directory resolution in `paths.rs`

## Used By

| Consumer              | Usage                                      |
|-----------------------|--------------------------------------------|
| `Installation` struct | Associates installations with specific agents |
| `Scanner`             | Scans agent paths for installed skills     |
| CLI commands          | `--agent` flag filtering                   |
| Config loading        | Maps agent names to path configurations    |
