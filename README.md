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

### From Crates.io (Recommended)

```bash
cargo install sikil
```

### From Source

```bash
git clone https://github.com/user/sikil.git
cd sikil
cargo install --path .
```

### Pre-built Binaries

```bash
# Download the latest release
curl -L https://github.com/user/sikil/releases/latest/download/sikil-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv sikil /usr/local/bin/
```

**Requirements**: Rust 1.75+ if building from source

## Quick Start

```bash
# 1. See what skills exist across all agents
sikil list

# 2. Install a skill from GitHub to all agents
sikil install user/repo --to all

# 3. Or install from a local directory
sikil install ./my-skill --to claude-code,windsurf

# 4. Show details about a skill
sikil show git-workflow

# 5. Adopt an existing skill into management
sikil adopt git-workflow --from claude-code

# 6. Sync a managed skill to all agents
sikil sync git-workflow --to all

# 7. Validate a skill before installation
sikil validate ./my-skill
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

### `list` - List all skills

```bash
# List all skills across all agents
sikil list

# Filter by agent
sikil list --agent claude-code

# Show only managed/unmanaged skills
sikil list --managed
sikil list --unmanaged

# Show conflicts or duplicates
sikil list --conflicts
sikil list --duplicates

# JSON output
sikil list --json
```

### `show` - Show skill details

```bash
# Show skill details
sikil show git-workflow

# JSON output
sikil show git-workflow --json
```

### `install` - Install skills

```bash
# Install from GitHub (short form)
sikil install user/repo

# Install from GitHub with subdirectory
sikil install user/repo/path/to/skill

# Install from full HTTPS URL
sikil install https://github.com/user/repo.git

# Install from local directory
sikil install ./path/to/skill

# Install to specific agents
sikil install user/repo --to claude-code,windsurf

# Install to all enabled agents
sikil install user/repo --to all
```

### `adopt` - Adopt existing skills

```bash
# Adopt a skill (requires --from if multiple locations)
sikil adopt git-workflow

# Adopt from specific agent
sikil adopt git-workflow --from claude-code
```

### `unmanage` - Convert to unmanaged

```bash
# Unmanage from all agents
sikil unmanage git-workflow

# Unmanage from specific agent
sikil unmanage git-workflow --agent claude-code

# Skip confirmation
sikil unmanage git-workflow --yes
```

### `remove` - Remove skills

```bash
# Remove from specific agents
sikil remove git-workflow --agent claude-code,windsurf

# Remove from all agents (managed and unmanaged)
sikil remove git-workflow --all

# Skip confirmation
sikil remove git-workflow --all --yes
```

### `sync` - Sync skills across agents

```bash
# Sync specific skill to missing agents
sikil sync git-workflow

# Sync all managed skills
sikil sync --all

# Sync to specific agents
sikil sync git-workflow --to claude-code,windsurf
```

### `validate` - Validate skill structure

```bash
# Validate local skill
sikil validate ./my-skill

# Validate installed skill
sikil validate git-workflow

# JSON output
sikil validate ./my-skill --json
```

### `config` - Configuration management

```bash
# Show current configuration
sikil config

# Edit configuration file
sikil config --edit

# Set configuration values
sikil config set agents.claude-code.global_path "/custom/path"
sikil config set agents.windsurf.enabled false
```

### `completions` - Generate shell completions

```bash
# Generate bash completions
sikil completions bash

# Generate zsh completions
sikil completions zsh

# Generate fish completions
sikil completions fish

# Save to file
sikil completions bash --output ~/.local/share/bash-completion/completions/sikil
```

## JSON Output Schema

Several commands support structured JSON output with `--json` flag:

### `sikil list --json`

```json
{
  "skills": [
    {
      "name": "git-workflow",
      "description": "Git workflow automation skill",
      "version": "1.0.0",
      "author": "user",
      "installations": [
        {
          "agent": "claude-code",
          "path": "/home/user/.claude/skills/git-workflow",
          "scope": "global",
          "is_managed": true,
          "is_symlink": true,
          "symlink_target": "/home/user/.sikil/repo/git-workflow"
        }
      ]
    }
  ],
  "summary": {
    "total_skills": 1,
    "managed_skills": 1,
    "unmanaged_skills": 0,
    "conflicts": 0,
    "duplicates": 0
  }
}
```

### `sikil show <skill> --json`

```json
{
  "name": "git-workflow",
  "description": "Git workflow automation skill",
  "version": "1.0.0",
  "author": "user",
  "license": "MIT",
  "is_managed": true,
  "canonical_path": "/home/user/.sikil/repo/git-workflow",
  "total_size": 24576,
  "installations": [
    {
      "agent": "claude-code",
      "path": "/home/user/.claude/skills/git-workflow",
      "scope": "global",
      "is_managed": true,
      "is_symlink": true,
      "symlink_target": "/home/user/.sikil/repo/git-workflow"
    },
    {
      "agent": "windsurf",
      "path": "/home/user/project/.windsurf/skills/git-workflow",
      "scope": "workspace",
      "is_managed": true,
      "is_symlink": true,
      "symlink_target": "/home/user/.sikil/repo/git-workflow"
    }
  ],
  "file_tree": [
    "SKILL.md",
    "scripts/",
    "scripts/setup.sh",
    "references/",
    "references/git-commands.md"
  ]
}
```

### `sikil validate <path> --json`

```json
{
  "path": "/home/user/my-skill",
  "valid": true,
  "errors": [],
  "warnings": [
    {
      "code": "missing_optional_field",
      "message": "Optional field 'version' not found in frontmatter"
    }
  ],
  "checks": {
    "skill_md_exists": true,
    "frontmatter_valid": true,
    "required_fields_present": true,
    "name_format_valid": true,
    "description_length_valid": true
  },
  "detected_structure": {
    "has_scripts_dir": true,
    "has_references_dir": false,
    "file_count": 5
  }
}
```

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

## Performance

Sikil is designed for speed with built-in caching and efficient filesystem operations:

| Operation | Target | Typical Performance |
|-----------|--------|---------------------|
| `sikil list` (50 skills, uncached) | <500ms | ~20-30ms |
| `sikil list` (50 skills, cached) | <100ms | ~40-50ms |
| `sikil show` | <200ms | ~10ms |
| `sikil validate` | <100ms | ~7ms |

**Caching**: Sikil uses SQLite caching to avoid redundant filesystem scans. Cache is automatically invalidated based on file modification times and sizes. Use `--no-cache` to bypass cache for any command.

**Optimization Tips**:
- First run after installation will be slower due to cache population
- Cached operations are significantly faster for large skill sets
- JSON output has minimal overhead compared to human-readable output

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
