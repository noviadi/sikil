# Agent Execution Guidance

## IMPORTANT: One Task at a Time

**Execute exactly ONE task per session.** Do not batch multiple tasks.

---

## Quick Reference

| File | Purpose |
|------|---------|
| `docs/plan/STATE.yaml` | Task statuses, current focus |
| `docs/plan/LOG.md` | Session history (append-only) |
| `specs/implementation_roadmap.md` | Task subtasks and dependencies |

---

## Workflow

### 1. Pick Task
1. Read `docs/plan/STATE.yaml` → check which tasks are `done`
2. Read `specs/implementation_roadmap.md` → find **ONE** smallest eligible task ID
   - Eligible = not done + all `[DEP: ...]` are done
3. Update STATE.yaml: `focus: { current_task: "<ID>", by: "<agent>" }`

**STOP here if no eligible task. Do NOT pick multiple tasks.**

### 2. Implement
1. Read task's subtask table in roadmap
2. Implement each subtask
3. Verify against "Verifiable By" column

### 3. Complete (ALL steps required)

```
□ Run: cargo test
□ Run: cargo clippy -- -D warnings  
□ Run: cargo fmt --check
□ Update STATE.yaml:
    items:
      "<TASK_ID>": { status: done, at: "<timestamp>" }
    focus: { current_task: null, by: null }
□ Update PLAN.md Quick View table
□ Append entry to LOG.md
□ Git commit (include STATE.yaml, LOG.md, PLAN.md in same commit)
```

### Commit Message Format

```
<TASK_ID>: <description>

Verification:
- cargo test: <result>
- cargo clippy -- -D warnings: <result>
- cargo fmt --check: <result>

Subtasks:
- S01: <evidence>
- S02: <evidence>
...
```

---

## Source of Truth

- [PRD.md](specs/PRD.md) - Product requirements
- [TRD.md](specs/TRD.md) - Technical specs
- [implementation_roadmap.md](specs/implementation_roadmap.md) - Task breakdown
- [use_cases.md](specs/use_cases.md) - Acceptance criteria
- [traceability_matrix.md](specs/traceability_matrix.md) - Requirements coverage

The `specs/research_archive/` directory is background only. DO NOT use as requirements.

---

## Project Context

**Sikil** is a Rust CLI for managing Agent Skills across 5 AI coding agents.

- Tech: Rust 2021, clap 4, serde, rusqlite, anyhow/thiserror
- Targets: macOS (Intel/ARM), Linux (x86_64/aarch64)
- Architecture: `cli/` → `commands/` → `core/` → `utils/`

## Implementation Rules

**Architecture**
- Dependencies flow downward: `cli/` → `commands/` → `core/` → `utils/`
- `thiserror` in `core/`, `anyhow` in `commands/`
- `fs-err` over `std::fs`

**File Locations**
- Commands: `src/commands/<cmd>.rs`
- Domain types: `src/core/{skill,agent}.rs`
- Utilities: `src/utils/{paths,symlink,atomic,git}.rs`

**Testing**
- Unit tests in `src/**/tests.rs`
- Integration tests in `tests/*.rs`

Read [docs/coding-practices.md](docs/coding-practices.md) for detailed patterns.
