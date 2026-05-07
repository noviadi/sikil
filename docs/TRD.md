# Sikil — Technical Requirements Document

**Version**: 2.0
**Last Updated**: 2026-05-07
**Status**: Active (v0.1.0 implementation complete)
**Source of Truth**: This TRD is regenerated from `specs/` (spec-driven). When this document disagrees with a spec, the spec wins.

> Previous editions are archived under [docs/archive/TRD.v1.0.md](archive/TRD.v1.0.md).

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-16 | Initial draft (archived) |
| 1.1 | 2026-01-16 | Security hardening additions (archived) |
| 2.0 | 2026-05-07 | Wholesale rewrite from `specs/`. **Categories of change**: §State & Caching rewritten end-to-end (SQLite → JSON file with version/size/atomic-rename rules); §Domain Model resynced with `src/core/skill.rs` (added `directory_name`, `repo_path`, `Installation::is_symlink: Option<bool>`); §Filesystem Contract corrected (`cache.json` not `cache.db`); §Command Specifications regenerated per command spec; §Security regenerated (SSH dropped, GitHub-only HTTPS, symlink rejection details, config size cap, subprocess hardening); §Architecture & Module Boundaries resynced with `specs/README.md` mapping; per-spec cross-reference table added. Engineering framing in §2 / §13 / §14 preserved with stale tech claims scrubbed. |

---

## Table of Contents

