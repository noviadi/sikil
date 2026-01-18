# PLAN (Agent Execution State)

This file is the entrypoint for task execution. State is stored separately for lean context.

**State file**: [`plan/STATE.yaml`](plan/STATE.yaml) — claims, focus, task statuses  
**Session log**: [`plan/LOG.md`](plan/LOG.md) — append-only session history  
**Roadmap**: [`specs/implementation_roadmap.md`](../specs/implementation_roadmap.md) — task definitions

---

## How to Pick the Next Task

1. Read `plan/STATE.yaml` to find task statuses and claims
2. A task is **eligible** if:
   - `status` is `todo` (or not listed), AND
   - all `[DEP: ...]` dependencies are `done`, AND
   - it is not `claimed` by another agent
3. Choose the eligible task with the **smallest lexical ID** (deterministic)
4. If no eligible tasks exist, pick a `blocked` task and resolve the blocker

---

## Quick View

| Status | Tasks |
|--------|-------|
| **Next** | M1-E04-T03 |
| **In progress** | — |
| **Blocked** | — |
| **Recent** | M1-E04-T02, M1-E04-T01, M1-E03-T03, M1-E03-T02, M1-E03-T01 |

---

## Status Definitions

| Status | Meaning |
|--------|---------|
| `todo` | Not started (default, no entry needed) |
| `in_progress` | Claimed and actively being worked on |
| `blocked` | Cannot proceed; set `blocked_reason` in STATE.yaml |
| `done` | Completed with verification |

---

## Verification Requirements

A task is `done` only if STATE.yaml has:
- `status: done`
- `done_at: <timestamp>`
- `verify.status: passed`
- `verify.commit: <hash>` (new format) OR `verify.archive: <file#anchor>` (legacy)
