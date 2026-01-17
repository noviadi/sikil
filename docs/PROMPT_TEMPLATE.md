# Task Implementation Prompt Template

Use these prompts when starting a new task implementation session.

---

## Standard Prompt

```
Implement the next eligible task.

Read these files first:
1. AGENTS.md (project root) - workflow instructions
2. docs/PLAN.md - current progress and next task
3. specs/implementation_roadmap.md - task subtasks

Complete ALL items in the Completion Checklist before ending.
```

---

## Task-Specific Prompt

```
Implement task <TASK_ID>.

Read these files first:
1. AGENTS.md (project root) - workflow instructions
2. docs/PLAN.md - current progress
3. specs/implementation_roadmap.md - task subtasks

Complete ALL items in the Completion Checklist before ending.
```

---

## Code Review Prompt

```
Review the latest completed task for workflow adherence and implementation completeness.

Check these files:
1. AGENTS.md (project root) - workflow checklist
2. docs/PLAN.md - task verification block
3. specs/implementation_roadmap.md - subtask requirements
4. git log - commit message format

Verify:
1. All subtasks match "Verifiable By" criteria in roadmap
2. All verification commands were run (cargo test, clippy, fmt)
3. PLAN.md has complete verification block with subtasks and evidence
4. Git commit exists with format: <TASK_ID>: <description>

Report any missing items or incomplete subtasks.
```

---

## File Locations

| File | Path |
|------|------|
| Workflow & Checklist | `AGENTS.md` (project root, NOT in docs/) |
| Task Progress | `docs/PLAN.md` |
| Task Details | `specs/implementation_roadmap.md` |