1. [Document Overview](#1-document-overview)
2. [Technical Goals & Constraints](#2-technical-goals--constraints)
3. [System Overview](#3-system-overview)
4. [Domain Model](#4-domain-model)
5. [Filesystem Contract](#5-filesystem-contract)
6. [Command Specifications](#6-command-specifications)
7. [Architecture & Module Boundaries](#7-architecture--module-boundaries)
8. [Data Flows & Algorithms](#8-data-flows--algorithms)
9. [State & Caching](#9-state--caching)
10. [Security & Safety](#10-security--safety)
11. [Observability & UX](#11-observability--ux)
12. [Testing Strategy](#12-testing-strategy)
13. [Technical Risks & Mitigations](#13-technical-risks--mitigations)
14. [Open Questions & Deferred Decisions](#14-open-questions--deferred-decisions)
15. [Spec Cross-Reference](#15-spec-cross-reference)

---

## 1. Document Overview

### Purpose

This Technical Requirements Document (TRD) defines the technical implementation for **Sikil**, a Rust CLI tool for managing Agent Skills across multiple AI coding agents. It translates the PRD's functional requirements into concrete contracts, module boundaries, and architectural decisions, all anchored in the SSOT specs under `specs/`.

### Related Documents

| Document | Purpose |
|----------|---------|
| [PRD.md](PRD.md) | Product requirements (v2.0, regenerated from specs) |
| [specs/](../specs/) | Source of truth for behavior |
| [specs/README.md](../specs/README.md) | Topic index and architecture mapping |
| [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) | Pending and completed implementation tasks |
| [archive/TRD.v1.0.md](archive/TRD.v1.0.md) | Frozen prior edition |

---

## 2. Technical Goals & Constraints

### Technical Goals

| Goal | Metric | Rationale |
|------|--------|-----------|
| **Safety** | Zero data loss | Destructive operations are atomic with rollback (atomic-operations spec) |
| **Predictability** | Deterministic output | Scripting requires stable, consistent behavior (`BTreeMap` for cache, alphabetical sort in `list`) |
| **Performance** | < 500 ms `list` (50 skills); < 100 ms cached | Fast feedback for interactive use |
| **Portability** | macOS + Linux (x86_64 + aarch64) | Primary developer platforms |
| **Simplicity** | Single binary, no native deps | Easy install, no runtime dependencies beyond Git |

### Technical Constraints

| Constraint | Rationale |
|------------|-----------|
| **No Windows support (v1)** | Symlink semantics differ; uses `std::os::unix::fs::symlink` directly |
| **No script execution** | Security: skills are read-only configuration |
| **No telemetry** | Privacy: no network calls except `git clone` |
| **Shell out to `git`** | Leverage existing credential helpers; avoid `libgit2` complexity |
| **Single-threaded** | Simplicity; performance targets are modest |
| **Cache is a JSON file** | Smaller binary than embedded SQLite, simpler atomic semantics, no native dep |

### Language & Toolchain

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | 2021 edition, ≥ 1.75 |
| Targets | `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` | — |
| Release profile | `opt-level=3`, `lto=true`, `codegen-units=1`, `panic="abort"`, `strip=true` | Cargo |

---

## 3. System Overview

### High-Level Architecture

```diagram
╭─────────────────────────────────────────────────────────────────╮
│                        USER / SCRIPTS                           │
╰────────────────────────────┬────────────────────────────────────╯
                             │ CLI invocation
                             ▼
╭─────────────────────────────────────────────────────────────────╮
│                       CLI LAYER (clap)                          │
│  • Argument parsing (src/cli/app.rs)                            │
│  • Exit-code mapping (src/main.rs::get_exit_code)               │
│  • Output mode selection (src/cli/output.rs)                    │
╰────────────────────────────┬────────────────────────────────────╯
                             │
                             ▼
╭─────────────────────────────────────────────────────────────────╮
│                       COMMANDS LAYER                            │
│  list │ show │ install │ adopt │ unmanage │ remove │ sync       │
│  validate │ config │ completions │ agent_selection             │
│  • Orchestration; uses anyhow::Result for plumbing             │
╰────────────────────────────┬────────────────────────────────────╯
                             │
                             ▼
╭─────────────────────────────────────────────────────────────────╮
│                          CORE LAYER                             │
│  Scanner │ Parser │ Conflicts │ Cache │ Config │ Errors        │
│  Skill / Agent / Scope / Installation models                   │
│  • Domain logic; uses thiserror for SikilError                 │
╰────────────────────────────┬────────────────────────────────────╯
                             │
                             ▼
╭─────────────────────────────────────────────────────────────────╮
│                          UTILS LAYER                            │
│  paths │ symlink │ atomic │ git                                │
╰────────────────────────────┬────────────────────────────────────╯
                             │
                             ▼
╭─────────────────────────────────────────────────────────────────╮
│                       EXTERNAL SYSTEMS                          │
│  • Filesystem (agent dirs, ~/.sikil/repo, workspace)            │
│  • Git CLI (subprocess)                                         │
│  • Config file (~/.sikil/config.toml)                           │
│  • Cache file (~/.sikil/cache.json)                             │
╰─────────────────────────────────────────────────────────────────╯
```

### IO Boundaries

| Boundary | Type | Description |
|----------|------|-------------|
| Agent directories | Read / write | Scan and create / remove symlinks |
| Managed repository | Read / write | `~/.sikil/repo/` skill storage |
| Config file | Read / write | `~/.sikil/config.toml` (TOML, ≤ 1 MB) |
| Cache file | Read / write | `~/.sikil/cache.json` (JSON, ≤ 15 MB; atomic temp + rename) |
| Git CLI | Subprocess | Shallow clone (`--depth=1`) of GitHub HTTPS URLs |
| stdout | Write | Primary output (human or JSON) |
| stderr | Write | Errors, warnings, progress, JSON-mode messages |

---

## 4. Domain Model

### Core Types

#### Agent (`src/core/skill.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    ClaudeCode,
    Windsurf,
    OpenCode,
    KiloCode,
    Amp,
}

impl Agent {
    pub fn all() -> &'static [Agent];                  // 5 variants
    pub fn cli_name(&self) -> &'static str;            // claude-code, windsurf, opencode, kilocode, amp
    pub fn from_cli_name(name: &str) -> Option<Agent>; // unknown name → None
}

impl Display for Agent { /* writes cli_name */ }
```

#### Agent Paths (defaults)

| Agent | Global Path | Workspace Path |
|-------|-------------|----------------|
| `ClaudeCode` | `~/.claude/skills` | `.claude/skills` |
| `Windsurf` | `~/.codeium/windsurf/skills` | `.windsurf/skills` |
| `OpenCode` | `~/.config/opencode/skill` | `.opencode/skill` |
| `KiloCode` | `~/.kilocode/skills` | `.kilocode/skills` |
| `Amp` | `~/.config/agents/skills` | `.agents/skills` |

`AgentConfig` (in `src/core/config.rs`):

```rust
pub struct AgentConfig {
    pub enabled: bool,
    pub global_path: PathBuf,
    pub workspace_path: PathBuf,
}
```

Per-agent overrides live under `[agents.<cli-name>]` in `~/.sikil/config.toml`.

#### Scope

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Global,
    Workspace,
}
```

#### SkillMetadata

```rust
pub struct SkillMetadata {
    pub name: String,                     // required, validated
    pub description: String,              // required, 1..=1024 chars
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}
```

#### Installation

```rust
pub struct Installation {
    pub agent: Agent,
    pub path: PathBuf,
    pub scope: Scope,
    pub is_symlink: Option<bool>,
    pub symlink_target: Option<PathBuf>,
}
```

#### Skill

```rust
pub struct Skill {
    pub metadata: SkillMetadata,
    pub directory_name: String,           // physical directory name (may differ from metadata.name)
    pub installations: Vec<Installation>,
    pub is_managed: bool,                 // true if any installation is a managed symlink
    pub repo_path: Option<PathBuf>,       // canonical path under ~/.sikil/repo/
}
```

Helpers: `Skill::with_installation`, `Skill::with_repo` (sets `is_managed=true`), `Skill::is_orphan` (`true` iff `installations.is_empty()`).

#### Skill name validation (`src/core/parser.rs::validate_skill_name`)

- Regex: `^[a-z0-9][a-z0-9_-]{0,63}$`
- Length: 1..=64 characters
- Rejects empty, `/` or `\`, `.`, `..`

---

## 5. Filesystem Contract

### Sikil-managed paths (`src/utils/paths.rs`)

| Function | Path |
|----------|------|
| `get_repo_path()` | `~/.sikil/repo/` |
| `get_config_path()` | `~/.sikil/config.toml` |
| `get_cache_path()` | `~/.sikil/cache.json` |

`expand_path(&str) -> PathBuf` resolves `~` and `$VAR`.
`ensure_dir_exists(&Path) -> io::Result<()>` is idempotent and creates missing parents.

Home directory lookup failures cause `get_*` to panic — these calls are never expected to fail on supported platforms.

### Repository layout

```
~/.sikil/
├── repo/                # one subdirectory per managed skill (canonical copy)
│   └── <skill-name>/
│       ├── SKILL.md
│       ├── scripts/    (optional)
│       └── references/ (optional)
├── config.toml          # TOML, ≤ 1 MB; deny_unknown_fields
└── cache.json           # JSON, ≤ 15 MB; atomic temp + rename
```

### Agent layout (after install or sync)

```
<agent_path>/<skill-name>  →  ~/.sikil/repo/<skill-name>   (Unix symlink)
```

---

## 6. Command Specifications

> Each subsection summarizes the command's CLI surface, behavior, and error variants. The corresponding spec under `specs/` is the normative source.

### 6.1 `list` (spec: `skill-discovery.md`, `skill-scanner.md`)

| Aspect | Detail |
|--------|--------|
| Args | `--agent <name>`, `--managed`, `--unmanaged`, `--conflicts`, `--duplicates` (alias of `--conflicts`), global `--json`, `--verbose`, `--no-cache` |
| Behavior | Run `Scanner::new()` (or `Scanner::without_cache()` if `--no-cache`); scan every enabled agent's global + workspace paths plus `~/.sikil/repo/`; merge same-named skills; classify each installation |
| Output (text) | Header `Found N skills (X managed, Y unmanaged) - <conflict-summary>`; columns NAME, DESCRIPTION (truncated to 50 chars), AGENTS; status `✓` managed / `?` unmanaged |
| Output (JSON) | Skill array; conflicts always included regardless of `--verbose` |
| Conflict suppression | `DuplicateManaged` hidden in text mode unless `--verbose`; `--conflicts` filter shows error-level by default |

### 6.2 `show` (spec: `skill-discovery.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<name>` (positional), `--json`, `--no-cache` |
| Behavior | Look up skill by name; missing → `SikilError::SkillNotFound` (exit 3); compute file tree (`has_skill_md`, `has_scripts_dir`, `has_references_dir`, `file_count`) and total size, excluding `.git/` |
| Output | Metadata, managed status, canonical path, all installations (agent / path / scope / symlink target), file tree, size (human-readable in text, bytes in JSON) |
| JSON | `{name, directory_name?, description, version?, author?, license?, managed, canonical_path?, installations[], file_tree, total_size_bytes}` |

### 6.3 `install` (spec: `skill-installation.md`, `git-operations.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<source>` (path or GitHub URL), `--to <agents>`, `--json` |
| Source detection | `src/main.rs::is_git_url()` returns `true` for `https://github.com/...` and short forms `owner/repo[/sub/path]` (excluding existing local paths and paths starting with `/`, `.`, `-`, or single-segment names) |
| Local flow | Validate source dir + SKILL.md → parse name → resolve `--to` → ensure repo dest absent → ensure each agent dest absent → `copy_skill_dir` (rejects symlinks) → `create_symlink` per agent → rollback on failure |
| Git flow | `parse_git_url` → `clone_repo` (`git clone -c protocol.file.allow=never --depth=1 -- <url> <dest>`) into `tempfile::TempDir` → optional `extract_subdirectory` (rejects `..`, absolute paths) → validate SKILL.md → cleanup `.git/` → `copy_skill_dir` → `create_symlink` per agent → rollback + temp cleanup on failure |
| Errors | `DirectoryNotFound`, `ValidationError` ("source path is not a directory", "no agents selected"), `InvalidSkillMd`, `AlreadyExists` (with `sikil adopt` / `sikil sync` suggestions), `SymlinkNotAllowed`, `GitError`, `InvalidGitUrl`, `PathTraversal`, `PermissionDenied` |

### 6.4 `adopt` (spec: `skill-adoption.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<name>`, `--from <agent>`, `--json` |
| Preconditions | Skill exists in ≥ 1 agent dir; source not a symlink; `~/.sikil/repo/<name>/` absent; repo writable |
| `--from` | Optional with single installation; required with multiple (error lists every location) |
| Process | Scan → resolve target install → verify unmanaged → `atomic_move_dir(source, dest)` → `create_symlink(dest, source)`. Symlink failure after move: best-effort rollback (delete partial copy and move back). |
| Errors | `SkillNotFound`, `ValidationError`, `AlreadyExists`, `PermissionDenied`, `SymlinkError` |

### 6.5 `unmanage` (spec: `skill-unmanagement.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<name>`, `--agent <name>`, `--yes`, `--json` |
| Preconditions | Skill exists in `~/.sikil/repo/`; ≥ 1 managed installation |
| Default scope | All managed installations; with `--agent` only that agent's symlink |
| Process | For each: `fs::remove_file` symlink → `copy_skill_dir(repo, original_path)`; on copy failure remove partial copy + recreate symlink. After all: if every managed install was converted, delete `~/.sikil/repo/<name>/` |
| Confirmation | `[y/N]` (default no); `y/yes/Y` confirms; bypass with `--yes` or `--json` |
| Errors | `ValidationError`, `SkillNotFound`, `SymlinkError`, `PermissionDenied` |

### 6.6 `remove` (spec: `skill-removal.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<name>`, `--agent <csv>`, `--all`, `--yes`, `--json` |
| Validation | Exactly one of `--agent` or `--all` required |
| Per-agent | Remove only listed agents' installations; preserve repo unless orphaned (then prompt to delete) |
| `--all` | Remove every agent installation AND delete `~/.sikil/repo/<name>/` via `safe_remove_dir(confirmed=true)` |
| Symlinks vs dirs | Symlink → `fs::remove_file`; physical dir → `fs::remove_dir_all` |
| Confirmation | `[y/N]` (default no); bypass with `--yes` or `--json` |
| Errors | `ValidationError`, `SkillNotFound`, `PermissionDenied` |

### 6.7 `sync` (spec: `skill-synchronization.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<name>` (positional) OR `--all`; `--to <agents>`; `--json` |
| Validation | Either name or `--all` must be set |
| Single | Look up `~/.sikil/repo/<name>/`; verify SKILL.md present |
| `--all` | Iterate every entry under `~/.sikil/repo/`; skip names beginning with `.`; require SKILL.md per entry |
| `--to` default | All enabled agents (`parse_agent_selection(Some("all"), config)`); always uses `agent_config.global_path` |
| Per-target | Existing symlink → `already synced`; physical dir → fail with `sikil adopt` suggestion; missing → `ensure_dir_exists` + `create_symlink` |
| Failure mode | Per-agent `ensure_dir_exists` / `create_symlink` errors become warnings (continue with next agent) |
| Idempotent | Print info "all in sync" and take no action when nothing to do |

### 6.8 `validate` (spec: `skill-validation.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<path-or-name>`, `--json` |
| Input resolution | If path exists, validate that directory; otherwise resolve as installed skill name via `Scanner` |
| Checks (in order) | (1) SKILL.md exists [blocking], (2) frontmatter valid [blocking], (3) required `name`+`description` present [blocking], (4) name format valid [non-blocking], (5) description length 1..=1024 [non-blocking] |
| Warnings | Missing optional `version`, `author`, `license` |
| Detected dirs | `scripts/`, `references/` |
| Output (text) | Per-check `✓`/`✗`, metadata block, "Detected directories" block, trailing `PASSED` or `FAILED` |
| Output (JSON) | `{passed, skill_path, checks[{name,passed}], warnings[], metadata, detected_directories}` |
| Exit codes | 0 pass; 2 validation failure; 4 permission-denied during file read |

### 6.9 `config` (spec: `configuration.md`)

| Aspect | Detail |
|--------|--------|
| Args | (none) shows config; `--edit` opens editor; `--set <key> <value>` sets one field; `--json` for machine output |
| Display | Marks each value `(default)` or `(custom)`; respects `--json` |
| `--edit` | Creates default config when missing; runs `$EDITOR` (fallback `vi`); validates after edit |
| `--set` | Key format `agents.<agent>.<field>`; valid fields: `enabled`, `global_path`, `workspace_path`. Validation error otherwise. |
| Loading | Missing file → `Config::default()`; existing file parsed without merging defaults; `deny_unknown_fields`; size > 1 MB → `ConfigError::ConfigTooLarge` |
| Tilde expansion | `expand_paths()` runs `shellexpand::tilde` on path fields |

### 6.10 `completions` (spec: `shell-completions.md`)

| Aspect | Detail |
|--------|--------|
| Args | `<shell>` (case-insensitive: `bash`, `zsh`, `fish`), `--output <PATH>` |
| Behavior | `clap_complete::generate()` against `Cli::command()` with binary name `"sikil"` |
| Stdout | Default destination |
| `--output` | Write to file + emit confirmation to stderr |
| Errors | Unknown shell → exact message `Unsupported shell '<name>'. Supported shells: bash, zsh, fish` |

### 6.11 Global flags (spec: `cli-schema.md`, `cli-output.md`)

| Flag | Description |
|------|-------------|
| `--json` | Emit JSON on stdout; route messages to stderr; suppress progress indicators |
| `--verbose` / `-v` | Show extra detail (e.g., reveal `DuplicateManaged` conflicts) |
| `--quiet` / `-q` | Suppress non-essential output |
| `--no-cache` | Bypass scan cache (Scanner uses `without_cache()`) |
| `--version` | Print version `0.1.0` |
| `--help` / `-h` | Print help |

`--quiet` and `--verbose` are mutually exclusive; combination is rejected in `main.rs`.

### 6.12 Exit codes (spec: `cli-schema.md`)

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Generic / unmapped error |
| 2 | Validation failure |
| 3 | Skill not found |
| 4 | Permission denied |
| 5 | Network error (e.g., `git clone` failure) |

`SikilError::exit_code()` performs the mapping; `main.rs::get_exit_code()` falls back to `1` for non-`SikilError` errors.

---

## 7. Architecture & Module Boundaries

| Module | Responsibility | Notes |
|--------|----------------|-------|
| `src/main.rs` | Process entry, exit-code mapping, Git-URL detection (`is_git_url`) | Wires CLI → command dispatcher |
| `src/cli/app.rs` | `clap` derive structs (`Cli`, `Commands`) | Defines all subcommand args |
| `src/cli/output.rs` | `Output`, `Progress`, `MessageWriter` | TTY + `NO_COLOR` handling, JSON-mode stream routing |
| `src/commands/<cmd>.rs` | One file per subcommand | Returns `anyhow::Result<()>` |
| `src/commands/agent_selection.rs` | `parse_agent_selection`, `prompt_agent_selection`, `list_valid_agents` | Used by install / sync |
| `src/core/skill.rs` | `Skill`, `SkillMetadata`, `Installation`, `Agent`, `Scope` | Domain model |
| `src/core/parser.rs` | SKILL.md parsing, `validate_skill_name` | YAML frontmatter handling |
| `src/core/scanner.rs` | `Scanner`, `ScanResult`, classification logic | Optional cache, single-level traversal, non-fatal error collection |
| `src/core/conflicts.rs` | `Conflict`, `ConflictType`, formatting helpers, `filter_displayable_conflicts` | Verbose suppression of `DuplicateManaged` |
| `src/core/cache.rs` | `Cache` trait, `JsonCache`, `CacheFile`, `CachedEntry` | Atomic temp + rename, version + size guards |
| `src/core/config.rs` | `Config`, `AgentConfig`, TOML load + write, `deny_unknown_fields`, 1 MB cap | |
| `src/core/errors.rs` | `SikilError` (thiserror), `ConfigError`, `exit_code()` | |
| `src/utils/paths.rs` | Path helpers, tilde expansion, `~/.sikil/` discovery | |
| `src/utils/symlink.rs` | `create_symlink`, `is_symlink`, `read_symlink_target`, `resolve_realpath`, `is_managed_symlink` | Unix-only |
| `src/utils/atomic.rs` | `copy_skill_dir`, `atomic_move_dir`, `safe_remove_dir` | Reverse-order rollback, symlink rejection |
| `src/utils/git.rs` | `parse_git_url`, `ParsedGitUrl`, `clone_repo`, `extract_subdirectory`, `cleanup_clone` | GitHub-only validation, subprocess hardening |

Dependencies flow strictly downward: `cli` → `commands` → `core` → `utils`. The `core` and `utils` layers never depend on `cli` / `commands`.

---

## 8. Data Flows & Algorithms

### 8.1 Scan Algorithm (`src/core/scanner.rs`)

```
Input:  Config (with enabled agents)
Output: ScanResult { skills: Vec<Skill>, parse_errors: Vec<ParseError> }

1. For each enabled agent in config:
   a. Resolve global path (expand ~)
   b. Resolve workspace path (relative to CWD)
   c. For each path that exists:
      i.   fs::read_dir entries; skip names starting with '.'; skip non-dir/non-symlink
      ii.  For each candidate:
           - Check symlink classification (managed / unmanaged / broken / foreign)
           - Parse SKILL.md (cache lookup → mtime check; on miss, parse + put)
           - On parse error, record into parse_errors and continue
           - On success, build Installation

2. Scan ~/.sikil/repo/ (managed canonical store)

3. Group installations by metadata.name
4. For each group, build a Skill with `is_managed = any installation is a managed symlink`
5. Return ScanResult
```

### 8.2 Install Plan (local)

```
1. Validate source: exists, is dir, contains SKILL.md, parses
2. Extract skill_name from metadata
3. Compute repo_dest = ~/.sikil/repo/<skill_name>/
4. If repo_dest exists → AlreadyExists (with adopt/sync suggestion)
5. Resolve target agents from --to (or prompt)
6. For each target agent:
   a. Compute agent_dest = global_path/<skill_name>/
   b. If agent_dest exists → AlreadyExists
7. copy_skill_dir(source, repo_dest)   -- rejects symlinks; reverse-order rollback on failure
8. For each target agent:
   a. ensure_dir_exists(parent of agent_dest)
   b. create_symlink(repo_dest, agent_dest)
   c. On failure: rollback (remove all created symlinks, delete repo_dest), return error
```

### 8.3 Conflict Detection (`src/core/conflicts.rs`)

```
For each Skill:
  Let unmanaged = installations where is_symlink == Some(false)
                  (or symlink target outside ~/.sikil/repo/)
  Let managed   = installations whose symlink target is under ~/.sikil/repo/

  If unique unmanaged paths > 1:
    → Conflict::DuplicateUnmanaged   (is_error = true)

  If managed > 1 AND all point to the same single repo path:
    → Conflict::DuplicateManaged     (is_error = false; suppressed unless --verbose)
```

`filter_displayable_conflicts(conflicts, verbose)` returns `filter_error_conflicts(conflicts)` when `verbose=false`, else the original list. JSON output ignores verbosity (always all conflicts).

### 8.4 Git URL Parsing (`src/utils/git.rs`)

Accepted forms:

| Form | Example | Clone URL | Subdir |
|------|---------|-----------|--------|
| Short | `owner/repo` | `https://github.com/owner/repo.git` | None |
| Short + subdir | `owner/repo/path/to/skill` | `https://github.com/owner/repo.git` | `path/to/skill` |
| HTTPS with `.git` | `https://github.com/owner/repo.git` | as-is | None |
| HTTPS without `.git` | `https://github.com/owner/repo` | as-is | None |

Rejected forms (`InvalidGitUrl`):

| Form | Reason |
|------|--------|
| `git@github.com:owner/repo.git` | SSH not supported in v1 |
| `https://gitlab.com/...`, self-hosted | Non-GitHub HTTPS not supported |
| `file://...` | File protocol blocked |
| URLs containing whitespace or NUL | Injection prevention |
| URLs starting with `-` | Argument injection prevention |

Subdirectory extraction additionally rejects paths containing `..` or absolute paths (`PathTraversal`); uses `canonicalize()` to verify the extracted path lies inside the clone root.

### 8.5 Git Clone Invocation

```
git clone -c protocol.file.allow=never --depth=1 -- <url> <dest>
```

- Always invoked via `std::process::Command` array args (no shell)
- `protocol.file.allow=never` blocks any `file://` resolution at the protocol level
- `--depth=1` shallow clone reduces transfer
- `--` ends option parsing so a malicious URL cannot pose as a flag

After clone, `.git/` is removed (`cleanup_clone`) before the directory is copied into `~/.sikil/repo/`.

### 8.6 Atomic Operations (`src/utils/atomic.rs`)

| Operation | Notes |
|-----------|-------|
| `copy_skill_dir(src, dest)` | Deep copy via `WalkDir::follow_links(false)`; excludes `.git`; rejects symlinks (`SymlinkNotAllowed`); records each created file for reverse-order rollback on failure |
| `atomic_move_dir(src, dest)` | Try `fs::rename`; on cross-filesystem failure, fall back to copy+delete with backup of any pre-existing `dest`; restore backup on failure |
| `safe_remove_dir(path, confirmed)` | Returns `ValidationError` unless `confirmed=true` |

---

## 9. State & Caching

### 9.1 Cache file format (`src/core/cache.rs`)

| Constant | Value |
|----------|-------|
| `CACHE_VERSION` | `1` |
| `MAX_HASH_SIZE` | `64` (SHA-256 hex chars) |
| `MAX_CACHE_SIZE` | `15 * 1024 * 1024` bytes (15 MB) |

```json
{
  "version": 1,
  "entries": {
    "/abs/path/to/skill": {
      "mtime": 1706000000,
      "size": 4096,
      "content_hash": "abc123…",
      "cached_at": 1706000000,
      "skill_name": "my-skill",
      "is_valid_skill": true
    }
  }
}
```

In-memory representation:

```rust
pub struct CacheFile {
    pub version: u32,
    pub entries: BTreeMap<String, CachedEntry>,   // sorted for determinism
}

pub struct ScanEntry {                            // public API type (path is owned)
    pub path: PathBuf,
    pub mtime: u64,
    pub size: u64,
    pub content_hash: String,                     // ≤ MAX_HASH_SIZE
    pub cached_at: u64,
    pub skill_name: Option<String>,
    pub is_valid_skill: bool,
}
```

### 9.2 `Cache` trait

| Operation | Behavior |
|-----------|----------|
| `get(path)` | Return cached entry iff SKILL.md current `mtime` matches stored `mtime`; missing file or mismatch → `Ok(None)`; read/parse failure → `Ok(None)` (non-fatal) |
| `put(entry)` | Insert / replace; reject `content_hash.len() > 64` with `ValidationError`; write atomically |
| `invalidate(path)` | Remove entry and re-write |
| `clean_stale()` | Remove entries whose path no longer exists; return count |
| `clear()` | Remove every entry |

### 9.3 Invalidation rules

1. mtime mismatch on `get()`
2. SKILL.md missing on `get()`
3. `invalidate(path)` called explicitly
4. `clean_stale()` removes orphaned entries
5. `clear()` empties the cache
6. Loaded `version != CACHE_VERSION` → discard everything (rebuild)
7. File size > `MAX_CACHE_SIZE` on load → clear

### 9.4 Write semantics

- Write to `cache.json.tmp`, then `fs::rename` to `cache.json`.
- No locking. Last writer wins (acceptable: the worst case is a re-scan).
- All non-`put`-validation errors are silently swallowed; the operation continues without caching.

### 9.5 `--no-cache`

`--no-cache` constructs `Scanner::without_cache()` instead of `Scanner::new()`. The scanner then calls neither `get` nor `put`, forcing a fresh filesystem scan and a fresh re-parse of every SKILL.md.

---

## 10. Security & Safety

### 10.1 Skill name validation

Spec: `skill-model.md`, `skill-validation.md`.

- Regex `^[a-z0-9][a-z0-9_-]{0,63}$` enforced before any path is constructed.
- Rejects empty, path separators (`/`, `\`), traversal sequences (`.`, `..`).

### 10.2 Path-traversal protection

Spec: `git-operations.md`, `atomic-operations.md`.

- Subdirectory extraction rejects paths containing `..` or absolute paths.
- After extraction, the result is `canonicalize()`d and verified to lie within the clone root (`PathTraversal` otherwise).
- `copy_skill_dir` records files relative to `src` via `strip_prefix`; mismatch triggers `PathTraversal`.

### 10.3 Symlink policy

Spec: `atomic-operations.md`, `skill-installation.md`.

- Source skill payloads must be physical: `WalkDir::follow_links(false)`; any encountered symlink → `SymlinkNotAllowed`.
- `create_symlink` removes any existing file or symlink at the destination before linking; missing parents are created.
- Symlinks pointing outside `~/.sikil/repo/` are classified as foreign symlinks (not managed); broken symlinks are reported but never crash the scan.

### 10.4 Git operation hardening

Spec: `git-operations.md`.

- GitHub-only HTTPS in v1; `file://`, leading `-`, whitespace, NUL, and SSH (`git@…`) all rejected.
- `git clone -c protocol.file.allow=never --depth=1 --` invoked via array args (no shell).
- Pre-flight `git --version` confirms `git` is installed (`GitError "git is not installed"` otherwise).
- `.git/` is removed from the clone before copying anything into the managed repo.

### 10.5 Config hardening

Spec: `configuration.md`.

- `#[serde(deny_unknown_fields)]` on TOML structs.
- Files larger than 1 MB → `ConfigError::ConfigTooLarge`.
- `--set` validates field names (`enabled`, `global_path`, `workspace_path`); other fields rejected.

### 10.6 Cache safety

Spec: `cache.md`.

- Atomic temp + rename for every write.
- Version mismatch and 15 MB overflow trigger automatic full clear.
- `put()` rejects content hashes longer than 64 chars; all other failures are non-fatal so the cache never blocks an operation.

### 10.7 Confirmation policy for destructive operations

Spec: `skill-removal.md`, `skill-unmanagement.md`, `atomic-operations.md`.

- `[y/N]` prompt with default `no` for `remove` and `unmanage`.
- Bypass with `--yes` or `--json`.
- `safe_remove_dir(path, confirmed)` requires `confirmed=true` regardless of UI confirmation, providing a programmatic safety net.

---

## 11. Observability & UX

### 11.1 Output streams (`src/cli/output.rs`)

| Mode | stdout | stderr |
|------|--------|--------|
| Default (TTY) | Messages, colored `✓ / ⚠ / ✗`, command output | Errors |
| `--json` | JSON payloads only | Messages, errors, progress |
| Non-TTY | Messages without color | Errors |
| `NO_COLOR` set | Messages without color (Unicode icons retained) | Errors |

`MessageWriter` routes its write methods to stderr in JSON mode and to stdout otherwise.

### 11.2 Progress indicators

`Progress::new(json_mode, total: Option<u64>)`:

- `Some(total)` → bar
- `None` → spinner with template `{spinner:.green} {wide_msg}` and frames `⠁⠂⠄⡀⢀⠠⠐⠈`
- Disabled when `--json`, non-TTY, or when stdout is redirected.

### 11.3 Error messages

Spec: `error-handling.md`.

- All errors flow through `SikilError` (domain) and surface to the user as `Error: {message}` on stderr.
- Each variant carries enough context to suggest a fix; complex commands (install, sync) emit additional guidance ("To update the skill, first remove it: sikil remove …").
- Exit codes are mapped via `SikilError::exit_code()` (see §6.12).

---

## 12. Testing Strategy

Spec: `testing-strategy.md`.

### 12.1 Test layers

| Layer | Location | Tooling |
|-------|----------|---------|
| Unit | `#[cfg(test)]` blocks alongside source | `assert_eq!`, `tempfile::TempDir`, `fs-err` |
| Integration | `tests/<area>_test.rs` | `assert_cmd`, `predicates`, `sikil_cmd!` macro |
| End-to-end | `tests/e2e_test.rs` | Full subprocess flows |
| Snapshot | `src/core/snapshots/` and `tests/snapshots/` | `insta` (`yaml` feature) |

### 12.2 Required integration test files

`cli_framework_test.rs`, `config_command_test.rs`, `conflict_detection_test.rs`, `e2e_test.rs`, `error_handling_test.rs`, `filesystem_utils_test.rs`, `install_command_test.rs`, `list_command_test.rs`, `parser_integration_test.rs`, `performance_test.rs`, `show_command_test.rs`, `unmanage_command_test.rs`, `validate_command_test.rs`, `build_test.rs`.

### 12.3 Conventions

- Use `tempfile::TempDir` and set `HOME` to its path in subprocess calls; cleanup is automatic on drop.
- Use `fs_err` (not `std::fs`) inside tests.
- Helpers in `tests/common/mod.rs`: `sikil_cmd!`, `setup_temp_skill_dir`, `create_skill_dir`, `create_skill_md`, `create_minimal_skill_md`, `create_complete_skill_md`.
- Snapshot redaction: replace dynamic paths with `[SKILLS_DIR]`, `[HOME]`, `[REPO_DIR]` before snapshotting; review with `cargo insta review`.
- Fixtures live under `tests/fixtures/skills/{valid,invalid}/`.

### 12.4 Verification command

```
./scripts/verify.sh        # runs cargo test, cargo clippy -- -D warnings, cargo fmt --check
```

`AGENTS.md` requires this to pass before any task is marked complete.

---

## 13. Technical Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Agent path changes upstream | Medium | Medium | Per-agent overrides in config; documented defaults |
| Git auth failures | Medium | Medium | Shell out to `git` so credential helpers / SSH agents work |
| Symlink semantics on network drives or atypical filesystems | Low | Medium | Document limitation; classification surfaces broken/foreign symlinks rather than crashing |
| Large skill directories slow scan | Low | Low | JSON cache with mtime-based invalidation; `--no-cache` escape hatch |
| Cache file corruption | Low | Low | Version + size guard auto-clear; read failures fall through to a fresh scan |
| Subprocess injection via Git URL | Low | High | URL validation + `--` arg terminator + array-style invocation |
| Path traversal via subdirectory extraction | Low | High | `..` / absolute path rejection + canonicalization check |

---

## 14. Open Questions & Deferred Decisions

- **Concurrent invocations of Sikil.** Current design accepts last-writer-wins for the cache and assumes user-level serialization for repo writes. Should we introduce a lock file under `~/.sikil/` for `install` / `adopt` / `unmanage` / `remove`?
- **Multi-host Git support.** Spec restricts to GitHub HTTPS for security in v1. Spec evolution required to allow user-configurable hostname allow-lists or SSH formats.
- **Workspace scope writes.** `sync --to` and `install --to` always target the global agent path. A workspace-scope target (`.claude/skills` etc.) is implied by the data model but not yet exposed by the CLI.
- **Telemetry / opt-in usage analytics.** Currently a non-goal; a future opt-in mechanism would need a separate spec.
- **Workspace-aware adoption.** `adopt --from <agent>` does not currently disambiguate global vs workspace installations of the same skill; consider an additional `--scope` flag if real-world conflicts emerge.

---

## 15. Spec Cross-Reference

| Spec | Module(s) | Section in this TRD |
|------|-----------|---------------------|
| `skill-model.md` | `src/core/skill.rs`, `src/core/parser.rs` | §4 Domain Model |
| `agent-model.md` | `src/core/skill.rs`, `src/core/config.rs` | §4 Domain Model |
| `error-handling.md` | `src/core/errors.rs`, `src/main.rs` | §6.12 Exit codes, §11.3 Error messages |
| `skill-scanner.md` | `src/core/scanner.rs` | §6.1 list, §8.1 Scan algorithm |
| `skill-discovery.md` | `src/commands/list.rs`, `src/commands/show.rs` | §6.1 list, §6.2 show |
| `conflict-detection.md` | `src/core/conflicts.rs`, `src/commands/list.rs` | §6.1 list, §8.3 Conflicts |
| `skill-validation.md` | `src/commands/validate.rs`, `src/core/parser.rs` | §6.8 validate |
| `skill-installation.md` | `src/commands/install.rs`, `src/main.rs::is_git_url` | §6.3 install, §8.2 Install plan |
| `skill-adoption.md` | `src/commands/adopt.rs` | §6.4 adopt |
| `skill-removal.md` | `src/commands/remove.rs` | §6.6 remove |
| `skill-unmanagement.md` | `src/commands/unmanage.rs` | §6.5 unmanage |
| `skill-synchronization.md` | `src/commands/sync.rs` | §6.7 sync |
| `agent-targeting.md` | `src/commands/agent_selection.rs` | §6.3, §6.7 |
| `cli-schema.md` | `src/cli/app.rs`, `src/main.rs` | §6.11 Global flags, §6.12 Exit codes |
| `cli-output.md` | `src/cli/output.rs` | §11 Observability & UX |
| `configuration.md` | `src/core/config.rs`, `src/commands/config.rs` | §6.9 config, §10.5 Config hardening |
| `cache.md` | `src/core/cache.rs` | §9 State & Caching |
| `filesystem-paths.md` | `src/utils/paths.rs` | §5 Filesystem Contract |
| `symlink-operations.md` | `src/utils/symlink.rs` | §10.3 Symlink policy |
| `atomic-operations.md` | `src/utils/atomic.rs` | §8.6 Atomic operations |
| `git-operations.md` | `src/utils/git.rs` | §8.4 Git URL parsing, §8.5 Git clone, §10.4 Git hardening |
| `shell-completions.md` | `src/commands/completions.rs` | §6.10 completions |
| `build-and-platform.md` | `Cargo.toml`, `scripts/build.sh`, `build.rs` | §2 Goals & Constraints |
| `testing-strategy.md` | `tests/`, `src/core/snapshots/` | §12 Testing Strategy |

---

**Document Version**: 2.0
**Source of Truth**: `specs/`
