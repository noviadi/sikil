# Agent Execution Guidance

## Execution Workflow

**State file**: [`docs/plan/STATE.yaml`](docs/plan/STATE.yaml)  
**Session log**: [`docs/plan/LOG.md`](docs/plan/LOG.md)  
**Roadmap**: [`specs/implementation_roadmap.md`](specs/implementation_roadmap.md)

### Start of Session
1. Read `docs/PLAN.md` for task selection algorithm
2. Read `docs/plan/STATE.yaml` to find task statuses and claims
3. Select the next eligible task (smallest ID where deps are `done` and not claimed)
4. **Read the task's subtasks** in `specs/implementation_roadmap.md` before starting
5. Claim the task in STATE.yaml:
   ```yaml
   claims:
     "M1-E04-T03": { by: "agent-name", at: "2026-01-18T12:00:00Z", stale_after_hours: 24 }
   ```
6. Update `focus.current_task` in STATE.yaml
7. Update "Quick View" table in PLAN.md

### During Implementation
- Keep changes scoped to the claimed task
- If blocked, set `status: blocked` with `blocked_reason` in STATE.yaml

### Task Completion

**Before marking done, you MUST:**

1. **Verify ALL subtasks**: Re-read the task's subtask table in `implementation_roadmap.md` and confirm each row's "Verifiable By" criterion is satisfied
2. Run ALL verification commands:
   - `cargo test`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`
3. **Create git commit** with verification block (see format below)
4. Update STATE.yaml:
   ```yaml
   items:
     "M1-E04-T03":
       status: done
       done_at: "2026-01-18T13:00:00Z"
       verify: { status: passed, commit: "abc1234" }
   ```
5. Remove entry from `claims`
6. Update "Quick View" table in PLAN.md
7. Append session entry to LOG.md
8. Update traceability_matrix.md if implementing new features

**A task is NOT done if any subtask's verification criterion is unmet.**

### Commit Message Format (REQUIRED)

```
<TASK_ID>: <brief description>

Verification:
- cargo test: <pass/fail, test count>
- cargo clippy -- -D warnings: <clean/warnings>
- cargo fmt --check: <clean/issues>

Subtasks verified:
- S01: <brief evidence>
- S02: <brief evidence>
...
```

Example:
```
M1-E04-T03: Add skill name validation

Verification:
- cargo test: 85 tests passed
- cargo clippy -- -D warnings: clean
- cargo fmt --check: clean

Subtasks verified:
- S01: validate_skill_name() implemented with regex ^[a-z0-9][a-z0-9_-]{0,63}$
- S02: Test with "-skill" fails validation
- S03: Test with "my.skill" fails validation
- S04: Test with 65-char name fails validation
- S05: Test with "my/skill" fails validation
- S06: Test with ".." fails validation
- S07: Test with "" fails validation
```

### Completion Checklist (REQUIRED)

Before ending your session, confirm ALL items:

- [ ] All subtasks verified against "Verifiable By" column
- [ ] `cargo test` passed
- [ ] `cargo clippy -- -D warnings` passed
- [ ] `cargo fmt --check` passed
- [ ] Git commit created with verification block in message body
- [ ] STATE.yaml `items` updated with `verify.commit: <hash>`
- [ ] STATE.yaml `claims` entry removed
- [ ] PLAN.md Quick View updated
- [ ] LOG.md session entry appended

**DO NOT end session without completing ALL checklist items.**

### End of Session / Handoff
- Update `focus.current_task` and `focus.current_subtask` in STATE.yaml
- Append session entry to LOG.md with `started_at`, `ended_at`, `notes`

---

## Source of Truth

You MUST use these documents as the authoritative plan:
- [PRD.md](specs/PRD.md) - Product requirements, functional specs, user workflows
- [TRD.md](specs/TRD.md) - Technical specs, domain model, security contracts
- [implementation_roadmap.md](specs/implementation_roadmap.md) - Epic/task breakdown
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
