# Task Implementation Prompts

## Standard Prompt

```
Implement the next eligible task (exactly ONE task this session).

Select the task by reading:
- docs/plan/STATE.yaml (what's done / current focus)
- specs/implementation_roadmap.md (eligibility + subtasks)

Rules:
- Eligible = not done AND all [DEP: ...] are done.
- If no eligible task, respond "No eligible tasks" and STOP.
- If any step fails, STOP and fix before proceeding.
- When finished, run:
  ./scripts/finish-task.sh "<TASK_ID>" "<agent>" "<description>" \
    --subtask "S01:<evidence>" --subtask "S02:<evidence>" ...
- Final response MUST include the commit hash.
```

## Task-Specific Prompt

```
Implement task <TASK_ID> (exactly ONE task this session).

Read:
- docs/plan/STATE.yaml (confirm status/focus)
- specs/implementation_roadmap.md (subtasks + "Verifiable By")

Rules:
- If any step fails, STOP and fix before proceeding.

Finish by running:
./scripts/finish-task.sh "<TASK_ID>" "<agent>" "<description>" \
  --subtask "S01:<evidence>" --subtask "S02:<evidence>" ...

Final response MUST include the commit hash.
```

## Code Review Prompt

```
Review the latest completed task.

Verify:
1. All subtasks match "Verifiable By" in roadmap
2. Commit message has correct format (TASK_ID: description + Verification + Subtasks)
3. STATE.yaml + LOG.md updated

Report any issues.
```

## File Locations

| File | Path |
|------|------|
| Workflow | `AGENTS.md` |
| State | `docs/plan/STATE.yaml` |
| Log | `docs/plan/LOG.md` |
| Tasks | `specs/implementation_roadmap.md` |
| Finish | `scripts/finish-task.sh` |
| Verify | `scripts/verify.sh` |
| Complete | `scripts/complete-task.sh` |
| Validate | `scripts/validate-commit-msg.sh` |
