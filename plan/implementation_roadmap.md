# Sikil - Implementation Roadmap

## Overview

This document defines the implementation roadmap for **Sikil**, organized as Milestones → Epics → Tasks → Subtasks.

### Conventions

| Symbol | Meaning |
|--------|---------|
| `[DEP: X]` | Depends on task X (must complete X first) |
| `[L]` | Low complexity |
| `[N]` | Normal complexity |
| `[TEST]` | Test task (TDD) |
| `[DOC]` | Documentation task |
| `[WIRE]` | CLI wiring task (connects logic to CLI) |

### ID Format
`M{milestone}-E{epic}-T{task}-S{subtask}`

Example: `M1-E02-T03-S01` = Milestone 1, Epic 2, Task 3, Subtask 1

---

## Tech Stack Summary

### Core Dependencies
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
clap_complete = "4"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
toml = "0.8"
walkdir = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
anyhow = "1"
thiserror = "1"
anstream = "0.6"
anstyle = "1"
shellexpand = "3"
directories = "5"
fs-err = "2"
tempfile = "3"
once_cell = "1"
regex = "1"
sha2 = "0.10"              # Content hashing for cache

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
insta = { version = "1", features = ["yaml"] }
```

---

# Milestone 1: Foundation

**Goal**: Project setup, core types, infrastructure, and test harness.

---

## M1-E01: Project Setup

### M1-E01-T01: Initialize Rust Project `[L]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E01-T01-S01 | Run `cargo new sikil` | `Cargo.toml` exists |
| M1-E01-T01-S02 | Configure `Cargo.toml` with package metadata (name, version, authors, license, edition) | `cargo check` passes |
| M1-E01-T01-S03 | Add all production dependencies to `Cargo.toml` | `cargo build` succeeds |
| M1-E01-T01-S04 | Add all dev dependencies to `Cargo.toml` | `cargo test` runs (no tests yet) |
| M1-E01-T01-S05 | Create `.gitignore` for Rust project | File exists with `/target` |

### M1-E01-T02: Project Structure `[L]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E01-T02-S01 | Create `src/main.rs` with minimal clap setup | `cargo run -- --help` shows usage |
| M1-E01-T02-S02 | Create `src/lib.rs` for library exports | `cargo build` succeeds |
| M1-E01-T02-S03 | Create module structure directories | Directories exist |

```
src/
├── main.rs
├── lib.rs
├── cli/
│   └── mod.rs
├── core/
│   └── mod.rs
├── commands/
│   └── mod.rs
└── utils/
    └── mod.rs
```

### M1-E01-T03: Setup Test Infrastructure `[L]` `[TEST]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E01-T03-S01 | Create `tests/` directory for integration tests | Directory exists |
| M1-E01-T03-S02 | Create `tests/common/mod.rs` with test helpers | File compiles |
| M1-E01-T03-S03 | Create helper function to setup temp skill directories | Unit test passes |
| M1-E01-T03-S04 | Create helper function to create mock SKILL.md files | Unit test passes |
| M1-E01-T03-S05 | Verify `cargo test` runs successfully | Exit code 0 |

---

## M1-E02: Core Types & Models

### M1-E02-T01: Define Skill Model `[N]` `[DEP: M1-E01-T02]`
**Traces:** I: UC-02-01

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E02-T01-S01 | Create `src/core/skill.rs` | File exists |
| M1-E02-T01-S02 | Define `SkillMetadata` struct (name, description, version, author) | Compiles |
| M1-E02-T01-S03 | Define `Skill` struct (metadata, installations, is_managed) | Compiles |
| M1-E02-T01-S04 | Implement `Serialize`/`Deserialize` for structs | Compiles |
| M1-E02-T01-S05 | Write unit tests for struct creation | `cargo test` passes |

### M1-E02-T02: Define Installation Model `[N]` `[DEP: M1-E02-T01]`
**Traces:** I: UC-01-04, UC-01-05

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E02-T02-S01 | Define `Agent` enum (ClaudeCode, Windsurf, OpenCode, KiloCode, Amp) | Compiles |
| M1-E02-T02-S02 | Define `Scope` enum (Global, Workspace) | Compiles |
| M1-E02-T02-S03 | Define `Installation` struct (agent, path, scope, is_symlink, symlink_target) | Compiles |
| M1-E02-T02-S04 | Implement `Display` trait for `Agent` | `agent.to_string()` works |
| M1-E02-T02-S05 | Write unit tests for enums and struct | `cargo test` passes |

### M1-E02-T03: Define Error Types `[N]` `[DEP: M1-E01-T02]`
**Traces:** V: NFR-07, NFR-08, NFR-17

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E02-T03-S01 | Create `src/core/errors.rs` | File exists |
| M1-E02-T03-S02 | Define `SikilError` enum with `thiserror` | Compiles |
| M1-E02-T03-S03 | Add variants: `InvalidSkillMd`, `SkillNotFound`, `DirectoryNotFound` | Compiles |
| M1-E02-T03-S04 | Add variants: `SymlinkError`, `GitError`, `ConfigError` | Compiles |
| M1-E02-T03-S05 | Add variants: `AlreadyExists`, `PermissionDenied`, `ValidationError` | Compiles |
| M1-E02-T03-S06 | Add variants: `PathTraversal`, `SymlinkNotAllowed`, `InvalidGitUrl`, `ConfigTooLarge` | Compiles |
| M1-E02-T03-S07 | Write unit tests for error display messages | `cargo test` passes |

---

## M1-E03: Configuration System

### M1-E03-T01: Define Config Model `[N]` `[DEP: M1-E02-T02]`
**Traces:** I: UC-11-05, UC-11-06, UC-11-07

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E03-T01-S01 | Create `src/core/config.rs` | File exists |
| M1-E03-T01-S02 | Define `AgentConfig` struct (enabled, global_path, workspace_path) | Compiles |
| M1-E03-T01-S03 | Define `Config` struct with HashMap of agent configs | Compiles |
| M1-E03-T01-S04 | Implement `Default` trait with hardcoded agent paths | Unit test passes |
| M1-E03-T01-S05 | Write unit tests for default config values | `cargo test` passes |

### M1-E03-T02: Config File Loading `[N]` `[DEP: M1-E03-T01]`
**Traces:** I: UC-11-01, UC-11-08, UC-11-09 | V: NFR-25

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E03-T02-S01 | Add `toml` crate to dependencies | `cargo build` succeeds |
| M1-E03-T02-S02 | Implement `Config::load()` from `~/.sikil/config.toml` | Unit test passes |
| M1-E03-T02-S03 | Implement fallback to defaults if file missing | Unit test passes |
| M1-E03-T02-S04 | Implement path expansion using `shellexpand` | Unit test passes |
| M1-E03-T02-S05 | Add config file size limit (1MB max) | Unit test passes |
| M1-E03-T02-S06 | Add `#[serde(deny_unknown_fields)]` to config structs | Compiles |
| M1-E03-T02-S07 | Write integration test with temp config file | `cargo test` passes |

### M1-E03-T03: Test Config System `[L]` `[TEST]` `[DEP: M1-E03-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E03-T03-S01 | Test loading valid TOML config | Test passes |
| M1-E03-T03-S02 | Test invalid TOML returns error | Test passes |
| M1-E03-T03-S03 | Test partial config merges with defaults | Test passes |
| M1-E03-T03-S04 | Test disabled agent is excluded | Test passes |
| M1-E03-T03-S05 | Test config file over size limit returns error | Test passes |
| M1-E03-T03-S06 | Test unknown fields in config returns error | Test passes |

