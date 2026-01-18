# Task Implementation Prompt Template

Use these prompts when starting a new task implementation session.

---

## Standard Prompt

```
Implement the next eligible task.

Read these files first:
1. AGENTS.md (project root) - workflow instructions
2. docs/PLAN.md - task selection algorithm
3. docs/plan/STATE.yaml - current progress and claims
4. specs/implementation_roadmap.md - task subtasks

Complete ALL items in the Completion Checklist before ending.
```

---

## Task-Specific Prompt

```
Implement task <TASK_ID>.

Read these files first:
1. AGENTS.md (project root) - workflow instructions
2. docs/PLAN.md - task selection algorithm
3. docs/plan/STATE.yaml - current progress and claims
4. specs/implementation_roadmap.md - task subtasks

Complete ALL items in the Completion Checklist before ending.
```

---

## Code Review Prompt

```
Review the latest completed task for workflow adherence and implementation completeness.

Check these files:
1. AGENTS.md (project root) - workflow checklist
2. docs/plan/STATE.yaml - task verification entry
3. specs/implementation_roadmap.md - subtask requirements
4. git log - commit message format

Verify:
1. All subtasks match "Verifiable By" criteria in roadmap
2. All verification commands were run (cargo test, clippy, fmt)
3. Git commit has verification block with subtasks evidence
4. STATE.yaml updated with commit hash reference
5. LOG.md has session entry appended

Report any missing items or incomplete subtasks.
```

---

## File Locations

| File | Path | Purpose |
|------|------|---------|
| Workflow & Checklist | `AGENTS.md` | Agent execution guidance |
| Task Selection | `docs/PLAN.md` | Algorithm + Quick View |
| Execution State | `docs/plan/STATE.yaml` | Claims, focus, task statuses |
| Session Log | `docs/plan/LOG.md` | Append-only history |
| Task Details | `specs/implementation_roadmap.md` | Subtask definitions |
| Legacy Evidence | `docs/plan/ARCHIVE_M1.md` | Pre-migration task evidence |
