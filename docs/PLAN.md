# PLAN (Agent Execution State)

This file is the source of truth for implementation progress and multi-session continuity.

**Roadmap source**: [`specs/implementation_roadmap.md`](../specs/implementation_roadmap.md)

---

## How to Pick the Next Task

1. Consider only **tasks** (IDs like `M1-E02-T01`) for assignment.
2. A task is **eligible** if:
   - `status` is `todo` (or not listed in STATE), AND
   - all `[DEP: ...]` dependencies are `done`, AND
   - it is not `claimed` by another agent
3. Choose the eligible task with the **smallest lexical ID** (deterministic).
4. If no eligible tasks exist, pick a `blocked` task and resolve the blocker.

## Quick View

> Update this section when claiming/completing tasks.

| Status | Tasks |
|--------|-------|
| **Next candidates** | M1-E03-T03 |
| **In progress** | — |
| **Blocked** | — |
| **Recently completed** | M1-E01-T01, M1-E01-T02, M1-E01-T03, M1-E02-T01, M1-E02-T02, M1-E02-T03, M1-E03-T01, M1-E03-T02 |

---

## STATE

> Machine-readable YAML block. Only list items that differ from default (default = todo/unclaimed/unverified).

```yaml
schema_version: 1
updated_at: "2026-01-18T08:15:00Z"

roadmap:
  file: "specs/implementation_roadmap.md"
  id_format: "M{milestone}-E{epic}-T{task}-S{subtask}"

# Current focus for session continuity
focus:
  current_task: null
  current_subtask: null
  last_update_by: "claude-code"

# Claims prevent concurrent work on same task
claims: {}

# Task status entries (only list non-todo items)
# Status values: todo | in_progress | blocked | done
items:
  "M1-E01-T01":
    title: "Initialize Rust Project"
    status: "done"
    started_at: "2026-01-17T09:10:00Z"
    done_at: "2026-01-17T09:30:00Z"
    verification:
      status: "passed"
      at: "2026-01-17T09:30:00Z"
      commands:
        - "cargo test"
        - "cargo clippy -- -D warnings"
        - "cargo fmt --check"
      subtasks: ["S01", "S02", "S03", "S04", "S05"]
      evidence:
        - "S01: Cargo.toml exists at project root"
        - "S02: Package metadata complete (name, version, authors, license, edition)"
        - "S03: All 17 production deps added (clap, serde, rusqlite, sha2, etc.)"
        - "S04: All 3 dev deps added (assert_cmd, predicates, insta)"
        - "S05: .gitignore exists with /target"
  "M1-E01-T02":
    title: "Project Structure"
    status: "done"
    started_at: "2026-01-17T10:00:00Z"
    done_at: "2026-01-17T10:00:00Z"
    verification:
      status: "passed"
      at: "2026-01-17T10:00:00Z"
      commands:
        - "cargo build"
        - "cargo run -- --help"
      subtasks: ["S01", "S02", "S03"]
      evidence:
        - "S01: src/main.rs exists with clap setup, cargo run -- --help shows usage"
        - "S02: src/lib.rs exists for library exports, cargo build succeeds"
        - "S03: Module directories created (cli/, core/, commands/, utils/) with mod.rs files"
  "M1-E01-T03":
    title: "Setup Test Infrastructure"
    status: "done"
    started_at: "2026-01-17T11:00:00Z"
    done_at: "2026-01-17T11:15:00Z"
    verification:
      status: "passed"
      at: "2026-01-17T11:15:00Z"
      commands:
        - "cargo test"
        - "cargo clippy -- -D warnings"
        - "cargo fmt --check"
      subtasks: ["S01", "S02", "S03", "S04", "S05"]
      evidence:
        - "S01: tests/ directory created for integration tests"
        - "S02: tests/common/mod.rs created with test helpers, compiles successfully"
        - "S03: Helper functions setup_temp_skill_dir() and create_skill_dir() created, unit tests pass"
        - "S04: Helper functions create_skill_md(), create_minimal_skill_md(), create_complete_skill_md() created, tests pass"
        - "S05: cargo test runs successfully, 9 tests passed (5 unit tests in common/mod.rs + 4 integration tests)"
  "M1-E02-T01":
    title: "Define Skill Model"
    status: "done"
    started_at: "2026-01-17T12:00:00Z"
    done_at: "2026-01-17T12:15:00Z"
    verification:
      status: "passed"
      at: "2026-01-17T12:15:00Z"
      commands:
        - "cargo test"
        - "cargo clippy -- -D warnings"
        - "cargo fmt --check"
      subtasks: ["S01", "S02", "S03", "S04", "S05"]
      evidence:
        - "S01: src/core/skill.rs created with skill model types"
        - "S02: SkillMetadata struct defined with name, description, version, author, license fields"
        - "S03: Skill struct defined with metadata, directory_name, installations, is_managed, repo_path fields"
        - "S04: Serialize/Deserialize traits implemented for all structs (SkillMetadata, Skill, Installation, Agent, Scope)"
        - "S05: 12 unit tests written and passing (test_skill_metadata_new, test_skill_metadata_builder, test_skill_new, test_skill_with_installation, test_skill_with_repo, test_skill_is_orphan, test_agent_cli_name, test_agent_from_cli_name, test_agent_all, test_installation_new, test_scope_equality, test_serialization)"
  "M1-E02-T02":
    title: "Define Installation Model"
    status: "done"
    started_at: "2026-01-17T13:00:00Z"
    done_at: "2026-01-17T13:05:00Z"
    verification:
      status: "passed"
      at: "2026-01-17T13:05:00Z"
      commands:
        - "cargo test"
        - "cargo clippy -- -D warnings"
        - "cargo fmt --check"
      subtasks: ["S01", "S02", "S03", "S04", "S05"]
      evidence:
        - "S01: Agent enum defined in src/core/skill.rs (from M1-E02-T01)"
        - "S02: Scope enum defined in src/core/skill.rs (from M1-E02-T01)"
        - "S03: Installation struct completed with is_symlink and symlink_target fields (base from M1-E02-T01, fields added in M1-E02-T02)"
        - "S04: Display trait implemented for Agent (agent.to_string() returns cli_name)"
        - "S05: 14 unit tests passing (12 from M1-E02-T01 + test_agent_display + test_installation_with_symlink, test_installation_new updated)"
  "M1-E02-T03":
    title: "Define Error Types"
    status: "done"
    started_at: "2026-01-18T08:00:00Z"
    done_at: "2026-01-18T08:15:00Z"
    verification:
      status: "passed"
      at: "2026-01-18T08:15:00Z"
      commands:
        - "cargo test"
        - "cargo clippy -- -D warnings"
        - "cargo fmt --check"
      subtasks: ["S01", "S02", "S03", "S04", "S05", "S06", "S07"]
      evidence:
        - "S01: src/core/errors.rs created with SikilError enum"
        - "S02: SikilError enum defined with #[derive(Error, Debug)] using thiserror"
        - "S03: Error variants added: InvalidSkillMd, SkillNotFound, DirectoryNotFound"
        - "S04: Error variants added: SymlinkError, GitError, ConfigError"
        - "S05: Error variants added: AlreadyExists, PermissionDenied, ValidationError"
        - "S06: Security error variants added: PathTraversal, SymlinkNotAllowed, InvalidGitUrl, ConfigTooLarge"
        - "S07: 14 unit tests written and passing (test_error_display_invalid_skill_md, test_error_display_skill_not_found, test_error_display_directory_not_found, test_error_display_symlink_error, test_error_display_git_error, test_error_display_config_error, test_error_display_already_exists, test_error_display_permission_denied, test_error_display_validation_error, test_error_display_path_traversal, test_error_display_symlink_not_allowed, test_error_display_invalid_git_url, test_error_display_config_too_large, test_error_debug_format)"
  "M1-E03-T01":
    title: "Define Config Model"
    status: "done"
    started_at: "2026-01-18T09:00:00Z"
    done_at: "2026-01-18T09:10:00Z"
    verification:
      status: "passed"
      at: "2026-01-18T09:10:00Z"
      commands:
        - "cargo test"
        - "cargo clippy -- -D warnings"
        - "cargo fmt --check"
      subtasks: ["S01", "S02", "S03", "S04", "S05"]
      evidence:
        - "S01: src/core/config.rs created"
        - "S02: AgentConfig struct defined with enabled, global_path, workspace_path fields, compiles"
        - "S03: Config struct defined with HashMap<String, AgentConfig>, compiles"
        - "S04: Default trait implemented with hardcoded paths for all 5 agents (claude-code, windsurf, opencode, kilocode, amp), unit test passes"
        - "S05: 7 unit tests written and passing (test_agent_config_new, test_config_new, test_config_insert_and_get_agent, test_config_default_has_all_agents, test_config_default_all_agents_enabled, test_config_default_agent_paths, test_config_serialization)"
    notes: |
      FIX (2026-01-18): Corrected default agent paths to match TRD §Domain Model and official docs:
      - Claude Code: ~/.cache/claude-code/skills → ~/.claude/skills (per https://code.claude.com/docs/en/skills)
      - Windsurf: ~/.cache/windsurf/skills → ~/.codeium/windsurf/skills
      - OpenCode: ~/.cache/opencode/skills → ~/.config/opencode/skill
      - KiloCode: ~/.cache/kilocode/skills → ~/.kilocode/skills (per https://kilo.ai/docs/agent-behavior/skills)
      - Amp: ~/.cache/amp/skills → ~/.config/agents/skills
      Also fixed KiloCode agent name: kilo-code → kilocode (in TRD, skill.rs, config.rs)
      Verified via web search of official agent documentation.
  "M1-E03-T02":
    title: "Config File Loading"
    status: "done"
    started_at: "2026-01-18T09:30:00Z"
    done_at: "2026-01-18T09:50:00Z"
    verification:
      status: "passed"
      at: "2026-01-18T09:50:00Z"
      commands:
        - "cargo test"
        - "cargo clippy -- -D warnings"
        - "cargo fmt --check"
      subtasks: ["S01", "S02", "S03", "S04", "S05", "S06", "S07"]
      evidence:
        - "S01: toml crate already present in Cargo.toml dependencies"
        - "S02: Config::load() implemented, loads from ~/.sikil/config.toml, unit test passes (test_config_load_valid_toml)"
        - "S03: Fallback to defaults if file missing, unit test passes (test_config_load_missing_file_returns_defaults)"
        - "S04: Path expansion using shellexpand implemented in expand_path() helper, unit test passes (test_config_expand_paths)"
        - "S05: Config file size limit (1MB max) implemented and enforced, unit test passes (test_config_load_oversized_file_returns_error)"
        - "S06: #[serde(deny_unknown_fields)] added to AgentConfig and Config structs, unit test passes (test_config_deny_unknown_fields)"
        - "S07: Integration test with temp config file passes (all subtask tests pass)"

# Session log (append-only)
sessions:
  - started_at: "2026-01-18T09:30:00Z"
    ended_at: "2026-01-18T09:50:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E03-T02"
        outcome: "done"
    notes: "Implemented Config file loading with Config::load() method. Added ConfigError enum for config-specific errors. Implemented path expansion using shellexpand. Added file size limit (1MB max). Added deny_unknown_fields to config structs. Created 8 comprehensive unit tests covering valid TOML loading, missing file fallback, oversized files, invalid TOML, path expansion, and unknown fields. All 51 tests pass (42 lib + 9 integration)."
  - started_at: "2026-01-18T09:00:00Z"
    ended_at: "2026-01-18T09:10:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E03-T01"
        outcome: "done"
    notes: "Implemented Config model with AgentConfig struct and Default trait for all 5 agents. Created 7 unit tests for config creation, retrieval, defaults, and serialization. All tests passing."
   - started_at: "2026-01-17T09:10:00Z"
    ended_at: "2026-01-17T09:30:00Z"
    by: "amp-agent"
    worked_on:
      - id: "M1-E01-T01"
        outcome: "done"
    notes: "Completed project init with all deps. Fixed missing tempfile, once_cell, regex, sha2."
  - started_at: "2026-01-17T10:00:00Z"
    ended_at: "2026-01-17T10:15:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E01-T02"
        outcome: "done"
    notes: "Created project structure: main.rs with clap, lib.rs, and module directories (cli, core, commands, utils)."
  - started_at: "2026-01-17T11:00:00Z"
    ended_at: "2026-01-17T11:15:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E01-T03"
        outcome: "done"
    notes: "Created test infrastructure: tests/ directory, tests/common/mod.rs with helpers (setup_temp_skill_dir, create_skill_dir, create_skill_md, create_minimal_skill_md, create_complete_skill_md). All 9 tests pass."
  - started_at: "2026-01-17T12:00:00Z"
    ended_at: "2026-01-17T12:15:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E02-T01"
        outcome: "done"
    notes: "Created src/core/skill.rs with SkillMetadata, Skill, Installation, Agent, and Scope types. All structs implement Serialize/Deserialize. 12 unit tests passing."
  - started_at: "2026-01-17T13:00:00Z"
    ended_at: "2026-01-17T13:15:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E02-T02"
        outcome: "done"
    notes: "Implemented Display trait for Agent enum (uses cli_name for output). Completed Installation struct with is_symlink and symlink_target fields per roadmap spec. Added test_agent_display and test_installation_with_symlink unit tests. 14 tests passing."
  - started_at: "2026-01-18T08:00:00Z"
    ended_at: "2026-01-18T08:15:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E02-T03"
        outcome: "done"
    notes: "Created src/core/errors.rs with SikilError enum using thiserror. Added all 13 error variants (InvalidSkillMd, SkillNotFound, DirectoryNotFound, SymlinkError, GitError, ConfigError, AlreadyExists, PermissionDenied, ValidationError, PathTraversal, SymlinkNotAllowed, InvalidGitUrl, ConfigTooLarge). 14 unit tests passing. 28 tests total in codebase. Note: Initially created as error.rs, renamed to errors.rs for roadmap compliance (S01)."
```

---

## Status Definitions

| Status | Meaning |
|--------|---------|
| `todo` | Not started (default, no entry needed) |
| `in_progress` | Claimed and actively being worked on |
| `blocked` | Cannot proceed; include `blocked_reason` |
| `done` | Completed with verification |

## Verification Requirements

A task marked `done` MUST have:
- `verification.status: passed`
- `verification.commands`: list of commands run
- `verification.subtasks`: list of subtask IDs verified (e.g., `["S01", "S02", "S03"]`)
- `verification.evidence`: brief description confirming each subtask's "Verifiable By" criterion was met