---

## M1-E04: SKILL.md Parser

### M1-E04-T01: Frontmatter Extraction `[N]` `[DEP: M1-E02-T01]`
**Traces:** I: UC-01-03, UC-09-03

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E04-T01-S01 | Create `src/core/parser.rs` | File exists |
| M1-E04-T01-S02 | Implement `extract_frontmatter(content: &str) -> Result<&str>` | Unit test passes |
| M1-E04-T01-S03 | Handle missing frontmatter (no `---` markers) | Returns error |
| M1-E04-T01-S04 | Handle malformed frontmatter (single `---`) | Returns error |
| M1-E04-T01-S05 | Write unit tests for valid/invalid frontmatter | `cargo test` passes |

### M1-E04-T02: Metadata Parsing `[N]` `[DEP: M1-E04-T01]`
**Traces:** I: UC-02-01, UC-09-04, UC-09-05

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E04-T02-S01 | Implement `parse_skill_md(path: &Path) -> Result<SkillMetadata>` | Unit test passes |
| M1-E04-T02-S02 | Parse required field: `name` | Unit test passes |
| M1-E04-T02-S03 | Parse required field: `description` | Unit test passes |
| M1-E04-T02-S04 | Parse optional fields: `version`, `author`, `license` | Unit test passes |
| M1-E04-T02-S05 | Return error if required fields missing | Unit test passes |

