# Task Implementation Prompts

## Standard Prompt

```
Implement the next eligible task.

Read first:
1. AGENTS.md - workflow
2. docs/plan/STATE.yaml - current progress
3. specs/implementation_roadmap.md - task subtasks

Complete ALL items in the "Complete" checklist before ending.
```

## Task-Specific Prompt

```
Implement task <TASK_ID>.

Read first:
1. AGENTS.md - workflow
2. docs/plan/STATE.yaml - current progress  
3. specs/implementation_roadmap.md - task subtasks

Complete ALL items in the "Complete" checklist before ending.
```

## Code Review Prompt

```
Review the latest completed task.

Verify:
1. All subtasks match "Verifiable By" in roadmap
2. cargo test/clippy/fmt passed
3. Git commit has verification block
4. STATE.yaml updated
5. LOG.md has entry

Report any issues.
```

## File Locations

| File | Path |
|------|------|
| Workflow | `AGENTS.md` |
| State | `docs/plan/STATE.yaml` |
| Log | `docs/plan/LOG.md` |
| Tasks | `specs/implementation_roadmap.md` |
| Quick View | `docs/PLAN.md` |
