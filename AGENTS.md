# Agent Execution Guidance

## Execution Workflow (PLAN.md)

**PLAN.md** ([docs/PLAN.md](docs/PLAN.md)) is the execution state file. Follow this workflow:

### Start of Session
1. Read `docs/PLAN.md` to understand current progress
2. Parse the `STATE` YAML block to find task statuses
3. Select the next eligible task (smallest ID where deps are `done` and not claimed)
4. **Read the task's subtasks** in `specs/implementation_roadmap.md` before starting
5. Claim the task:
   - Add entry under `claims` with `claimed_by`, `claimed_at`
   - Add/update entry under `items` with `status: in_progress`, `started_at`
6. Update "Quick View" table
7. Append new session entry under `sessions` with `started_at`, `by`

### During Implementation
- Keep changes scoped to the claimed task
- If blocked, set `status: blocked` with `blocked_reason` and `blocked_at`
- Reference task ID in commit messages (e.g., `M1-E02-T01: Add Skill model`)

### Task Completion

**Before marking done, you MUST:**

1. **Verify ALL subtasks are complete**: Re-read the task's subtask table in `implementation_roadmap.md` and confirm each row's "Verifiable By" criterion is satisfied
2. Run ALL verification commands:
   - `cargo test`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`
3. Update `items.<TASK_ID>`:
   - `status: done`
   - `done_at: <timestamp>`
   - `verification`:
     - `status: passed`
     - `at: <timestamp>`
     - `commands: [list of commands run]`
     - `subtasks: [list of subtask IDs verified, e.g., "S01", "S02", ...]`
     - `evidence: [brief description per subtask or summary]`
4. Remove entry from `claims`
5. Update "Quick View" table
6. Update traceability_matrix.md if implementing new features
7. Commit with message format: `<TASK_ID>: <brief description>`
   - Example: `M1-E01-T01: Initialize Rust project with dependencies`
   - Include task title and key changes in description

**A task is NOT done if any subtask's verification criterion is unmet.**

### End of Session / Handoff
- Update `focus.current_task` and `focus.current_subtask`
- Update session entry with `ended_at`, `worked_on`, and `notes`

---

## Source of Truth

You MUST use these documents as the authoritative plan:
- [PRD.md](specs/PRD.md) - Product requirements, functional specs, user workflows
- [TRD.md](specs/TRD.md) - Technical specs, domain model, security contracts
- [implementation_roadmap.md](specs/implementation_roadmap) - Epic/task breakdown
- [use_cases.md](specs/use_cases.md) - Acceptance criteria
- [traceability_matrix.md](specs/traceability_matrix.md) - Requirements coverage

The `specs/research_archive/` directory is background context only. DO NOT use it as execution requirements.

## Project Context

**Sikil** is a Rust CLI for managing Agent Skills across 5 AI coding agents.
- Tech: Rust 2021, clap 4, serde, rusqlite, anyhow/thiserror
- Targets: macOS (Intel/ARM), Linux (x86_64/aarch64)
- Architecture: `cli/` → `commands/` → `core/` → `utils/`

## Implementation Rules

ALWAYS follow these constraints:

**Architecture**
- Dependencies flow downward only: `cli/` → `commands/` → `core/` → `utils/`
- `thiserror` in `core/`, `anyhow` in `commands/`
- `fs-err` over `std::fs`

**File Locations**
- Commands: `src/commands/<cmd>.rs`
- Domain types: `src/core/{skill,agent}.rs`
- Utilities: `src/utils/{paths,symlink,atomic,git}.rs`

Read [docs/coding-practices.md](docs/coding-practices.md) for detailed patterns.

IF implementing new features:
- Add unit tests in `src/**/tests.rs`
- Add integration tests in `tests/*.rs`

Commit messages SHOULD reference FR/NFR IDs when applicable.
