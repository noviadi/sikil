# Task Implementation Prompts

## Standard Prompt

```
Implement the next eligible task (exactly ONE task this session).

Select the task by reading:
- docs/plan/STATE.yaml (what's done / current focus)
- specs/implementation_roadmap.md (eligibility + subtasks)

Rules:
- Eligible = not done AND all [DEP: ...] are done; STOP if none eligible.
- When finished: run ./scripts/verify.sh, then ./scripts/complete-task.sh "<TASK_ID>" "<agent>" "<notes>".
- If anything is unclear, follow AGENTS.md (already loaded in context).
```

## Task-Specific Prompt

```
Implement task <TASK_ID> (exactly ONE task this session).

Read:
- docs/plan/STATE.yaml (confirm status/focus)
- specs/implementation_roadmap.md (subtasks + "Verifiable By")

Finish by running:
- ./scripts/verify.sh
- ./scripts/complete-task.sh "<TASK_ID>" "<agent>" "<notes>"

If workflow details are unclear, follow AGENTS.md (already loaded in context).
```

## Code Review Prompt

```
Review the latest completed task.

Verify:
1. All subtasks match "Verifiable By" in roadmap
2. ./scripts/verify.sh passed (test + clippy + fmt)
3. Git commit has verification block
4. STATE.yaml + LOG.md updated (via ./scripts/complete-task.sh)

Report any issues.
```

## File Locations

| File | Path |
|------|------|
| Workflow | `AGENTS.md` |
| State | `docs/plan/STATE.yaml` |
| Log | `docs/plan/LOG.md` |
| Tasks | `specs/implementation_roadmap.md` |
| Verify | `scripts/verify.sh` |
| Complete | `scripts/complete-task.sh` |
