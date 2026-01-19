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

### 3. Complete (ONE command)

```
□ Run: ./scripts/finish-task.sh "<TASK_ID>" "<agent>" "<description>" \
    --subtask "S01:<evidence>" --subtask "S02:<evidence>" ...
```

This single command:
1. Runs `verify.sh` (fails fast if tests/clippy/fmt fail)
2. Updates STATE.yaml + LOG.md via `complete-task.sh`
3. Creates properly formatted commit with verification results

**Options:**
- `--notes "<notes>"` — Custom notes for LOG.md (defaults to description)
- `--force` — Skip focus mismatch check
- `--no-verify` — Skip verification (use with caution)

**Example:**
```bash
./scripts/finish-task.sh "M2-E01-T05" "amp" "implement cache integration" \
  --subtask "S01:Added CacheService struct" \
  --subtask "S02:Unit tests in cache_test.rs"
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
