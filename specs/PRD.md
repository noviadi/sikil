# Sikil - Product Requirements Document

**Version**: 1.1  
**Created**: January 16, 2026  
**Status**: Approved for Implementation  
**Author**: Product Team

> **v1.1 Update**: Added security hardening requirements (NFR-23 to NFR-26) based on technical review.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Problem Statement](#problem-statement)
3. [Goals & Objectives](#goals--objectives)
4. [Target Users](#target-users)
5. [Scope](#scope)
6. [Functional Requirements](#functional-requirements)
7. [Non-Functional Requirements](#non-functional-requirements)
8. [Technical Architecture](#technical-architecture)
9. [User Experience](#user-experience)
10. [Success Metrics](#success-metrics)
11. [Timeline & Milestones](#timeline--milestones)
12. [Risks & Mitigations](#risks--mitigations)
13. [Dependencies](#dependencies)
14. [Appendix](#appendix)

---

## Executive Summary

**Sikil** is a command-line tool for managing Agent Skills across multiple AI coding agents. It provides unified discovery, installation, synchronization, and management of skills that are currently fragmented across 5+ different agent installations.

### The Problem
Developers using multiple AI coding agents (Claude Code, Windsurf, OpenCode, Kilo Code, Amp) face a management nightmare: each agent stores skills in different directories with incompatible path structures. This leads to duplicate skills, version mismatches, maintenance overhead, and lack of visibility.

### The Solution
A CLI-first tool that:
- **Discovers** all skills across all agent installations
- **Manages** them through a central repository with symlinks
- **Synchronizes** skills across agents with one command
- **Validates** skill structure and detects conflicts

### Key Value Proposition
Turn 2-4 hours of manual skill management into < 5 minutes of automated workflows.

---

## Problem Statement

### Current State

Developers using multiple AI coding agents experience:

```
Typical developer's machine:
├── ~/.claude/skills/           (13 skills)
├── ~/.codeium/windsurf/skills/ (18 skills) ← Many duplicates
├── ~/.config/opencode/skill/   (9 skills)  ← Version conflicts
├── ~/.kilocode/skills/         (8 skills)  
├── ~/.config/agents/skills/    (5 skills)  
├── Project-1/.claude/skills/   (4 skills)  ← Which is current?
├── Project-2/.opencode/skill/  (3 skills)
└── ... 10+ more project directories

Result: 47+ skills across 15+ directories with no unified management
```

### Pain Points

| Pain Point | Impact | Frequency |
|------------|--------|-----------|
| **No unified visibility** | Can't see what skills exist where | Daily |
| **Duplicate installations** | Wasted disk space, inconsistent behavior | Weekly |
| **Version mismatches** | Same skill, different versions across agents | Weekly |
| **Manual cross-agent install** | 5+ manual steps per agent per skill | Per install |
| **No validation** | Invalid skills fail silently | Per creation |
| **Maintenance burden** | Updates require touching multiple directories | Monthly |
| **Onboarding friction** | New developers spend hours on setup | Per hire |

### Root Cause
Each agent implements the Agent Skills specification independently with different:
- Directory structures (`~/.claude/skills/` vs `~/.codeium/windsurf/skills/`)
- Priority systems (project vs global vs managed)
- Configuration mechanisms
- No cross-agent coordination tools

---

## Goals & Objectives

### Primary Goals

| Goal | Metric | Target |
|------|--------|--------|
| **Unified Visibility** | Time to see all installed skills | < 2 seconds |
| **Simplified Installation** | Steps to install across all agents | 1 command |
| **Conflict Detection** | Duplicate/mismatch detection rate | 100% |
| **Cross-Agent Sync** | Time to sync skill to all agents | < 5 seconds |

### Secondary Goals

| Goal | Metric | Target |
|------|--------|--------|
| **Validation** | Invalid skill detection before install | 100% |
| **Adoption** | Existing skills adoptable into management | Yes |
| **Reversibility** | Ability to unmanage skills | Yes |
| **Scriptability** | JSON output for automation | All commands |

### Non-Goals (Explicit Exclusions)

- Version comparison and upgrade recommendations
- Marketplace/registry integration
- Skill dependency resolution
- Windows support (v1.0)
- Hot-reload triggering in agents
- Usage analytics/telemetry
- TUI (Terminal UI) interface
- Enterprise/team management features

---

## Target Users

### Primary Persona: Multi-Agent Developer

**Profile:**
- Uses 2+ AI coding agents daily
- Has 10-50+ skills installed
- Experienced with command-line tools
- Values automation and efficiency

**Jobs to be Done:**
1. See what skills I have and where
2. Install a skill to all my agents at once
3. Keep skills consistent across agents
4. Clean up duplicate and outdated skills
5. Validate skills before sharing with team

### Secondary Persona: Team Lead / DevOps

**Profile:**
- Manages development environment standards
- Onboards new team members
- Maintains team skill repositories

**Jobs to be Done:**
1. Standardize skills across team machines
2. Script skill installation in onboarding
3. Audit skill installations
4. Share approved skills with team

---

## Scope

### In Scope (v1.0)

| Category | Features |
|----------|----------|
| **Discovery** | Scan all 5 agents (global + workspace), list skills, show details |
| **Management** | Install (local + git), adopt, unmanage, remove |
| **Synchronization** | Sync skills to missing agents |
| **Validation** | SKILL.md format validation |
| **Conflict Detection** | Duplicate detection, report-only resolution |
| **Configuration** | Agent path overrides, enable/disable agents |
| **Output** | Human-readable + JSON formats |
| **Platforms** | macOS (Intel + Apple Silicon), Linux (x86_64 + aarch64) |

### Out of Scope (v1.0)

| Feature | Rationale | Future Version |
|---------|-----------|----------------|
| Version comparison | Complexity, undefined in spec | v1.1 |
| Marketplace integration | No standard exists | v2.0 |
| Skill dependencies | Not in agent skills spec | v2.0 |
| Windows support | Platform abstraction needed | v1.1 |
| TUI interface | CLI-first approach | v1.2 |
| Usage analytics | Privacy concerns | TBD |
| Team/enterprise features | Scope creep | v2.0 |
| Kilo Code mode-specific dirs | Low usage | v1.1 |

---

## Functional Requirements

### FR-01: Discovery & Listing

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01.1 | Scan all 5 agent global paths | P0 |
| FR-01.2 | Scan workspace paths relative to CWD | P0 |
| FR-01.3 | Parse SKILL.md YAML frontmatter | P0 |
| FR-01.4 | Classify skills as managed/unmanaged | P0 |
| FR-01.5 | Detect broken symlinks | P1 |
| FR-01.6 | Filter by agent, managed status | P1 |
| FR-01.7 | Display summary with counts | P1 |

**Commands:** `sikil list`, `sikil list --agent`, `sikil list --managed`

### FR-02: Skill Details

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-02.1 | Display metadata (name, description, version, author) | P0 |
| FR-02.2 | Show all installation locations | P0 |
| FR-02.3 | Display file tree within skill | P1 |
| FR-02.4 | Show total size on disk | P2 |

**Commands:** `sikil show <name>`

### FR-03: Installation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-03.1 | Install from local directory | P0 |
| FR-03.2 | Install from Git URL (GitHub shorthand, HTTPS, SSH) | P0 |
| FR-03.3 | Support subdirectory extraction from Git repos | P0 |
| FR-03.4 | Copy to `~/.sikil/repo/` and create symlinks | P0 |
| FR-03.5 | Validate skill before installation | P0 |
| FR-03.6 | Select target agents with `--to` flag | P0 |
| FR-03.7 | Create agent directories if missing | P1 |
| FR-03.8 | Fail gracefully if skill exists (no overwrite) | P0 |

**Commands:** `sikil install <source> --to <agents>`

### FR-04: Adoption

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-04.1 | Move unmanaged skill to managed repo | P0 |
| FR-04.2 | Replace original with symlink | P0 |
| FR-04.3 | Require `--from` if multiple locations | P0 |
| FR-04.4 | Atomic operation with rollback | P1 |

**Commands:** `sikil adopt <name> --from <agent>`

### FR-05: Unmanagement

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-05.1 | Copy from repo back to agent locations | P0 |
| FR-05.2 | Remove symlinks, create physical copies | P0 |
| FR-05.3 | Support per-agent or all-agents unmanage | P0 |
| FR-05.4 | Require confirmation (bypass with `--yes`) | P0 |

**Commands:** `sikil unmanage <name> [--agent <agent>] [--yes]`

### FR-06: Removal

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-06.1 | Remove from specific agent(s) | P0 |
| FR-06.2 | Remove from all agents + repo with `--all` | P0 |
| FR-06.3 | Support managed and unmanaged skills | P0 |
| FR-06.4 | Require confirmation (bypass with `--yes`) | P0 |

**Commands:** `sikil remove <name> --agent <agent>`, `sikil remove <name> --all`

### FR-07: Synchronization

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-07.1 | Sync skill to agents that don't have it | P0 |
| FR-07.2 | Sync all managed skills with `--all` | P0 |
| FR-07.3 | Limit sync to specific agents with `--to` | P1 |
| FR-07.4 | Fail if unmanaged skill blocks (suggest adopt) | P0 |

**Commands:** `sikil sync <name>`, `sikil sync --all`

### FR-08: Validation

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-08.1 | Validate SKILL.md exists | P0 |
| FR-08.2 | Validate YAML frontmatter syntax | P0 |
| FR-08.3 | Validate required fields (name, description) | P0 |
| FR-08.4 | Validate name format constraints | P0 |
| FR-08.5 | Warn on missing optional fields | P1 |
| FR-08.6 | Exit code 0 on pass, non-zero on fail | P0 |

**Commands:** `sikil validate <path>`

### FR-09: Conflict Detection

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-09.1 | Detect duplicate-unmanaged (same name, multiple physical) | P0 |
| FR-09.2 | Detect duplicate-managed (same name, same realpath) | P1 |
| FR-09.3 | Report conflicts with recommendations | P0 |
| FR-09.4 | Filter with `--conflicts` and `--duplicates` | P1 |

**Commands:** `sikil list --conflicts`

### FR-10: Configuration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-10.1 | Config file at `~/.sikil/config.toml` | P0 |
| FR-10.2 | Override agent global/workspace paths | P0 |
| FR-10.3 | Enable/disable specific agents | P1 |
| FR-10.4 | Display current config | P1 |
| FR-10.5 | Edit config with `--edit` | P2 |
| FR-10.6 | Set individual values with `config set` | P1 |

**Commands:** `sikil config`, `sikil config set <key> <value>`

### FR-11: Output Formats

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-11.1 | Human-readable colored output (default) | P0 |
| FR-11.2 | JSON output with `--json` flag | P0 |
| FR-11.3 | Respect `NO_COLOR` environment variable | P1 |

### FR-12: Shell Completions

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-12.1 | Generate bash completions | P1 |
| FR-12.2 | Generate zsh completions | P1 |
| FR-12.3 | Generate fish completions | P2 |

**Commands:** `sikil completions <shell>`

---

## Non-Functional Requirements

### Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | `sikil list` response time | < 500ms (50 skills) |
| NFR-02 | Cached scan response time | < 100ms |
| NFR-03 | Cache invalidation | Based on mtime |
| NFR-04 | Binary size | < 10MB |

### Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-05 | Destructive ops require confirmation | 100% |
| NFR-06 | Failed ops restore original state | Atomic rollback |
| NFR-07 | Graceful permission error handling | No crashes |
| NFR-08 | Graceful missing directory handling | No crashes |
| NFR-09 | Broken symlink detection | Warning, not error |

### Compatibility

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-10 | macOS Intel support | Full |
| NFR-11 | macOS Apple Silicon support | Full |
| NFR-12 | Linux x86_64 support | Full |
| NFR-13 | Linux aarch64 support | Full |
| NFR-14 | Windows support | Explicitly unsupported |
| NFR-15 | Minimum Git version | 2.0+ |

### Usability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-16 | Colored terminal output | ANSI colors |
| NFR-17 | Clear error messages | Actionable guidance |
| NFR-18 | Progress indicators | Long operations |
| NFR-19 | First-time user success | < 5 minutes |

### Security

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-20 | No execution of skill scripts | Read-only |
| NFR-21 | Path traversal prevention | Skill name validation, path canonicalization |
| NFR-22 | Git clone over HTTPS | GitHub-only in v1, TLS enforced |
| NFR-23 | Symlink rejection in skills | Error if skill contains symlinks |
| NFR-24 | Git URL validation | Reject dangerous formats (file://, `-` prefix) |
| NFR-25 | Config file hardening | Size limit, reject unknown fields |
| NFR-26 | Database crash safety | SQLite WAL mode enabled |

---

## Technical Architecture

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Language** | Rust | Performance, single binary, cross-platform |
| **CLI Framework** | clap 4 | Industry standard, derive macros |
| **Serialization** | serde + serde_yaml + serde_json | Standard Rust ecosystem |
| **File System** | walkdir, fs-err | Robust traversal, better errors |
| **Symlinks** | std::os::unix::fs | Unix-native operations |
| **Git** | Shell out to `git` | Best auth/credential support |
| **Caching** | rusqlite (bundled) | Reliable, embedded |
| **Terminal** | anstream + anstyle | Modern color handling |
| **Testing** | assert_cmd, insta | CLI testing, snapshots |

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                           │
│  ┌─────────┐ ┌──────┐ ┌─────────┐ ┌──────┐ ┌──────────────┐│
│  │  list   │ │ show │ │ install │ │ sync │ │    ...       ││
│  └────┬────┘ └───┬──┘ └────┬────┘ └───┬──┘ └──────────────┘│
└───────┼──────────┼─────────┼──────────┼────────────────────┘
        │          │         │          │
┌───────▼──────────▼─────────▼──────────▼────────────────────┐
│                      Commands Layer                         │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  execute_list() | execute_show() | execute_install()   ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                        Core Layer                           │
│  ┌──────────┐ ┌────────┐ ┌──────────┐ ┌───────────────────┐│
│  │ Scanner  │ │ Parser │ │ Config   │ │ Conflict Detector ││
│  └──────────┘ └────────┘ └──────────┘ └───────────────────┘│
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                       Utils Layer                           │
│  ┌──────────┐ ┌──────────┐ ┌────────┐ ┌───────────────────┐│
│  │  Paths   │ │ Symlinks │ │ Atomic │ │       Git         ││
│  └──────────┘ └──────────┘ └────────┘ └───────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Data Model

```rust
// Core types
struct SkillMetadata {
    name: String,           // Required, primary identity
    description: String,    // Required
    version: Option<String>,
    author: Option<String>,
    license: Option<String>,
}

struct Skill {
    metadata: SkillMetadata,
    installations: Vec<Installation>,
    is_managed: bool,
}

struct Installation {
    agent: Agent,
    path: PathBuf,
    scope: Scope,           // Global or Workspace
    is_symlink: bool,
    symlink_target: Option<PathBuf>,
}

enum Agent {
    ClaudeCode,
    Windsurf,
    OpenCode,
    KiloCode,
    Amp,
}
```

### Directory Structure

```
~/.sikil/
├── repo/                    # Managed skills (canonical copies)
│   ├── git-workflow/
│   │   ├── SKILL.md
│   │   ├── scripts/
│   │   └── references/
│   └── code-review/
│       └── SKILL.md
├── config.toml              # User configuration
└── cache.db                 # SQLite cache (optional)

Agent Directories (symlinks point here):
~/.claude/skills/git-workflow → ~/.sikil/repo/git-workflow
~/.codeium/windsurf/skills/git-workflow → ~/.sikil/repo/git-workflow
~/.config/opencode/skill/git-workflow → ~/.sikil/repo/git-workflow
```

---

## User Experience

### Command Structure

```
sikil <command> [options] [arguments]

Commands:
  list        List all skills across agents
  show        Show details for a specific skill
  install     Install a skill from local path or Git
  adopt       Adopt an unmanaged skill into management
  unmanage    Convert managed skill to unmanaged
  remove      Remove a skill from agents
  sync        Sync a managed skill to all agents
  validate    Validate a skill's structure
  config      View or modify configuration
  completions Generate shell completions

Global Options:
  --json      Output in JSON format
  --verbose   Show detailed output
  --quiet     Suppress non-essential output
  --version   Show version
  --help      Show help
```

### Example Workflows

**Workflow 1: First-time setup**
```bash
# See what skills exist
$ sikil list
Found 15 skills (0 managed, 15 unmanaged) across 3 agents

# Adopt a skill you want to manage
$ sikil adopt git-workflow --from claude-code
✓ Adopted 'git-workflow' to managed repo
✓ Created symlink at ~/.claude/skills/git-workflow

# Sync to all other agents
$ sikil sync git-workflow
✓ Synced to windsurf, opencode, kilo-code, amp
```

**Workflow 2: Install new skill**
```bash
# Install from GitHub to all agents
$ sikil install github.com/team/skills/new-skill --to all
✓ Installed 'new-skill' to 5 agents
```

**Workflow 3: Clean up duplicates**
```bash
# Find conflicts
$ sikil list --conflicts
Found 3 duplicate-unmanaged conflicts

# Adopt and consolidate
$ sikil adopt code-review --from claude-code
$ sikil sync code-review
$ sikil remove code-review --agent windsurf  # Remove old unmanaged copy
```

### Error Handling

All errors include:
1. What went wrong (clear description)
2. Why it happened (context)
3. How to fix it (actionable suggestion)

Example:
```
Error: Cannot install 'git-workflow' - skill already exists

The skill 'git-workflow' already exists in the managed repository.
Location: ~/.sikil/repo/git-workflow/

To update the skill, first remove it:
  sikil remove git-workflow --all

Or use a different name when installing.
```

---

## Success Metrics

### Launch Metrics (v1.0)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Binary builds on all platforms | 4/4 | CI |
| All commands functional | 10/10 | Integration tests |
| Test coverage | > 80% | cargo-tarpaulin |
| Documentation complete | 100% | Review |

### Adoption Metrics (3 months post-launch)

| Metric | Target | Measurement |
|--------|--------|-------------|
| GitHub stars | 100+ | GitHub |
| Downloads | 500+ | Release downloads |
| Issues resolved | > 90% | GitHub issues |
| User feedback score | > 4/5 | Survey |

### Usage Metrics (if telemetry added later)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Daily active users | 100+ | Telemetry |
| Commands per session | > 3 | Telemetry |
| Error rate | < 5% | Telemetry |

---

## Timeline & Milestones

### Overview

| Milestone | Duration | Goal |
|-----------|----------|------|
| M1: Foundation | 2 weeks | Project setup, core types, infrastructure |
| M2: Discovery | 2 weeks | Scanning, list, show, validate commands |
| M3: Management | 2 weeks | Install, adopt, unmanage, remove commands |
| M4: Sync & Config | 1 week | Sync command, configuration |
| M5: Polish | 1 week | Completions, docs, release |

**Total: 8 weeks**

### Detailed Timeline

```
Week 1-2: Foundation (M1)
├── Project setup, dependencies
├── Core types (Skill, Installation, Agent)
├── Configuration system
├── SKILL.md parser
├── Filesystem utilities
└── CLI framework

Week 3-4: Discovery (M2)
├── Directory scanner
├── List command with filters
├── Show command
├── Validate command
└── Conflict detection

Week 5-6: Management (M3)
├── Install from local path
├── Install from Git
├── Adopt command
├── Unmanage command
└── Remove command

Week 7: Sync & Config (M4)
├── Sync command
└── Config command

Week 8: Polish (M5)
├── Shell completions
├── Documentation
├── Build/release scripts
└── Final testing
```

### Release Plan

| Version | Scope | Timeline |
|---------|-------|----------|
| v0.1.0 | MVP (all features) | Week 8 |
| v0.1.x | Bug fixes, polish | Ongoing |
| v1.0.0 | Stable release | Week 10 |
| v1.1.0 | Windows support, version comparison | TBD |

---

## Risks & Mitigations

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agent path changes | Medium | Medium | Config-based paths, regular updates |
| Git auth complexity | Medium | Medium | Shell out to git, leverage user config |
| Symlink issues on network drives | Low | Medium | Document limitation, detect and warn |
| Large skill directories slow scan | Low | Low | Caching, incremental scan |

### Product Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Low adoption | Medium | High | Clear value prop, good docs, community |
| Agent skills spec changes | Low | Medium | Modular parser, versioned support |
| Competition from agent vendors | Medium | Medium | Cross-agent value, open source |

### Operational Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking changes needed | Medium | Medium | Semantic versioning, deprecation policy |
| Maintenance burden | Medium | Medium | Comprehensive tests, good architecture |

---

## Dependencies

### External Dependencies

| Dependency | Version | Purpose | Risk |
|------------|---------|---------|------|
| Git | 2.0+ | Clone repositories | Low (widely installed) |
| SKILL.md spec | Current | Skill format | Low (stable spec) |

### Internal Dependencies (Crates)

| Crate | Version | Purpose |
|-------|---------|---------|
| clap | 4.x | CLI parsing |
| serde | 1.x | Serialization |
| serde_yaml | 0.9.x | YAML parsing |
| serde_json | 1.x | JSON output |
| walkdir | 2.x | Directory traversal |
| rusqlite | 0.31.x | Caching |
| anyhow | 1.x | Error handling |
| thiserror | 1.x | Error types |
| tempfile | 3.x | Atomic operations |

---

## Appendix

### Related Documents

| Document | Description |
|----------|-------------|
| [use_cases.md](use_cases.md) | Detailed use cases with acceptance criteria |
| [implementation_roadmap.md](implementation_roadmap.md) | Epic/task/subtask breakdown |
| [traceability_matrix.md](traceability_matrix.md) | Requirements to implementation mapping |
| [agent_skills_research.md](research_archive/agent_skills_research.md) | Background research on agent skills |
| [agent_skills_comparison_matrix.md](research_archive/agent_skills_comparison_matrix.md) | Agent-by-agent comparison |
| [agent_skills_technical_spec.md](research_archive/agent_skills_technical_spec.md) | Technical specification |

### Supported Agents

| Agent | Global Path | Workspace Path | Priority |
|-------|-------------|----------------|----------|
| Claude Code | `~/.claude/skills/` | `.claude/skills/` | Project > Global |
| Windsurf | `~/.codeium/windsurf/skills/` | `.windsurf/skills/` | Workspace > Global |
| OpenCode | `~/.config/opencode/skill/` | `.opencode/skill/` | Project > Global |
| Kilo Code | `~/.kilocode/skills/` | `.kilocode/skills/` | Project > Global |
| Amp | `~/.config/agents/skills/` | `.agents/skills/` | Workspace > User |

### SKILL.md Format

```yaml
---
name: skill-name              # Required: lowercase, alphanumeric + hyphens, max 64
description: "What it does"   # Required: 1-1024 characters
version: 1.0                  # Optional
author: name                  # Optional
license: MIT                  # Optional
---

# Instructions

Your detailed instructions here...
```

### Glossary

| Term | Definition |
|------|------------|
| **Agent** | AI coding assistant (Claude Code, Windsurf, etc.) |
| **Skill** | Directory with SKILL.md containing instructions for an agent |
| **Managed Skill** | Skill in `~/.sikil/repo/` symlinked to agent directories |
| **Unmanaged Skill** | Skill as physical directory in agent location |
| **Global Path** | Agent's skill directory in user's home |
| **Workspace Path** | Agent's skill directory in project |

---

## Approval

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Product Owner | | | |
| Tech Lead | | | |
| Engineering | | | |

---

**Document Version**: 1.1  
**Last Updated**: January 16, 2026  
**Next Review**: February 16, 2026