### M1-E04-T03: Name Validation `[N]` `[DEP: M1-E04-T02]`
**Traces:** I: UC-09-06, UC-09-07 | V: NFR-21

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E04-T03-S01 | Implement `validate_skill_name(name: &str) -> Result<()>` with regex `^[a-z0-9][a-z0-9_-]{0,63}$` | Unit test passes |
| M1-E04-T03-S02 | Validate: must start with lowercase letter or digit | Test with "-skill" fails |
| M1-E04-T03-S03 | Validate: alphanumeric, hyphens, underscores only | Test with "my.skill" fails |
| M1-E04-T03-S04 | Validate: max 64 characters | Test with 65-char name fails |
| M1-E04-T03-S05 | Validate: reject path separators (`/`, `\`) | Test with "my/skill" fails |
| M1-E04-T03-S06 | Validate: reject path traversal (`.`, `..`) | Test with ".." fails |
| M1-E04-T03-S07 | Validate: reject empty name | Test with "" fails |

### M1-E04-T04: Test Parser `[L]` `[TEST]` `[DEP: M1-E04-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E04-T04-S01 | Create test fixtures: valid SKILL.md files | Files exist in tests/fixtures/ |
| M1-E04-T04-S02 | Create test fixtures: invalid SKILL.md files | Files exist |
| M1-E04-T04-S03 | Integration test: parse valid skill | Test passes |
| M1-E04-T04-S04 | Integration test: parse skill with all optional fields | Test passes |
| M1-E04-T04-S05 | Integration test: parse skill with minimal fields | Test passes |
| M1-E04-T04-S06 | Snapshot test JSON output of parsed metadata | Snapshot matches |

---

## M1-E05: Filesystem Utilities

### M1-E05-T01: Path Utilities `[N]` `[DEP: M1-E03-T02]`
**Traces:** I: UC-03-11

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E05-T01-S01 | Create `src/utils/paths.rs` | File exists |
| M1-E05-T01-S02 | Implement `expand_path(path: &str) -> PathBuf` using shellexpand | Unit test passes |
| M1-E05-T01-S03 | Implement `get_repo_path() -> PathBuf` returning `~/.sikil/repo/` | Unit test passes |
| M1-E05-T01-S04 | Implement `get_config_path() -> PathBuf` returning `~/.sikil/config.toml` | Unit test passes |
| M1-E05-T01-S05 | Implement `ensure_dir_exists(path: &Path) -> Result<()>` | Unit test passes |

### M1-E05-T02: Symlink Utilities `[N]` `[DEP: M1-E05-T01]`
**Traces:** I: UC-01-10, UC-03-05, UC-04-09 | V: NFR-09

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E05-T02-S01 | Create `src/utils/symlink.rs` | File exists |
| M1-E05-T02-S02 | Implement `create_symlink(src: &Path, dest: &Path) -> Result<()>` | Unit test passes |
| M1-E05-T02-S03 | Implement `is_symlink(path: &Path) -> bool` | Unit test passes |
| M1-E05-T02-S04 | Implement `read_symlink_target(path: &Path) -> Result<PathBuf>` | Unit test passes |
| M1-E05-T02-S05 | Implement `resolve_realpath(path: &Path) -> Result<PathBuf>` | Unit test passes |
| M1-E05-T02-S06 | Implement `is_managed_symlink(path: &Path) -> bool` (target under ~/.sikil/repo/) | Unit test passes |

### M1-E05-T03: Atomic File Operations `[N]` `[DEP: M1-E05-T01]`
**Traces:** I: UC-04-13, UC-04-17, UC-05-07, UC-05-09 | V: NFR-06, NFR-23

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E05-T03-S01 | Create `src/utils/atomic.rs` | File exists |
| M1-E05-T03-S02 | Implement `copy_skill_dir(src: &Path, dest: &Path) -> Result<()>` that rejects symlinks | Unit test passes |
| M1-E05-T03-S03 | Implement `atomic_move_dir(src: &Path, dest: &Path) -> Result<()>` | Unit test passes |
| M1-E05-T03-S04 | Implement `safe_remove_dir(path: &Path) -> Result<()>` with confirmation check | Unit test passes |
| M1-E05-T03-S05 | Exclude `.git/` directory during copy | Unit test passes |
| M1-E05-T03-S06 | Write tests for rollback on failure | Unit test passes |

### M1-E05-T04: Test Filesystem Utilities `[L]` `[TEST]` `[DEP: M1-E05-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E05-T04-S01 | Test symlink creation and reading | Test passes |
| M1-E05-T04-S02 | Test symlink to non-existent target detection | Test passes |
| M1-E05-T04-S03 | Test atomic copy with temp directory | Test passes |
| M1-E05-T04-S04 | Test atomic move preserves content | Test passes |
| M1-E05-T04-S05 | Test permission error handling | Test passes |
| M1-E05-T04-S06 | Test `copy_skill_dir` rejects symlinks in source | Test passes |
| M1-E05-T04-S07 | Test `copy_skill_dir` excludes .git directory | Test passes |

---

## M1-E06: CLI Framework

### M1-E06-T01: Setup Clap Structure `[N]` `[DEP: M1-E01-T02]`
**Traces:** I: UC-13-01, UC-13-02, UC-13-03, UC-13-04, UC-13-05

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E06-T01-S01 | Create `src/cli/app.rs` with main `Cli` struct | Compiles |
| M1-E06-T01-S02 | Define `Commands` enum with all subcommands (stubs) | Compiles |
| M1-E06-T01-S03 | Add global flags: `--json`, `--verbose`, `--quiet` | `--help` shows flags |
| M1-E06-T01-S04 | Add `--version` flag | `sikil --version` shows version |
| M1-E06-T01-S05 | Wire up `main.rs` to use CLI parser | `cargo run -- --help` works |
| M1-E06-T01-S06 | Add usage examples to clap commands using `#[command(after_help)]` | `--help` shows Examples |

### M1-E06-T02: Output Formatting `[N]` `[DEP: M1-E06-T01]`
**Traces:** I: UC-01-09, UC-12-01, UC-12-03, UC-12-05 | V: NFR-16, NFR-18

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E06-T02-S01 | Create `src/cli/output.rs` | File exists |
| M1-E06-T02-S02 | Implement `Output` struct with json mode flag | Compiles |
| M1-E06-T02-S03 | Implement `print_success(msg)` with green color | Visual test |
| M1-E06-T02-S04 | Implement `print_warning(msg)` with yellow color | Visual test |
| M1-E06-T02-S05 | Implement `print_error(msg)` with red color | Visual test |
| M1-E06-T02-S06 | Implement `print_json(value)` for structured output | Unit test passes |
| M1-E06-T02-S07 | Respect `NO_COLOR` environment variable | Unit test passes |
| M1-E06-T02-S08 | Implement stderr for messages when `--json` is set (stdout for data only) | Unit test passes |
| M1-E06-T02-S09 | Add progress helper (wrapper around `indicatif`) with disable on non-TTY/`--json` | Unit test passes |

### M1-E06-T03: Test CLI Framework `[L]` `[TEST]` `[DEP: M1-E06-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E06-T03-S01 | Test `--help` output | `assert_cmd` test passes |
| M1-E06-T03-S02 | Test `--version` output | `assert_cmd` test passes |
| M1-E06-T03-S03 | Test unknown command error | Exit code non-zero |
| M1-E06-T03-S04 | Test `--json` flag parsing | Unit test passes |
| M1-E06-T03-S05 | Test `--json` emits valid JSON on stdout with no non-JSON noise | Integration test passes |
| M1-E06-T03-S06 | Test `sikil <cmd> --help` includes Examples section | Integration test passes |

---

## M1-E07: Caching System

### M1-E07-T01: Cache Storage & API `[N]` `[DEP: M1-E03-T02]`
**Traces:** V: NFR-02, NFR-03, NFR-26

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M1-E07-T01-S01 | Create `src/core/cache.rs` with `Cache` trait + `SqliteCache` impl | Compiles |
| M1-E07-T01-S02 | Define cache location (`~/.sikil/cache.sqlite`) via `utils/paths.rs` | Unit test passes |
| M1-E07-T01-S03 | Implement SQLite schema + migrations/init | Unit test passes |
| M1-E07-T01-S04 | Implement basic get/put for cached scan entries | Unit test passes |
| M1-E07-T01-S05 | Implement invalidation primitives (by mtime + size) | Unit test passes |
| M1-E07-T01-S06 | Write unit tests for cache CRUD + invalidation logic | `cargo test` passes |

---

# Milestone 2: Discovery

**Goal**: Implement scanning, listing, showing, and validation commands.

---

## M2-E01: Directory Scanner

### M2-E01-T01: Implement Scanner Core `[N]` `[DEP: M1-E04-T02, M1-E03-T02]`
**Traces:** I: UC-01-01, UC-01-02, UC-01-03

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E01-T01-S01 | Create `src/core/scanner.rs` | File exists |
| M2-E01-T01-S02 | Implement `Scanner` struct with config | Compiles |
| M2-E01-T01-S03 | Implement `scan_directory(path: &Path) -> Vec<SkillEntry>` | Unit test passes |
| M2-E01-T01-S04 | Parse SKILL.md for each subdirectory | Unit test passes |
| M2-E01-T01-S05 | Detect symlinks vs physical directories | Unit test passes |
| M2-E01-T01-S06 | Handle missing/invalid SKILL.md gracefully | Unit test passes |

### M2-E01-T02: Implement Multi-Agent Scanning `[N]` `[DEP: M2-E01-T01]`
**Traces:** I: UC-01-01, UC-01-02

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E01-T02-S01 | Implement `scan_all_agents() -> ScanResult` | Unit test passes |
| M2-E01-T02-S02 | Scan all global paths from config | Unit test passes |
| M2-E01-T02-S03 | Scan workspace paths relative to CWD | Unit test passes |
| M2-E01-T02-S04 | Scan `~/.sikil/repo/` for managed skills | Unit test passes |
| M2-E01-T02-S05 | Skip non-existent directories gracefully | Unit test passes |
| M2-E01-T02-S06 | Aggregate results by skill name | Unit test passes |

### M2-E01-T03: Managed/Unmanaged Classification `[N]` `[DEP: M2-E01-T02]`
**Traces:** I: UC-01-04, UC-01-05, UC-01-10

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E01-T03-S01 | Implement `classify_installation(path: &Path) -> InstallationType` | Unit test passes |
| M2-E01-T03-S02 | Return `Managed` if symlink target is under `~/.sikil/repo/` | Unit test passes |
| M2-E01-T03-S03 | Return `Unmanaged` if physical directory | Unit test passes |
| M2-E01-T03-S04 | Return `BrokenSymlink` if symlink target missing | Unit test passes |
| M2-E01-T03-S05 | Return `ForeignSymlink` if symlink to other location | Unit test passes |

### M2-E01-T04: Test Scanner `[N]` `[TEST]` `[DEP: M2-E01-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E01-T04-S01 | Setup test fixtures with mock agent directories | Fixtures exist |
| M2-E01-T04-S02 | Test scanning empty directory | Test passes |
| M2-E01-T04-S03 | Test scanning directory with skills | Test passes |
| M2-E01-T04-S04 | Test scanning with symlinks | Test passes |
| M2-E01-T04-S05 | Test scanning with broken symlinks | Test passes |
| M2-E01-T04-S06 | Test multi-agent scanning | Test passes |
| M2-E01-T04-S07 | Snapshot test scan result JSON | Snapshot matches |

### M2-E01-T05: Integrate Cache with Scanner `[N]` `[DEP: M2-E01-T01, M1-E07-T01]`
**Traces:** V: NFR-02

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E01-T05-S01 | Modify scanner to consult cache before walking filesystem | Unit test passes |
| M2-E01-T05-S02 | Update cache after fresh scan | Unit test passes |
| M2-E01-T05-S03 | Add `--no-cache` global flag to bypass cache | `--help` shows flag |
| M2-E01-T05-S04 | Integration test: second run uses cache (no filesystem walk) | Test passes |
| M2-E01-T05-S05 | Integration test: cached run is faster than uncached | Perf test passes |

---

## M2-E02: List Command

### M2-E02-T01: Implement List Logic `[N]` `[DEP: M2-E01-T03]`
**Traces:** I: UC-01-11, UC-01-12

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E02-T01-S01 | Create `src/commands/list.rs` | File exists |
| M2-E02-T01-S02 | Implement `execute_list(args: ListArgs) -> Result<()>` | Compiles |
| M2-E02-T01-S03 | Call scanner and get all skills | Unit test passes |
| M2-E02-T01-S04 | Group skills by managed/unmanaged | Unit test passes |
| M2-E02-T01-S05 | Format output with skill name, agents, scope | Unit test passes |
| M2-E02-T01-S06 | Show directory name if different from YAML name | Unit test passes |

### M2-E02-T02: Implement List Filters `[N]` `[DEP: M2-E02-T01]`
**Traces:** I: UC-01-06, UC-01-07, UC-01-08, UC-10-03, UC-10-04

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E02-T02-S01 | Implement `--agent <name>` filter | Unit test passes |
| M2-E02-T02-S02 | Implement `--managed` filter | Unit test passes |
| M2-E02-T02-S03 | Implement `--unmanaged` filter | Unit test passes |
| M2-E02-T02-S04 | Implement `--conflicts` filter | Unit test passes |
| M2-E02-T02-S05 | Implement `--duplicates` filter | Unit test passes |

### M2-E02-T03: Implement List Output `[N]` `[DEP: M2-E02-T02]`
**Traces:** I: UC-01-09

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E02-T03-S01 | Implement human-readable table output | Visual test |
| M2-E02-T03-S02 | Implement summary line (totals) | Visual test |
| M2-E02-T03-S03 | Implement `--json` output | Unit test passes |
| M2-E02-T03-S04 | Color managed skills green, unmanaged yellow | Visual test |

### M2-E02-T04: Wire List Command to CLI `[L]` `[WIRE]` `[DEP: M2-E02-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E02-T04-S01 | Add `ListArgs` struct with clap derives | Compiles |
| M2-E02-T04-S02 | Add `List` variant to `Commands` enum | Compiles |
| M2-E02-T04-S03 | Wire command in main dispatch | `sikil list --help` works |
| M2-E02-T04-S04 | Handle errors and display messages | Manual test |

### M2-E02-T05: Test List Command `[N]` `[TEST]` `[DEP: M2-E02-T04]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E02-T05-S01 | Integration test: list with no skills | Test passes |
| M2-E02-T05-S02 | Integration test: list with skills | Test passes |
| M2-E02-T05-S03 | Integration test: list --agent filter | Test passes |
| M2-E02-T05-S04 | Integration test: list --managed | Test passes |
| M2-E02-T05-S05 | Integration test: list --json | Test passes, valid JSON |
| M2-E02-T05-S06 | Snapshot test: list output | Snapshot matches |

---

## M2-E03: Show Command

### M2-E03-T01: Implement Show Logic `[N]` `[DEP: M2-E01-T03]`
**Traces:** I: UC-02-01, UC-02-02, UC-02-03, UC-02-04, UC-02-05, UC-02-09, UC-02-10

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E03-T01-S01 | Create `src/commands/show.rs` | File exists |
| M2-E03-T01-S02 | Implement `execute_show(name: &str) -> Result<()>` | Compiles |
| M2-E03-T01-S03 | Find skill by name across all installations | Unit test passes |
| M2-E03-T01-S04 | Aggregate all installations for the skill | Unit test passes |
| M2-E03-T01-S05 | Return error if skill not found | Unit test passes |

### M2-E03-T02: Implement Show Output `[N]` `[DEP: M2-E03-T01]`
**Traces:** I: UC-02-06, UC-02-07, UC-02-08

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E03-T02-S01 | Display metadata (name, description, version, author) | Visual test |
| M2-E03-T02-S02 | Display managed/unmanaged status | Visual test |
| M2-E03-T02-S03 | Display canonical path for managed skills | Visual test |
| M2-E03-T02-S04 | Display all installations with agent and path | Visual test |
| M2-E03-T02-S05 | Display file tree (SKILL.md, scripts/, references/) | Visual test |
| M2-E03-T02-S06 | Display total size | Unit test passes |
| M2-E03-T02-S07 | Implement `--json` output | Unit test passes |

### M2-E03-T03: Wire Show Command to CLI `[L]` `[WIRE]` `[DEP: M2-E03-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E03-T03-S01 | Add `ShowArgs` struct with skill name argument | Compiles |
| M2-E03-T03-S02 | Add `Show` variant to `Commands` enum | Compiles |
| M2-E03-T03-S03 | Wire command in main dispatch | `sikil show --help` works |

### M2-E03-T04: Test Show Command `[N]` `[TEST]` `[DEP: M2-E03-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E03-T04-S01 | Integration test: show existing skill | Test passes |
| M2-E03-T04-S02 | Integration test: show non-existent skill | Error message, non-zero exit |
| M2-E03-T04-S03 | Integration test: show --json | Test passes, valid JSON |
| M2-E03-T04-S04 | Snapshot test: show output | Snapshot matches |

---

## M2-E04: Validate Command

### M2-E04-T01: Implement Validation Logic `[N]` `[DEP: M1-E04-T03]`
**Traces:** I: UC-09-01, UC-09-02, UC-09-03, UC-09-04, UC-09-05, UC-09-08, UC-09-09

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E04-T01-S01 | Create `src/commands/validate.rs` | File exists |
| M2-E04-T01-S02 | Implement `execute_validate(path: &Path) -> Result<ValidationResult>` | Compiles |
| M2-E04-T01-S03 | Check SKILL.md exists | Unit test passes |
| M2-E04-T01-S04 | Check YAML frontmatter is valid | Unit test passes |
| M2-E04-T01-S05 | Check required fields present | Unit test passes |
| M2-E04-T01-S06 | Check name format constraints | Unit test passes |
| M2-E04-T01-S07 | Check description length (1-1024) | Unit test passes |

### M2-E04-T02: Implement Validation Output `[N]` `[DEP: M2-E04-T01]`
**Traces:** I: UC-09-10 | V: NFR-17

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E04-T02-S01 | Display checklist of validations (✓/✗) | Visual test |
| M2-E04-T02-S02 | Show warnings for missing optional fields | Visual test |
| M2-E04-T02-S03 | Show info for detected directories (scripts/, references/) | Visual test |
| M2-E04-T02-S04 | Display final PASSED/FAILED status | Visual test |
| M2-E04-T02-S05 | Exit code 0 on pass, non-zero on fail | Unit test passes |

### M2-E04-T03: Wire Validate Command to CLI `[L]` `[WIRE]` `[DEP: M2-E04-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E04-T03-S01 | Add `ValidateArgs` struct with path argument | Compiles |
| M2-E04-T03-S02 | Support path or installed skill name | Unit test passes |
| M2-E04-T03-S03 | Add `Validate` variant to `Commands` enum | Compiles |
| M2-E04-T03-S04 | Wire command in main dispatch | `sikil validate --help` works |

### M2-E04-T04: Test Validate Command `[N]` `[TEST]` `[DEP: M2-E04-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E04-T04-S01 | Integration test: validate valid skill | Exit 0 |
| M2-E04-T04-S02 | Integration test: validate missing SKILL.md | Exit non-zero |
| M2-E04-T04-S03 | Integration test: validate invalid name | Exit non-zero |
| M2-E04-T04-S04 | Integration test: validate missing required field | Exit non-zero |
| M2-E04-T04-S05 | Snapshot test: validation output | Snapshot matches |

---

## M2-E05: Conflict Detection

### M2-E05-T01: Implement Conflict Logic `[N]` `[DEP: M2-E01-T03]`
**Traces:** I: UC-10-01, UC-10-02, UC-10-07

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E05-T01-S01 | Create `src/core/conflicts.rs` | File exists |
| M2-E05-T01-S02 | Define `Conflict` struct (skill_name, locations, conflict_type) | Compiles |
| M2-E05-T01-S03 | Define `ConflictType` enum (DuplicateUnmanaged, DuplicateManaged) | Compiles |
| M2-E05-T01-S04 | Implement `detect_conflicts(scan_result: &ScanResult) -> Vec<Conflict>` | Unit test passes |
| M2-E05-T01-S05 | DuplicateUnmanaged: same name, multiple physical paths | Unit test passes |
| M2-E05-T01-S06 | DuplicateManaged: same name, all symlinks to same realpath | Unit test passes |

### M2-E05-T02: Implement Conflict Output `[N]` `[DEP: M2-E05-T01]`
**Traces:** I: UC-10-03, UC-10-04, UC-10-05, UC-10-06, UC-10-08

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E05-T02-S01 | Format conflict list with locations | Visual test |
| M2-E05-T02-S02 | Add recommendations for resolution | Visual test |
| M2-E05-T02-S03 | Integrate with `sikil list --conflicts` | Unit test passes |
| M2-E05-T02-S04 | Include in summary count | Unit test passes |

### M2-E05-T03: Test Conflict Detection `[N]` `[TEST]` `[DEP: M2-E05-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M2-E05-T03-S01 | Test no conflicts scenario | Test passes |
| M2-E05-T03-S02 | Test duplicate-unmanaged detection | Test passes |
| M2-E05-T03-S03 | Test duplicate-managed detection | Test passes |
| M2-E05-T03-S04 | Test mixed managed/unmanaged conflict | Test passes |
| M2-E05-T03-S05 | Snapshot test: conflict output | Snapshot matches |

---

# Milestone 3: Management

**Goal**: Implement install, adopt, unmanage, and remove commands.

---

## M3-E01: Install from Local Path

### M3-E01-T01: Implement Install Logic `[N]` `[DEP: M1-E05-T03, M2-E04-T01]`
**Traces:** I: UC-03-01, UC-03-02, UC-03-03, UC-03-05, UC-03-10, UC-03-11 | V: NFR-18

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E01-T01-S01 | Create `src/commands/install.rs` | File exists |
| M3-E01-T01-S02 | Implement `execute_install_local(path: &Path, agents: Vec<Agent>) -> Result<()>` | Compiles |
| M3-E01-T01-S03 | Validate source skill before install | Unit test passes |
| M3-E01-T01-S04 | Copy skill to `~/.sikil/repo/<name>/` | Unit test passes |
| M3-E01-T01-S05 | Create symlinks to specified agents | Unit test passes |
| M3-E01-T01-S06 | Create agent directories if missing | Unit test passes |
| M3-E01-T01-S07 | Show progress while copying skill directory (bytes/files) | Visual test |

### M3-E01-T02: Implement Install Guards `[N]` `[DEP: M3-E01-T01]`
**Traces:** I: UC-03-06, UC-03-07, UC-03-08 | V: NFR-06

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E01-T02-S01 | Check if skill name exists in repo, fail if so | Unit test passes |
| M3-E01-T02-S02 | Check if destination is physical dir, fail with adopt suggestion | Unit test passes |
| M3-E01-T02-S03 | Check if destination is symlink, fail as already installed | Unit test passes |
| M3-E01-T02-S04 | Rollback on partial failure | Unit test passes |

### M3-E01-T03: Implement Agent Selection `[N]` `[DEP: M3-E01-T01]`
**Traces:** I: UC-03-04

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E01-T03-S01 | Parse `--to` flag: `--to claude-code,windsurf` | Unit test passes |
| M3-E01-T03-S02 | Parse `--to all` for all enabled agents | Unit test passes |
| M3-E01-T03-S03 | Prompt user if `--to` not specified (interactive) | Manual test |
| M3-E01-T03-S04 | Validate agent names | Unit test passes |

### M3-E01-T04: Wire Install Command to CLI `[L]` `[WIRE]` `[DEP: M3-E01-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E01-T04-S01 | Add `InstallArgs` struct | Compiles |
| M3-E01-T04-S02 | Add `Install` variant to `Commands` enum | Compiles |
| M3-E01-T04-S03 | Wire command in main dispatch | `sikil install --help` works |

### M3-E01-T05: Test Install Local `[N]` `[TEST]` `[DEP: M3-E01-T04]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E01-T05-S01 | Integration test: install valid skill | Skill in repo, symlinks created |
| M3-E01-T05-S02 | Integration test: install to specific agents | Only specified agents have symlinks |
| M3-E01-T05-S03 | Integration test: install to all | All agents have symlinks |
| M3-E01-T05-S04 | Integration test: install duplicate fails | Error message |
| M3-E01-T05-S05 | Integration test: install over physical dir fails | Error with suggestion |
| M3-E01-T05-S06 | Integration test: install invalid skill fails | Validation error |

---

## M3-E02: Install from Git

### M3-E02-T01: Implement Git URL Parsing `[N]` `[DEP: M3-E01-T01]`
**Traces:** I: UC-04-01, UC-04-02, UC-04-06, UC-04-14, UC-04-15, UC-04-16 | V: NFR-22, NFR-24

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E02-T01-S01 | Create `src/utils/git.rs` | File exists |
| M3-E02-T01-S02 | Implement `parse_git_url(input: &str) -> Result<ParsedGitUrl>` | Unit test passes |
| M3-E02-T01-S03 | Parse short form: `user/repo` → `https://github.com/user/repo.git` | Unit test passes |
| M3-E02-T01-S04 | Parse short form with subdir: `user/repo/path/to/skill` | Unit test passes |
| M3-E02-T01-S05 | Parse HTTPS URL: `https://github.com/user/repo.git` | Unit test passes |
| M3-E02-T01-S06 | Parse HTTPS URL without .git suffix | Unit test passes |
| M3-E02-T01-S07 | Reject `file://` protocol with error | Unit test passes |
| M3-E02-T01-S08 | Reject URLs with whitespace or NUL characters | Unit test passes |
| M3-E02-T01-S09 | Reject URLs starting with `-` (argument injection) | Unit test passes |
| M3-E02-T01-S10 | Reject non-GitHub URLs with clear error message | Unit test passes |

### M3-E02-T02: Implement Git Clone `[N]` `[DEP: M3-E02-T01]`
**Traces:** I: UC-04-05, UC-04-10 | V: NFR-15, NFR-18

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E02-T02-S01 | Implement `clone_repo(url: &ParsedGitUrl, dest: &Path) -> Result<()>` | Unit test passes |
| M3-E02-T02-S02 | Use `std::process::Command` with array args (no shell) | Code review |
| M3-E02-T02-S03 | Use `--` separator before URL to prevent option injection | Unit test passes |
| M3-E02-T02-S04 | Use `-c protocol.file.allow=never` to block file protocol | Unit test passes |
| M3-E02-T02-S05 | Use `--depth=1` for shallow clone | Unit test passes |
| M3-E02-T02-S06 | Check `git` is installed, error if not | Unit test passes |
| M3-E02-T02-S07 | Add progress indicator during git clone | Visual test |

### M3-E02-T03: Implement Subdirectory Extraction `[N]` `[DEP: M3-E02-T02]`
**Traces:** I: UC-04-04

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E02-T03-S01 | Clone to temp directory using `tempfile::tempdir()` | Unit test passes |
| M3-E02-T03-S02 | If subdirectory specified in URL, validate it exists | Unit test passes |
| M3-E02-T03-S03 | Extract subdirectory to separate temp location | Unit test passes |
| M3-E02-T03-S04 | Validate extracted path is within clone root (no traversal) | Unit test passes |
| M3-E02-T03-S05 | Clean up clone (remove .git/, temp files) | Unit test passes |

### M3-E02-T04: Implement Git Install Flow `[N]` `[DEP: M3-E02-T03]`
**Traces:** I: UC-04-07, UC-04-08, UC-04-09, UC-04-11, UC-04-12

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E02-T04-S01 | Implement `execute_install_git(url: &str, agents: Vec<Agent>) -> Result<()>` | Compiles |
| M3-E02-T04-S02 | Clone repo to temp | Unit test passes |
| M3-E02-T04-S03 | Extract skill (root or subdirectory) | Unit test passes |
| M3-E02-T04-S04 | Validate extracted skill (SKILL.md, no symlinks) | Unit test passes |
| M3-E02-T04-S05 | Copy to repo using `copy_skill_dir` (rejects symlinks) | Unit test passes |
| M3-E02-T04-S06 | Create symlinks to agents | Unit test passes |
| M3-E02-T04-S07 | Clean up temp directory | Unit test passes |

### M3-E02-T05: Test Install Git `[N]` `[TEST]` `[DEP: M3-E02-T04]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E02-T05-S01 | Create local git repo fixture for testing | Fixture exists |
| M3-E02-T05-S02 | Integration test: install from short form `user/repo` | Skill installed |
| M3-E02-T05-S03 | Integration test: install with subdirectory | Correct skill installed |
| M3-E02-T05-S04 | Integration test: install non-existent subdirectory | Error message |
| M3-E02-T05-S05 | Integration test: git not installed | Clear error message |
| M3-E02-T05-S06 | Integration test: reject skill containing symlinks | Error message |
| M3-E02-T05-S07 | Integration test: reject invalid URL formats | Error message |

---

## M3-E03: Adopt Command

### M3-E03-T01: Implement Adopt Logic `[N]` `[DEP: M1-E05-T03, M2-E01-T03]`
**Traces:** I: UC-05-01, UC-05-02, UC-05-03, UC-05-04, UC-05-05

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E03-T01-S01 | Create `src/commands/adopt.rs` | File exists |
| M3-E03-T01-S02 | Implement `execute_adopt(name: &str, from_agent: Option<Agent>) -> Result<()>` | Compiles |
| M3-E03-T01-S03 | Find unmanaged skill by name | Unit test passes |
| M3-E03-T01-S04 | If multiple locations, require `--from` | Unit test passes |
| M3-E03-T01-S05 | Move skill to `~/.sikil/repo/<name>/` | Unit test passes |
| M3-E03-T01-S06 | Replace original with symlink | Unit test passes |

### M3-E03-T02: Implement Adopt Guards `[N]` `[DEP: M3-E03-T01]`
**Traces:** I: UC-05-06, UC-05-09

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E03-T02-S01 | Check skill is unmanaged | Error if already managed |
| M3-E03-T02-S02 | Check skill name not in repo | Error if exists |
| M3-E03-T02-S03 | Atomic move with rollback on failure | Unit test passes |
| M3-E03-T02-S04 | Preserve permissions and structure | Unit test passes |

### M3-E03-T03: Wire Adopt Command to CLI `[L]` `[WIRE]` `[DEP: M3-E03-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E03-T03-S01 | Add `AdoptArgs` struct with name and `--from` | Compiles |
| M3-E03-T03-S02 | Add `Adopt` variant to `Commands` enum | Compiles |
| M3-E03-T03-S03 | Wire command in main dispatch | `sikil adopt --help` works |

### M3-E03-T04: Test Adopt Command `[N]` `[TEST]` `[DEP: M3-E03-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E03-T04-S01 | Integration test: adopt single unmanaged skill | Skill in repo, symlink created |
| M3-E03-T04-S02 | Integration test: adopt with --from | Correct source adopted |
| M3-E03-T04-S03 | Integration test: adopt multiple locations without --from | Error with locations |
| M3-E03-T04-S04 | Integration test: adopt already managed | Error message |
| M3-E03-T04-S05 | Integration test: adopt non-existent skill | Error message |

---

## M3-E04: Unmanage Command

### M3-E04-T01: Implement Unmanage Logic `[N]` `[DEP: M1-E05-T03, M2-E01-T03]`
**Traces:** I: UC-06-01, UC-06-02, UC-06-03, UC-06-04, UC-06-05, UC-06-06

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E04-T01-S01 | Create `src/commands/unmanage.rs` | File exists |
| M3-E04-T01-S02 | Implement `execute_unmanage(name: &str, agent: Option<Agent>) -> Result<()>` | Compiles |
| M3-E04-T01-S03 | Find managed skill by name | Unit test passes |
| M3-E04-T01-S04 | If `--agent` specified, unmanage only that agent | Unit test passes |
| M3-E04-T01-S05 | Remove symlink, copy content from repo to location | Unit test passes |
| M3-E04-T01-S06 | If all symlinks removed, delete from repo | Unit test passes |

### M3-E04-T02: Implement Unmanage Confirmation `[N]` `[DEP: M3-E04-T01]`
**Traces:** I: UC-06-07, UC-06-08, UC-06-09 | V: NFR-05

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E04-T02-S01 | Display affected locations before action | Visual test |
| M3-E04-T02-S02 | Prompt for confirmation (y/N) | Manual test |
| M3-E04-T02-S03 | Implement `--yes` flag to skip confirmation | Unit test passes |
| M3-E04-T02-S04 | Cancel on 'n' or Ctrl+C | Manual test |

### M3-E04-T03: Wire Unmanage Command to CLI `[L]` `[WIRE]` `[DEP: M3-E04-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E04-T03-S01 | Add `UnmanageArgs` struct with name, `--agent`, `--yes` | Compiles |
| M3-E04-T03-S02 | Add `Unmanage` variant to `Commands` enum | Compiles |
| M3-E04-T03-S03 | Wire command in main dispatch | `sikil unmanage --help` works |

### M3-E04-T04: Test Unmanage Command `[N]` `[TEST]` `[DEP: M3-E04-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E04-T04-S01 | Integration test: unmanage all agents | Symlinks removed, copies created, repo deleted |
| M3-E04-T04-S02 | Integration test: unmanage specific agent | Only that symlink removed |
| M3-E04-T04-S03 | Integration test: unmanage with --yes | No prompt |
| M3-E04-T04-S04 | Integration test: unmanage unmanaged skill | Error message |

---

## M3-E05: Remove Command

### M3-E05-T01: Implement Remove Logic `[N]` `[DEP: M1-E05-T03, M2-E01-T03]`
**Traces:** I: UC-07-01, UC-07-02, UC-07-03, UC-07-04, UC-07-05, UC-07-06, UC-07-10

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E05-T01-S01 | Create `src/commands/remove.rs` | File exists |
| M3-E05-T01-S02 | Implement `execute_remove(name: &str, agents: Option<Vec<Agent>>, all: bool) -> Result<()>` | Compiles |
| M3-E05-T01-S03 | Require `--agent` or `--all` (no default) | Error if neither |
| M3-E05-T01-S04 | If `--agent`, remove symlink from specified agents | Unit test passes |
| M3-E05-T01-S05 | If `--all`, remove all symlinks AND repo entry | Unit test passes |
| M3-E05-T01-S06 | Support removing unmanaged skills (delete physical dir) | Unit test passes |

### M3-E05-T02: Implement Remove Confirmation `[N]` `[DEP: M3-E05-T01]`
**Traces:** I: UC-07-07, UC-07-08, UC-07-09, UC-07-11 | V: NFR-05

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E05-T02-S01 | Display what will be removed | Visual test |
| M3-E05-T02-S02 | Prompt for confirmation | Manual test |
| M3-E05-T02-S03 | Implement `--yes` flag | Unit test passes |
| M3-E05-T02-S04 | After `--agent` removal, prompt if no symlinks remain | Manual test |

### M3-E05-T03: Wire Remove Command to CLI `[L]` `[WIRE]` `[DEP: M3-E05-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E05-T03-S01 | Add `RemoveArgs` struct with name, `--agent`, `--all`, `--yes` | Compiles |
| M3-E05-T03-S02 | Add `Remove` variant to `Commands` enum | Compiles |
| M3-E05-T03-S03 | Wire command in main dispatch | `sikil remove --help` works |

### M3-E05-T04: Test Remove Command `[N]` `[TEST]` `[DEP: M3-E05-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M3-E05-T04-S01 | Integration test: remove --agent | Only specified symlink removed |
| M3-E05-T04-S02 | Integration test: remove --all managed | All symlinks and repo deleted |
| M3-E05-T04-S03 | Integration test: remove --all unmanaged | Physical dir deleted |
| M3-E05-T04-S04 | Integration test: remove without flags | Error requiring flag |
| M3-E05-T04-S05 | Integration test: remove non-existent skill | Error message |

---

# Milestone 4: Sync & Config

**Goal**: Implement sync and config commands.

---

## M4-E01: Sync Command

### M4-E01-T01: Implement Sync Logic `[N]` `[DEP: M3-E01-T01, M2-E01-T03]`
**Traces:** I: UC-08-01, UC-08-02, UC-08-03, UC-08-04

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E01-T01-S01 | Create `src/commands/sync.rs` | File exists |
| M4-E01-T01-S02 | Implement `execute_sync(name: Option<&str>, all: bool, to: Option<Vec<Agent>>) -> Result<()>` | Compiles |
| M4-E01-T01-S03 | Find managed skill in repo | Error if not managed |
| M4-E01-T01-S04 | Identify agents missing the skill | Unit test passes |
| M4-E01-T01-S05 | Create symlinks to missing agents | Unit test passes |
| M4-E01-T01-S06 | Skip agents that already have symlink | Unit test passes |

### M4-E01-T02: Implement Sync All `[N]` `[DEP: M4-E01-T01]`
**Traces:** I: UC-08-06

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E01-T02-S01 | Implement `--all` flag to sync all managed skills | Unit test passes |
| M4-E01-T02-S02 | Iterate all skills in `~/.sikil/repo/` | Unit test passes |
| M4-E01-T02-S03 | Display summary of synced skills | Visual test |

### M4-E01-T03: Implement Sync Guards `[N]` `[DEP: M4-E01-T01]`
**Traces:** I: UC-08-05, UC-08-07, UC-08-08, UC-08-09, UC-08-10

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E01-T03-S01 | If agent has unmanaged skill with same name, fail | Error with adopt suggestion |
| M4-E01-T03-S02 | Implement `--to` flag for specific agents | Unit test passes |
| M4-E01-T03-S03 | Display "already synced" if nothing to do | Visual test |

### M4-E01-T04: Wire Sync Command to CLI `[L]` `[WIRE]` `[DEP: M4-E01-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E01-T04-S01 | Add `SyncArgs` struct with name, `--all`, `--to` | Compiles |
| M4-E01-T04-S02 | Add `Sync` variant to `Commands` enum | Compiles |
| M4-E01-T04-S03 | Wire command in main dispatch | `sikil sync --help` works |

### M4-E01-T05: Test Sync Command `[N]` `[TEST]` `[DEP: M4-E01-T04]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E01-T05-S01 | Integration test: sync single skill | Missing agents get symlinks |
| M4-E01-T05-S02 | Integration test: sync --all | All managed skills synced |
| M4-E01-T05-S03 | Integration test: sync --to specific agents | Only those agents synced |
| M4-E01-T05-S04 | Integration test: sync already synced | "Already synced" message |
| M4-E01-T05-S05 | Integration test: sync with blocking unmanaged | Error with suggestion |

---

## M4-E02: Config Command

### M4-E02-T01: Implement Config Display `[N]` `[DEP: M1-E03-T02]`
**Traces:** I: UC-11-02

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E02-T01-S01 | Create `src/commands/config.rs` | File exists |
| M4-E02-T01-S02 | Implement `execute_config_show() -> Result<()>` | Compiles |
| M4-E02-T01-S03 | Display current config (from file or defaults) | Visual test |
| M4-E02-T01-S04 | Indicate which values are default vs overridden | Visual test |
| M4-E02-T01-S05 | Display config file path | Visual test |

### M4-E02-T02: Implement Config Edit `[N]` `[DEP: M4-E02-T01]`
**Traces:** I: UC-11-03

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E02-T02-S01 | Implement `--edit` flag | Compiles |
| M4-E02-T02-S02 | Create default config file if missing | Unit test passes |
| M4-E02-T02-S03 | Open config in `$EDITOR` (fallback to `vi`) | Manual test |
| M4-E02-T02-S04 | Validate config after edit | Unit test passes |

### M4-E02-T03: Implement Config Set `[N]` `[DEP: M4-E02-T01]`
**Traces:** I: UC-11-04

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E02-T03-S01 | Implement `set <key> <value>` subcommand | Compiles |
| M4-E02-T03-S02 | Parse dotted keys: `agents.claude-code.global_path` | Unit test passes |
| M4-E02-T03-S03 | Update config file | Unit test passes |
| M4-E02-T03-S04 | Create config file if missing | Unit test passes |
| M4-E02-T03-S05 | Validate key exists | Error on invalid key |

### M4-E02-T04: Wire Config Command to CLI `[L]` `[WIRE]` `[DEP: M4-E02-T03]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E02-T04-S01 | Add `ConfigArgs` struct with subcommands | Compiles |
| M4-E02-T04-S02 | Add `Config` variant to `Commands` enum | Compiles |
| M4-E02-T04-S03 | Wire command in main dispatch | `sikil config --help` works |

### M4-E02-T05: Test Config Command `[N]` `[TEST]` `[DEP: M4-E02-T04]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M4-E02-T05-S01 | Integration test: config (show) | Displays config |
| M4-E02-T05-S02 | Integration test: config set | Value updated |
| M4-E02-T05-S03 | Integration test: config with no file | Defaults shown |
| M4-E02-T05-S04 | Integration test: config set creates file | File created |

---

# Milestone 5: Polish & Release

**Goal**: Shell completions, documentation, and release artifacts.

---

## M5-E01: Shell Completions

### M5-E01-T01: Generate Completions `[N]` `[DEP: M1-E06-T01]`
**Traces:** I: UC-13-01, UC-13-02, UC-13-03, UC-13-04, UC-13-05

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E01-T01-S01 | Add `completions` subcommand | Compiles |
| M5-E01-T01-S02 | Generate bash completions using `clap_complete` | Output valid bash |
| M5-E01-T01-S03 | Generate zsh completions | Output valid zsh |
| M5-E01-T01-S04 | Generate fish completions | Output valid fish |
| M5-E01-T01-S05 | Output to stdout or `--output <file>` | Unit test passes |

### M5-E01-T02: Wire Completions Command `[L]` `[WIRE]` `[DEP: M5-E01-T01]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E01-T02-S01 | Add `CompletionsArgs` with shell type | Compiles |
| M5-E01-T02-S02 | Add `Completions` variant to `Commands` enum | Compiles |
| M5-E01-T02-S03 | Wire command in main dispatch | `sikil completions --help` works |

### M5-E01-T03: Test Completions `[L]` `[TEST]` `[DEP: M5-E01-T02]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E01-T03-S01 | Test bash output is non-empty | Test passes |
| M5-E01-T03-S02 | Test zsh output is non-empty | Test passes |
| M5-E01-T03-S03 | Test fish output is non-empty | Test passes |

---

## M5-E02: Documentation

### M5-E02-T01: README.md `[N]` `[DOC]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E02-T01-S01 | Write project overview | File exists |
| M5-E02-T01-S02 | Write installation instructions | Section complete |
| M5-E02-T01-S03 | Write quick start guide | Section complete |
| M5-E02-T01-S04 | Document all commands with examples | Section complete |
| M5-E02-T01-S05 | Add supported agents table | Section complete |
| M5-E02-T01-S06 | Document JSON output schema for `list`, `show`, `validate` commands | Section complete |

### M5-E02-T02: CONTRIBUTING.md `[L]` `[DOC]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E02-T02-S01 | Write development setup instructions | File exists |
| M5-E02-T02-S02 | Write testing instructions | Section complete |
| M5-E02-T02-S03 | Write code style guidelines | Section complete |

### M5-E02-T03: CHANGELOG.md `[L]` `[DOC]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E02-T03-S01 | Create changelog with Keep a Changelog format | File exists |
| M5-E02-T03-S02 | Document v0.1.0 features | Section complete |

### M5-E02-T04: Man Page `[N]` `[DOC]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E02-T04-S01 | Add `clap_mangen` dependency | Compiles |
| M5-E02-T04-S02 | Generate man page from clap definitions | Man page file exists |
| M5-E02-T04-S03 | Add build script to generate man page | `cargo build` generates man |

---

## M5-E03: Build & Release

### M5-E03-T01: Release Build Configuration `[N]`
**Traces:** V: NFR-04

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E03-T01-S01 | Configure release profile in `Cargo.toml` (optimize, strip) | Config present |
| M5-E03-T01-S02 | Add `[profile.release]` with `lto = true` | Config present |
| M5-E03-T01-S03 | Add `strip = true` for smaller binary | Config present |
| M5-E03-T01-S04 | Test release build: `cargo build --release` | Binary exists |
| M5-E03-T01-S05 | Verify binary size is reasonable (<10MB) | Size check |

### M5-E03-T02: Version Management `[N]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E03-T02-S01 | Set initial version to `0.1.0` in Cargo.toml | Version set |
| M5-E03-T02-S02 | Verify `sikil --version` shows correct version | Manual test |
| M5-E03-T02-S03 | Document version bump process in CONTRIBUTING.md | Section exists |

### M5-E03-T03: Build Scripts `[N]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E03-T03-S01 | Create `scripts/build.sh` for release build | Script exists |
| M5-E03-T03-S02 | Build for current platform | Binary works |
| M5-E03-T03-S03 | Create `scripts/test.sh` to run all tests | Script exists |
| M5-E03-T03-S04 | Create `scripts/install.sh` to install locally | Script works |

### M5-E03-T04: Cross-Platform Builds `[N]`
**Traces:** V: NFR-10, NFR-11, NFR-12, NFR-13

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E03-T04-S01 | Document build for macOS x86_64 | Docs exist |
| M5-E03-T04-S02 | Document build for macOS aarch64 (Apple Silicon) | Docs exist |
| M5-E03-T04-S03 | Document build for Linux x86_64 | Docs exist |
| M5-E03-T04-S04 | Document build for Linux aarch64 | Docs exist |
| M5-E03-T04-S05 | Test at least one cross-compile | Binary exists |

---

## M5-E04: Final Testing & QA

### M5-E04-T01: End-to-End Testing `[N]` `[TEST]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E04-T01-S01 | E2E test: full install → list → show → sync → remove flow | Test passes |
| M5-E04-T01-S02 | E2E test: adopt → unmanage flow | Test passes |
| M5-E04-T01-S03 | E2E test: conflict detection flow | Test passes |
| M5-E04-T01-S04 | E2E test: git install flow (with local git repo) | Test passes |

### M5-E04-T02: Error Handling Review `[N]` `[TEST]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E04-T02-S01 | Test all error paths have clear messages | Review complete |
| M5-E04-T02-S02 | Test permission denied scenarios | Test passes |
| M5-E04-T02-S03 | Test missing directory scenarios | Test passes |
| M5-E04-T02-S04 | Test broken symlink scenarios | Test passes |

### M5-E04-T03: Performance Testing `[L]` `[TEST]`

| ID | Subtask | Verifiable By |
|----|---------|---------------|
| M5-E04-T03-S01 | Benchmark `sikil list` with 50 skills | <500ms |
| M5-E04-T03-S02 | Benchmark `sikil show` | <200ms |
| M5-E04-T03-S03 | Document performance characteristics | Docs exist |

---

# Dependency Graph Summary

```
M1 (Foundation)
├── M1-E01 Project Setup (no deps)
├── M1-E02 Core Types → M1-E01
├── M1-E03 Config → M1-E02
├── M1-E04 Parser → M1-E02
├── M1-E05 Filesystem → M1-E03
├── M1-E06 CLI Framework → M1-E01
└── M1-E07 Caching System → M1-E03

M2 (Discovery) → M1
├── M2-E01 Scanner → M1-E04, M1-E03
│   └── M2-E01-T05 Cache Integration → M2-E01-T01, M1-E07-T01
├── M2-E02 List → M2-E01
├── M2-E03 Show → M2-E01
├── M2-E04 Validate → M1-E04
└── M2-E05 Conflicts → M2-E01

M3 (Management) → M2
├── M3-E01 Install Local → M1-E05, M2-E04
├── M3-E02 Install Git → M3-E01
├── M3-E03 Adopt → M1-E05, M2-E01
├── M3-E04 Unmanage → M1-E05, M2-E01
└── M3-E05 Remove → M1-E05, M2-E01

M4 (Sync & Config) → M3
├── M4-E01 Sync → M3-E01, M2-E01
└── M4-E02 Config → M1-E03

M5 (Polish) → M4
├── M5-E01 Completions → M1-E06
├── M5-E02 Documentation (no deps)
├── M5-E03 Build (no deps)
└── M5-E04 Final Testing → All
```

---

# Task Count Summary

| Milestone | Epics | Tasks | Subtasks |
|-----------|-------|-------|----------|
| M1: Foundation | 7 | 21 | 118 |
| M2: Discovery | 5 | 19 | 80 |
| M3: Management | 5 | 19 | 88 |
| M4: Sync & Config | 2 | 10 | 42 |
| M5: Polish | 4 | 13 | 46 |
| **Total** | **23** | **82** | **374** |

*Note: Task counts updated to reflect security hardening additions (v1.1)*

---

**Document Version**: 1.2  
**Created**: January 16, 2026  
**Updated**: January 16, 2026  
**Status**: Ready for implementation
