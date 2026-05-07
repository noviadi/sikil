# Sikil — Product Requirements Document

**Version**: 2.0
**Last Updated**: 2026-05-07
**Status**: Active (v0.1.0 implementation complete)
**Source of Truth**: This PRD is regenerated from `specs/` (spec-driven). When a claim here disagrees with a spec, the spec wins.

> Previous editions are archived under [docs/archive/PRD.v1.1.md](archive/PRD.v1.1.md).

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-16 | Initial draft (archived) |
| 1.1 | 2026-01-16 | Security hardening NFRs added (archived) |
| 2.0 | 2026-05-07 | Wholesale rewrite from `specs/` as authoritative source. **Categories of change**: cache backend corrected from SQLite to JSON file (matches `specs/cache.md` and `src/core/cache.rs`); Git URL formats narrowed to GitHub-only HTTPS + short form (SSH dropped, matches `specs/git-operations.md`); dependency list resynced with `Cargo.toml`; new FRs added for `--no-cache`, agent targeting, conflict-display verbosity, and exit-code policy that were spec-defined but absent from PRD; obsolete sections removed (Timeline & Milestones, Release Plan, Adoption Metrics, Usage Metrics, Approval); Launch Metrics renamed Quality Targets and scoped to verifiable bars. Product framing (Executive Summary, Problem Statement, Personas, Goals, Risks) preserved from v1.1 with factual corrections. |

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
10. [Quality Targets](#quality-targets)
11. [Risks & Mitigations](#risks--mitigations)
12. [Dependencies](#dependencies)
13. [Appendix](#appendix)

---

## Executive Summary

**Sikil** is a command-line tool for managing Agent Skills across multiple AI coding agents. It provides unified discovery, installation, synchronization, and management of skills that are otherwise fragmented across five different agent installations.

### The Problem

Developers using multiple AI coding agents (Claude Code, Windsurf, OpenCode, Kilo Code, Amp) face a management nightmare: each agent stores skills in different directories with incompatible path structures. This leads to duplicate skills, version mismatches, maintenance overhead, and lack of visibility.

### The Solution

A CLI-first tool that:

- **Discovers** all skills across all agent installations
- **Manages** them through a central repository with symlinks
- **Synchronizes** skills across agents with one command
- **Validates** skill structure and detects conflicts

### Key Value Proposition

Turn 2–4 hours of manual skill management into < 5 minutes of automated workflows.

---

## Problem Statement

### Current State

Developers using multiple AI coding agents experience:

```
Typical developer's machine:
├── ~/.claude/skills/           (13 skills)
├── ~/.codeium/windsurf/skills/ (18 skills)  ← Many duplicates
├── ~/.config/opencode/skill/   (9 skills)   ← Version conflicts
├── ~/.kilocode/skills/         (8 skills)
├── ~/.config/agents/skills/    (5 skills)
├── Project-1/.claude/skills/   (4 skills)   ← Which is current?
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
- Usage analytics / telemetry
- TUI (Terminal UI) interface
- Enterprise / team management features
- SSH Git URLs (only GitHub HTTPS supported)

---

## Target Users

### Primary Persona: Multi-Agent Developer

**Profile:**

- Uses 2+ AI coding agents daily
- Has 10–50+ skills installed
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

### In Scope

| Category | Features |
|----------|----------|
| **Discovery** | Scan all 5 agents (global + workspace), list skills, show details, fresh-scan via `--no-cache` |
| **Management** | Install (local + GitHub), adopt, unmanage, remove |
| **Synchronization** | Sync managed skill(s) to agents missing the symlink |
| **Validation** | SKILL.md format validation by path or installed name |
| **Conflict Detection** | Duplicate-unmanaged (errors), duplicate-managed (info) |
| **Configuration** | TOML config, agent path overrides, enable/disable agents, `config set`, `--edit` |
| **Output** | Human-readable colored output + JSON via `--json` |
| **Caching** | JSON file cache at `~/.sikil/cache.json` with mtime-based invalidation |
| **Shell Completions** | bash, zsh, fish |
| **Platforms** | macOS (Intel + Apple Silicon), Linux (x86_64 + aarch64) |

### Out of Scope

| Feature | Rationale | Future Version |
|---------|-----------|----------------|
| Version comparison | Complexity, undefined in spec | v1.1 |
| Marketplace integration | No standard exists | v2.0 |
| Skill dependencies | Not in agent skills spec | v2.0 |
| Windows support | Symlink semantics differ; requires platform abstraction | v1.1 |
| TUI interface | CLI-first approach | v1.2 |
| Usage analytics | Privacy concerns | TBD |
| Team / enterprise features | Scope creep | v2.0 |
| Kilo Code mode-specific dirs | Low usage | v1.1 |
| SSH Git URLs (`git@github.com:...`) | Defer to post-v1; HTTPS suffices | v1.1 |
| Non-GitHub Git hosts (GitLab, Bitbucket, self-hosted) | Restricted for security and simplicity | v1.1+ |
| Script execution from skills | Security: skills are read-only configuration | Never |

---

## Functional Requirements

> Each FR maps to one or more specs in `specs/`. The spec is the normative source.

### FR-01: Discovery & Listing

Spec: [skill-scanner.md](../specs/skill-scanner.md), [skill-discovery.md](../specs/skill-discovery.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01.1 | Scan all 5 agent global paths for enabled agents | P0 |
| FR-01.2 | Scan workspace paths relative to CWD for enabled agents | P0 |
| FR-01.3 | Scan the managed repo at `~/.sikil/repo/` and merge into results | P0 |
| FR-01.4 | Parse `SKILL.md` YAML frontmatter for every candidate directory | P0 |
| FR-01.5 | Classify each installation as managed, unmanaged, broken-symlink, or foreign-symlink | P0 |
| FR-01.6 | Skip non-directory / non-symlink entries and entries beginning with `.` | P1 |
| FR-01.7 | Filter `list` output by `--agent`, `--managed`, `--unmanaged`, `--conflicts`, `--duplicates` | P1 |
| FR-01.8 | Display `list` summary header with counts (`Found N skills (X managed, Y unmanaged)`) | P1 |
| FR-01.9 | Truncate description column to 50 chars in `list` text output | P2 |
| FR-01.10 | Treat `--duplicates` as alias of `--conflicts` | P2 |

**Commands:** `sikil list [--agent <name>] [--managed] [--unmanaged] [--conflicts] [--duplicates]`

### FR-02: Skill Details

Spec: [skill-discovery.md](../specs/skill-discovery.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-02.1 | Display metadata (name, description, version, author, license) for one skill | P0 |
| FR-02.2 | Show all installation locations with agent, scope, and symlink target | P0 |
| FR-02.3 | Display file-tree summary (`has_skill_md`, `has_scripts_dir`, `has_references_dir`, `file_count`) | P1 |
| FR-02.4 | Show total disk size (excluding `.git/`) in human-readable text mode and bytes in JSON | P2 |
| FR-02.5 | Surface `directory_name` in JSON when it differs from `metadata.name` | P2 |
| FR-02.6 | Return `SkillNotFound` (exit code 3) when name does not resolve | P0 |

**Commands:** `sikil show <name>`

### FR-03: Installation

Spec: [skill-installation.md](../specs/skill-installation.md), [git-operations.md](../specs/git-operations.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-03.1 | Install from local directory (absolute or CWD-relative) containing `SKILL.md` | P0 |
| FR-03.2 | Install from GitHub URL — short form `owner/repo`, short form with subdir `owner/repo/path/to/skill`, HTTPS `https://github.com/owner/repo[.git]` | P0 |
| FR-03.3 | Extract a subdirectory from a Git repo when given short-form `owner/repo/subpath` | P0 |
| FR-03.4 | Copy the skill to `~/.sikil/repo/<name>/` and symlink each target agent path to it | P0 |
| FR-03.5 | Validate skill (`SKILL.md` exists + parseable + required fields) before any filesystem mutation | P0 |
| FR-03.6 | Select target agents with `--to <all\|name[,name...]>`; prompt interactively when omitted (default to all in `--json` mode) | P0 |
| FR-03.7 | Create missing agent directories before symlinking | P1 |
| FR-03.8 | Reject install when the skill name already exists in `~/.sikil/repo/` (no overwrite); suggest `sikil adopt` or `sikil sync` | P0 |
| FR-03.9 | Reject sources containing symlinks (skill payload must be physical files) | P0 |
| FR-03.10 | On any failure during symlink creation, roll back: remove created symlinks, delete copied repo dir, clean Git temp dirs | P0 |
| FR-03.11 | Reject Git URLs starting with `-`, containing whitespace/NUL, or using `file://`; reject subdirectories containing `..` or absolute paths | P0 |
| FR-03.12 | Run `git clone` with `-c protocol.file.allow=never --depth=1 --` and array-style args (no shell) | P0 |
| FR-03.13 | Strip `.git/` from cloned source before copying into the repo | P1 |

**Commands:** `sikil install <source> [--to <agents>]`

### FR-04: Adoption

Spec: [skill-adoption.md](../specs/skill-adoption.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-04.1 | Move an unmanaged skill from an agent directory to `~/.sikil/repo/<name>/` | P0 |
| FR-04.2 | Replace the original location with a symlink pointing into the repo | P0 |
| FR-04.3 | Make `--from` optional when only one installation exists; require it when multiple installations exist (error lists locations) | P0 |
| FR-04.4 | Refuse adoption of a symlink (skill must be physical) or when the repo entry already exists | P0 |
| FR-04.5 | On symlink-creation failure after the move, attempt best-effort rollback (delete partial copy and move source back) | P1 |

**Commands:** `sikil adopt <name> [--from <agent>]`

### FR-05: Unmanagement

Spec: [skill-unmanagement.md](../specs/skill-unmanagement.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-05.1 | Replace managed symlinks with physical copies of the skill from the repo | P0 |
| FR-05.2 | Operate on all managed installations by default; restrict to one with `--agent <name>` | P0 |
| FR-05.3 | Delete the repo entry once every managed installation has been converted | P0 |
| FR-05.4 | Require interactive `[y/N]` confirmation; bypass with `--yes` or `--json` | P0 |
| FR-05.5 | On copy failure, remove partial copy and recreate the original symlink | P1 |

**Commands:** `sikil unmanage <name> [--agent <name>] [--yes]`

### FR-06: Removal

Spec: [skill-removal.md](../specs/skill-removal.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-06.1 | Remove installations from a comma-separated list of agents with `--agent <csv>` | P0 |
| FR-06.2 | Remove from every agent and delete `~/.sikil/repo/<name>/` with `--all` | P0 |
| FR-06.3 | Require exactly one of `--agent` or `--all` (validation error otherwise) | P0 |
| FR-06.4 | Support both managed (symlink → `remove_file`) and unmanaged (directory → `remove_dir_all`) installations | P0 |
| FR-06.5 | Require interactive `[y/N]` confirmation; bypass with `--yes` or `--json` | P0 |
| FR-06.6 | When removing per-agent leaves the repo orphaned, prompt to delete the repo entry | P1 |

**Commands:** `sikil remove <name> --agent <csv>`, `sikil remove <name> --all`

### FR-07: Synchronization

Spec: [skill-synchronization.md](../specs/skill-synchronization.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-07.1 | Create missing `agent_path/<name> -> ~/.sikil/repo/<name>` symlinks for one named skill | P0 |
| FR-07.2 | With `--all`, iterate every directory in `~/.sikil/repo/` (skipping `.`-prefixed names) | P0 |
| FR-07.3 | Restrict targets with `--to <agents>`; default = all enabled agents | P1 |
| FR-07.4 | Skip an agent that already has a symlink (idempotent); fail with adopt-suggestion when a physical directory blocks | P0 |
| FR-07.5 | Continue with warnings if `ensure_dir_exists` or `create_symlink` fails for a single agent (do not abort the whole run) | P1 |
| FR-07.6 | Print informational message and take no action when every agent is already in sync | P2 |

**Commands:** `sikil sync <name>`, `sikil sync --all [--to <agents>]`

### FR-08: Validation

Spec: [skill-validation.md](../specs/skill-validation.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-08.1 | Accept either a filesystem path or an installed skill name | P0 |
| FR-08.2 | Run blocking checks in order: SKILL.md exists → frontmatter valid → required fields present → name format → description length (1–1024) | P0 |
| FR-08.3 | Treat `name` and `description` as required; treat `version`, `author`, `license` as warnings if absent | P0 |
| FR-08.4 | Enforce skill-name regex `^[a-z0-9][a-z0-9_-]{0,63}$`; reject empty, path separators, `.`, `..` | P0 |
| FR-08.5 | Detect optional `scripts/` and `references/` directories | P1 |
| FR-08.6 | Exit code 0 on pass, 2 on validation failure | P0 |
| FR-08.7 | Emit human-readable per-check `✓`/`✗` output and JSON via `--json` | P0 |

**Commands:** `sikil validate <path-or-name>`

### FR-09: Conflict Detection

Spec: [conflict-detection.md](../specs/conflict-detection.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-09.1 | Detect `DuplicateUnmanaged` (>1 unmanaged installations of a skill at distinct paths); classify as error | P0 |
| FR-09.2 | Detect `DuplicateManaged` (>1 managed installations all pointing to the same repo path); classify as info | P1 |
| FR-09.3 | Render conflict summary inline with the `list` header (e.g., `Found N skills (...) - 2 errors, 3 info suppressed`) | P0 |
| FR-09.4 | Suppress `DuplicateManaged` details in human-readable output unless `-v`/`--verbose` is set | P1 |
| FR-09.5 | Always include all conflicts in JSON output regardless of verbosity | P0 |
| FR-09.6 | `list --conflicts` shows error-level conflicts by default; with `-v` shows info conflicts too | P1 |
| FR-09.7 | Provide actionable recommendations per conflict type (`adopt` for unmanaged duplicates, `remove` for redundant managed copies) | P0 |

**Commands:** `sikil list --conflicts`, `sikil list --conflicts -v`

### FR-10: Configuration

Spec: [configuration.md](../specs/configuration.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-10.1 | Store config at `~/.sikil/config.toml` in TOML format | P0 |
| FR-10.2 | Support per-agent overrides under `[agents.<agent-name>]` with `enabled`, `global_path`, `workspace_path` | P0 |
| FR-10.3 | Reject unknown fields (`#[serde(deny_unknown_fields)]`) and reject files larger than 1 MB (`ConfigTooLarge`) | P0 |
| FR-10.4 | Treat a missing config file as `Config::default()` (every agent enabled with default paths) | P0 |
| FR-10.5 | Display current config via `sikil config`, marking `(default)` vs `(custom)` values; support `--json` | P1 |
| FR-10.6 | Open config in `$EDITOR` (fallback `vi`) via `sikil config --edit`; create defaults if missing; validate after edit | P2 |
| FR-10.7 | Set individual values via `sikil config --set <agents.<agent>.<field>> <value>` (fields: `enabled`, `global_path`, `workspace_path`) | P1 |
| FR-10.8 | Expand leading `~` in path fields via `shellexpand::tilde` | P1 |

**Commands:** `sikil config`, `sikil config --edit`, `sikil config --set <key> <value>`

### FR-11: Output Formats

Spec: [cli-output.md](../specs/cli-output.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-11.1 | Default to colored, human-readable output | P0 |
| FR-11.2 | Emit machine-parseable JSON on stdout when `--json` is set; route messages to stderr in JSON mode | P0 |
| FR-11.3 | Disable colors when `NO_COLOR` env var is set (any value) and when stdout is not a TTY | P1 |
| FR-11.4 | Use `✓` (green), `⚠` (yellow), `✗` (red), and informational lines without color | P1 |
| FR-11.5 | Display progress spinners/bars on long operations; suppress them in JSON or non-TTY | P2 |
| FR-11.6 | Suppress non-essential output when `--quiet` is set; reject the combination `--quiet --verbose` | P1 |

### FR-12: Shell Completions

Spec: [shell-completions.md](../specs/shell-completions.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-12.1 | Generate completions for `bash`, `zsh`, `fish` (case-insensitive shell name) | P1 |
| FR-12.2 | Print to stdout by default; write to file via `--output <PATH>` (with confirmation on stderr) | P1 |
| FR-12.3 | Reject unsupported shells with the exact message `Unsupported shell '<name>'. Supported shells: bash, zsh, fish` | P2 |

**Commands:** `sikil completions <shell> [--output <PATH>]`

### FR-13: Cache

Spec: [cache.md](../specs/cache.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-13.1 | Persist scan metadata at `~/.sikil/cache.json` as a JSON file with `version: 1` and a `BTreeMap` of path-keyed entries | P1 |
| FR-13.2 | Invalidate an entry when the cached `mtime` differs from the SKILL.md file's current `mtime`, or when the file is missing | P1 |
| FR-13.3 | Clear the entire cache when the on-disk `version` does not equal `CACHE_VERSION` | P1 |
| FR-13.4 | Clear the entire cache when the file size exceeds 15 MB (`MAX_CACHE_SIZE`) | P1 |
| FR-13.5 | Reject `put()` entries whose content hash exceeds 64 chars (`MAX_HASH_SIZE`); other failures are non-fatal | P1 |
| FR-13.6 | Use atomic `cache.json.tmp` + rename for every write; last-writer-wins | P1 |
| FR-13.7 | Bypass the cache entirely when `--no-cache` is supplied; treat any read failure as a cache miss | P0 |

### FR-14: Agent Targeting

Spec: [agent-targeting.md](../specs/agent-targeting.md)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-14.1 | Accept agent selectors as `all` (all enabled), single name, or comma-separated list (whitespace-trimmed) | P0 |
| FR-14.2 | Validate every named agent against the config; reject unknown or disabled agents with actionable messages | P0 |
| FR-14.3 | Prompt interactively (numbered list, `[a]` for all) when no selector is provided in TTY mode | P1 |
| FR-14.4 | Default to `all` (skip prompt) under `--json` mode | P0 |
| FR-14.5 | Reject empty selections with `no valid agents specified` / `no enabled agents in configuration` | P0 |

---

## Non-Functional Requirements

### Performance

Spec: [skill-scanner.md](../specs/skill-scanner.md), [cache.md](../specs/cache.md)

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-01 | `sikil list` response time (50 skills) | < 500 ms |
| NFR-02 | Cached scan response time | < 100 ms |
| NFR-03 | Cache invalidation policy | SKILL.md `mtime` |
| NFR-04 | Release binary size | < 10 MB |

### Reliability

Spec: [atomic-operations.md](../specs/atomic-operations.md), [error-handling.md](../specs/error-handling.md), [skill-scanner.md](../specs/skill-scanner.md)

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-05 | Destructive operations require confirmation (`[y/N]`, default no) | 100% (`remove`, `unmanage`) |
| NFR-06 | Failed operations restore prior state (atomic copy + reverse-order rollback for files; `tempfile` + rename for repo writes; per-agent best-effort rollback for symlink chains) | Atomic rollback |
| NFR-07 | Permission errors surface as `PermissionDenied { operation, path }` rather than panics | No crashes |
| NFR-08 | Missing agent directories are silently skipped during scan | No crashes |
| NFR-09 | Broken and foreign symlinks are detected and recorded; scan continues | Warning, not fatal |
| NFR-10 | Cache read/write failures are non-fatal (treat as cache miss) | Operation continues |

### Compatibility

Spec: [build-and-platform.md](../specs/build-and-platform.md)

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-11 | macOS Intel support (`x86_64-apple-darwin`) | Full |
| NFR-12 | macOS Apple Silicon support (`aarch64-apple-darwin`) | Full |
| NFR-13 | Linux x86_64 support (`x86_64-unknown-linux-gnu`) | Full |
| NFR-14 | Linux aarch64 support (`aarch64-unknown-linux-gnu`) | Full |
| NFR-15 | Windows | Explicitly unsupported (uses `std::os::unix::fs::symlink`) |
| NFR-16 | Minimum Rust toolchain | 1.75+ |
| NFR-17 | Minimum Git version | 2.0+ |

### Usability

Spec: [cli-output.md](../specs/cli-output.md), [error-handling.md](../specs/error-handling.md)

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-18 | Colored terminal output respecting TTY and `NO_COLOR` | ANSI colors |
| NFR-19 | Error messages follow the pattern `Error: <description>` and include actionable guidance where applicable | Actionable |
| NFR-20 | Progress indicators on long operations (spinner / bar) | `indicatif` |
| NFR-21 | First-time user can list, install, and sync a skill end-to-end | < 5 minutes |
| NFR-22 | Distinct exit codes: `0` success, `2` validation, `3` skill not found, `4` permission, `5` network, `1` other | 100% per `cli-schema.md` |

### Security

Spec: [git-operations.md](../specs/git-operations.md), [atomic-operations.md](../specs/atomic-operations.md), [configuration.md](../specs/configuration.md), [skill-installation.md](../specs/skill-installation.md)

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-23 | Sikil never executes scripts from skills | Read-only |
| NFR-24 | Skill name is validated (`^[a-z0-9][a-z0-9_-]{0,63}$`); paths are canonicalized; `..` and absolute subdirectory paths are rejected | Path-traversal blocked |
| NFR-25 | Git operations restricted to GitHub HTTPS; `file://`, leading `-`, whitespace, NUL, and SSH formats rejected | GitHub-only HTTPS |
| NFR-26 | Skill copy step rejects symlinks in the source tree (via `WalkDir::follow_links(false)`); existing symlinks at copy targets are removed before recreation | Symlinks not allowed in skill payload |
| NFR-27 | Config file rejects unknown fields and refuses files larger than 1 MB | Hardened |
| NFR-28 | Cache writes use atomic temp + rename; cache integrity is enforced by version check + 15 MB size cap (full clear on violation) | Crash-safe at the cache level |
| NFR-29 | `git clone` invoked with `-c protocol.file.allow=never --depth=1 --`, array-style args, no shell interpolation | Subprocess-safe |

---

## Technical Architecture

### Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust 2021 (≥ 1.75) | Performance, single binary, cross-platform |
| CLI Framework | `clap` 4 (derive) | Industry standard |
| Serialization | `serde` 1, `serde_yaml` 0.9, `serde_json` 1, `toml` 0.8 | YAML for SKILL.md, JSON for cache and `--json`, TOML for config |
| Filesystem | `fs-err` 2, `walkdir` 2, `tempfile` 3 | Better errors, robust traversal, atomic temp dirs |
| Path expansion | `shellexpand` 3, `directories` 5 | `~`/`$VAR` expansion, OS-aware home discovery |
| Symlinks | `std::os::unix::fs::symlink` | Unix-native; Windows out of scope |
| Git | Shell out to `git` CLI | Best authentication / credential support |
| Caching | JSON file (`serde_json` + atomic rename) | Smaller binary, simpler operations than SQLite |
| Hashing | `sha2` 0.10 | SHA-256 for cache content hash |
| Patterns | `regex` 1, `once_cell` 1 | Skill name validation, lazy regex statics |
| Terminal | `anstream` 0.6, `anstyle` 1, `indicatif` 0.17, `atty` 0.2 | Modern color handling, progress bars, TTY detection |
| Testing | `assert_cmd` 2, `predicates` 3, `insta` 1 | CLI testing, snapshot testing |
| Man page | `clap_mangen` 0.2 (build dep) | Generates `sikil.1` |

### Layered Architecture

```diagram
╭─────────────────────────────────────────────────────────────╮
│                        CLI Layer                            │
│  Argument parsing (clap), exit-code mapping, output mode    │
╰────────────────────────────┬────────────────────────────────╯
                             │
╭────────────────────────────▼────────────────────────────────╮
│                     Commands Layer                          │
│  list │ show │ install │ adopt │ unmanage │ remove │ sync   │
│  validate │ config │ completions │ agent_selection         │
╰────────────────────────────┬────────────────────────────────╯
                             │
╭────────────────────────────▼────────────────────────────────╮
│                       Core Layer                            │
│  Scanner │ Parser │ Config │ Cache │ Conflicts │ Errors    │
│  Skill / Agent / Scope / Installation models               │
╰────────────────────────────┬────────────────────────────────╯
                             │
╭────────────────────────────▼────────────────────────────────╮
│                       Utils Layer                           │
│  paths │ symlink │ atomic │ git                            │
╰─────────────────────────────────────────────────────────────╯
```

### Data Model (summary)

```rust
// src/core/skill.rs
pub struct SkillMetadata {
    pub name: String,                 // required, validated against ^[a-z0-9][a-z0-9_-]{0,63}$
    pub description: String,          // required, 1..=1024 chars
    pub version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
}

pub struct Skill {
    pub metadata: SkillMetadata,
    pub directory_name: String,
    pub installations: Vec<Installation>,
    pub is_managed: bool,
    pub repo_path: Option<PathBuf>,
}

pub struct Installation {
    pub agent: Agent,
    pub path: PathBuf,
    pub scope: Scope,
    pub is_symlink: Option<bool>,
    pub symlink_target: Option<PathBuf>,
}

pub enum Agent { ClaudeCode, Windsurf, OpenCode, KiloCode, Amp }
pub enum Scope { Global, Workspace }
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
├── config.toml              # User configuration (TOML, ≤ 1 MB)
└── cache.json               # JSON scan cache (≤ 15 MB; auto-cleared on overflow / version mismatch)

Agent directories (symlinks point into ~/.sikil/repo/):
~/.claude/skills/git-workflow              → ~/.sikil/repo/git-workflow
~/.codeium/windsurf/skills/git-workflow    → ~/.sikil/repo/git-workflow
~/.config/opencode/skill/git-workflow      → ~/.sikil/repo/git-workflow
~/.kilocode/skills/git-workflow            → ~/.sikil/repo/git-workflow
~/.config/agents/skills/git-workflow       → ~/.sikil/repo/git-workflow
```

---

## User Experience

### Command Structure

```
sikil <command> [options] [arguments]

Commands:
  list         List skills across agents
  show         Show details for one skill
  install      Install a skill from a local path or GitHub URL
  adopt        Adopt an unmanaged skill into the managed repo
  unmanage     Convert a managed skill back to physical copies
  remove       Remove a skill from agents (and optionally the repo)
  sync         Create missing symlinks for managed skills
  validate     Validate a skill's structure
  config       View or modify configuration
  completions  Generate shell completions

Global Options:
  --json       Emit JSON on stdout (messages on stderr)
  --verbose, -v   Show extra detail
  --quiet,   -q   Suppress non-essential output
  --no-cache   Bypass the scan cache
  --version    Show version
  --help       Show help
```

`--quiet` and `--verbose` are mutually exclusive.

### Example Workflows

**Workflow 1: First-time setup**

```bash
$ sikil list
Found 15 skills (0 managed, 15 unmanaged) across 3 agents

$ sikil adopt git-workflow --from claude-code
✓ Adopted 'git-workflow' to managed repo
✓ Created symlink at ~/.claude/skills/git-workflow

$ sikil sync git-workflow
✓ Synced to windsurf, opencode, kilo-code, amp
```

**Workflow 2: Install new skill from GitHub**

```bash
$ sikil install team/skills/new-skill --to all
✓ Installed 'new-skill' to 5 agents
```

**Workflow 3: Clean up duplicates**

```bash
$ sikil list --conflicts
Found 3 duplicate-unmanaged conflicts

$ sikil adopt code-review --from claude-code
$ sikil sync code-review
$ sikil remove code-review --agent windsurf
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

## Quality Targets

| Target | Measurement |
|--------|-------------|
| Binary builds for all four supported targets | CI (`./scripts/build.sh`) |
| All 10 subcommands functional and integration-tested | `cargo test` (per-command test files in `tests/`) |
| Release binary size ≤ 10 MB | `tests/build_test.rs::test_release_binary_size_under_10mb` |
| `./scripts/verify.sh` exits 0 (tests + clippy + fmt) on every commit | Local + CI |

---

## Risks & Mitigations

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agent path changes | Medium | Medium | Config-based paths, regular updates |
| Git auth complexity | Medium | Medium | Shell out to `git`, leverage user's credential helpers |
| Symlink issues on network drives | Low | Medium | Document limitation, detect and warn |
| Large skill directories slow scan | Low | Low | JSON cache with mtime-based invalidation |

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
| Maintenance burden | Medium | Medium | Comprehensive tests, layered architecture, spec-driven docs |

---

## Dependencies

### External Dependencies

| Dependency | Version | Purpose | Risk |
|------------|---------|---------|------|
| Git CLI | 2.0+ | `sikil install` clones | Low (widely installed) |
| SKILL.md spec | Current | Skill format | Low (stable) |

### Internal Dependencies (Cargo)

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4 (`derive`) | CLI parsing |
| `clap_complete` | 4 | Shell completion generation |
| `serde` | 1 | Serialization |
| `serde_yaml` | 0.9 | SKILL.md frontmatter parsing |
| `serde_json` | 1 | JSON output and cache file |
| `toml` | 0.8 | Config file parsing |
| `walkdir` | 2 | Directory traversal |
| `anyhow` | 1 | Command-layer error context |
| `thiserror` | 1 | Domain error variants |
| `anstream` | 0.6 | Terminal stream handling |
| `anstyle` | 1 | Color styling |
| `indicatif` | 0.17 | Progress bars / spinners |
| `atty` | 0.2 | TTY detection |
| `shellexpand` | 3 | `~` / `$VAR` expansion |
| `directories` | 5 | OS-aware home directory |
| `fs-err` | 2 | Filesystem operations with better errors |
| `tempfile` | 3 | Atomic temp dirs / files |
| `once_cell` | 1 | Lazy statics for regex |
| `regex` | 1 | Skill name + Git URL validation |
| `sha2` | 0.10 | Cache content hash |
| `assert_cmd` (dev) | 2 | CLI integration testing |
| `predicates` (dev) | 3 | Assertion combinators |
| `insta` (dev) | 1 (`yaml`) | Snapshot testing |
| `clap_mangen` (build) | 0.2 | `sikil.1` man page generation |

`rusqlite` is **not** a dependency. The earlier SQLite-based cache was replaced by a JSON file cache before v0.1.0.

---

## Appendix

### Related Documents

| Document | Description |
|----------|-------------|
| [TRD.md](TRD.md) | Technical Requirements Document (v2.0) |
| [specs/](../specs/) | Single source of truth — 24 SSOT specs |
| [specs/README.md](../specs/README.md) | Topic index and architecture mapping |
| [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) | Pending and completed implementation tasks |
| [docs/archive/](archive/) | Frozen v1.x PRD/TRD and supporting docs |

### Supported Agents

| Agent | CLI name | Global Path | Workspace Path |
|-------|----------|-------------|----------------|
| Claude Code | `claude-code` | `~/.claude/skills/` | `.claude/skills/` |
| Windsurf | `windsurf` | `~/.codeium/windsurf/skills/` | `.windsurf/skills/` |
| OpenCode | `opencode` | `~/.config/opencode/skill/` | `.opencode/skill/` |
| Kilo Code | `kilocode` | `~/.kilocode/skills/` | `.kilocode/skills/` |
| Amp | `amp` | `~/.config/agents/skills/` | `.agents/skills/` |

### SKILL.md Format

```yaml
---
name: skill-name              # Required: ^[a-z0-9][a-z0-9_-]{0,63}$
description: "What it does"   # Required: 1–1024 characters
version: 1.0                  # Optional
author: name                  # Optional
license: MIT                  # Optional
---

# Instructions

Your detailed instructions here...
```

Frontmatter must:

- Begin at file start (whitespace allowed before the opening `---`)
- Use exactly two `---` delimiters
- Contain valid YAML between the delimiters

### Glossary

| Term | Definition |
|------|------------|
| **Agent** | AI coding assistant (Claude Code, Windsurf, OpenCode, Kilo Code, Amp) |
| **Skill** | Directory with `SKILL.md` containing instructions for an agent |
| **Managed Skill** | Skill stored in `~/.sikil/repo/` and exposed to agents via symlinks |
| **Unmanaged Skill** | Skill present as a physical directory inside an agent's path |
| **Foreign Symlink** | Symlink in an agent path whose target lies **outside** `~/.sikil/repo/` |
| **Broken Symlink** | Symlink whose target does not resolve |
| **Global Path** | Agent's skill directory under the user's home |
| **Workspace Path** | Agent's skill directory inside a project |

---

**Document Version**: 2.0
**Source of Truth**: `specs/`
