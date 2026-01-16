# Sikil

> A CLI tool for managing Agent Skills across multiple AI coding agents.

**Sikil** provides unified discovery, installation, synchronization, and management of Agent Skills across multiple AI coding assistants.

## The Problem

Developers using multiple AI coding agents (Claude Code, Windsurf, OpenCode, Kilo Code, Amp) face a management nightmare: each agent stores skills in different directories with incompatible path structures. This leads to:

- Duplicate skills across 5+ agent installations
- Version mismatches between agents
- No unified visibility of what's installed where
- Manual cross-agent installation overhead

## Core Features

| Feature | Description |
|---------|-------------|
| **Discovery** | Scan all skills across all agent installations |
| **Installation** | Install from local path or Git to multiple agents at once |
| **Synchronization** | Sync skills across agents with one command |
| **Adoption** | Bring existing unmanaged skills under centralized management |
| **Validation** | Verify skill structure before installation |
| **Conflict Detection** | Detect duplicates and version mismatches |

## Installation

```bash
# Via cargo (coming soon)
cargo install sikil

# Or download pre-built binary
curl -sSf https://raw.githubusercontent.com/user/sikil/main/install.sh | sh
```

## Quick Start

```bash
# See what skills exist across all agents
sikil list

# Install a skill from GitHub to all agents
sikil install github.com/user/skills/my-skill --to all

# Adopt an existing skill into management
sikil adopt git-workflow --from claude-code

# Sync a managed skill to all agents
sikil sync git-workflow
```

## Architecture

Sikil is built as a **single binary CLI tool** in Rust, organized into four distinct layers:

```
┌─────────────────────────────────────┐
│         CLI Layer (clap)            │  Argument parsing, output formatting
├─────────────────────────────────────┤
│        Commands Layer               │  Orchestration, error handling
├─────────────────────────────────────┤
│         Core Layer                  │  Domain logic, types, scanner
├─────────────────────────────────────┤
│        Utils Layer                  │  Filesystem, symlinks, git, atomic ops
└─────────────────────────────────────┘
```

### Key Components

- **Scanner**: Discovers skills across all agent directories (global + workspace)
- **Parser**: Extracts metadata from SKILL.md YAML frontmatter
- **Planner**: Validates operations before execution (atomic with rollback)
- **Symlink Manager**: Creates/managed symlinks from agent dirs to central repo

### Directory Structure

```
~/.sikil/
├── repo/              # Managed skills (canonical copies)
├── config.toml        # Agent path configuration
└── cache.db           # SQLite cache for fast scans
```

Agent directories contain symlinks pointing to `~/.sikil/repo/`:

```
~/.claude/skills/git-workflow  →  ~/.sikil/repo/git-workflow/
~/.codeium/windsurf/skills/git-workflow  →  ~/.sikil/repo/git-workflow/
```

## Commands

| Command | Description |
|---------|-------------|
| `list` | List all skills across agents |
| `show <name>` | Show details for a specific skill |
| `install <source>` | Install from local path or Git |
| `adopt <name>` | Adopt unmanaged skill into management |
| `unmanage <name>` | Convert managed to unmanaged |
| `remove <name>` | Remove a skill from agents |
| `sync <name>` | Sync managed skill to all agents |
| `validate <path>` | Validate skill structure |
| `config` | View or modify configuration |

## Supported Agents

| Agent | Global Path | Workspace Path |
|-------|-------------|----------------|
| Claude Code | `~/.claude/skills/` | `.claude/skills/` |
| Windsurf | `~/.codeium/windsurf/skills/` | `.windsurf/skills/` |
| OpenCode | `~/.config/opencode/skill/` | `.opencode/skill/` |
| Kilo Code | `~/.kilocode/skills/` | `.kilocode/skills/` |
| Amp | `~/.config/agents/skills/` | `.agents/skills/` |

## Technology Stack

- **Language**: Rust 2021 edition (1.75+)
- **CLI Framework**: clap 4
- **Serialization**: serde + serde_yaml
- **Caching**: rusqlite (SQLite)
- **Platforms**: macOS (Intel + ARM), Linux (x86_64 + aarch64)

## Security

- **No script execution**: Skills are read-only configuration
- **Path traversal prevention**: Name validation + path canonicalization
- **Symlink rejection**: Skills cannot contain symlinks
- **GitHub-only Git URLs**: Only HTTPS GitHub URLs in v1.0

## Documentation

- [Product Requirements Document](plan/PRD.md)
- [Technical Requirements Document](plan/TRD.md)

## License

MIT

## Status

**v0.1.0** - MVP development in progress

---

*Sikil - turn 2-4 hours of manual skill management into < 5 minutes of automated workflows.*
