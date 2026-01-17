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
| **Next candidates** | M1-E01-T02, M1-E01-T03 |
| **In progress** | — |
| **Blocked** | — |
| **Recently completed** | M1-E01-T01 |

---

## STATE

> Machine-readable YAML block. Only list items that differ from default (default = todo/unclaimed/unverified).

```yaml
schema_version: 1
updated_at: "2026-01-17T00:00:00Z"

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

# Session log (append-only)
sessions:
  - started_at: "2026-01-17T09:10:00Z"
    ended_at: "2026-01-17T09:30:00Z"
    by: "amp-agent"
    worked_on:
      - id: "M1-E01-T01"
        outcome: "done"
    notes: "Completed project init with all deps. Fixed missing tempfile, once_cell, regex, sha2."
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
