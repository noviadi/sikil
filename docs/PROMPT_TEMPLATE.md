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

## File Locations

| File | Path |
|------|------|
| Workflow & Checklist | `AGENTS.md` (project root, NOT in docs/) |
| Task Progress | `docs/PLAN.md` |
| Task Details | `specs/implementation_roadmap.md` |
