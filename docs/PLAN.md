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
| **Next candidates** | M1-E02-T03 |
| **In progress** | — |
| **Blocked** | — |
| **Recently completed** | M1-E01-T01, M1-E01-T02, M1-E01-T03, M1-E02-T01, M1-E02-T02 |

---

## STATE

> Machine-readable YAML block. Only list items that differ from default (default = todo/unclaimed/unverified).

```yaml
schema_version: 1
updated_at: "2026-01-17T13:05:00Z"

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
        - "S01: Agent enum already defined in src/core/skill.rs (from M1-E02-T01)"
        - "S02: Scope enum already defined in src/core/skill.rs (from M1-E02-T01)"
        - "S03: Installation struct already defined in src/core/skill.rs (from M1-E02-T01)"
        - "S04: Display trait implemented for Agent (agent.to_string() returns cli_name)"
        - "S05: 13 unit tests passing (12 from M1-E02-T01 + new test_agent_display)"

# Session log (append-only)
sessions:
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
    ended_at: "2026-01-17T13:05:00Z"
    by: "claude-code"
    worked_on:
      - id: "M1-E02-T02"
        outcome: "done"
    notes: "Implemented Display trait for Agent enum (uses cli_name for output). Added test_agent_display unit test. 13 tests passing."
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
