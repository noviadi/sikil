Implement the next eligible task (exactly ONE task this session).

Select the task by reading:
- docs/plan/STATE.yaml (what's done / current focus)
- specs/implementation_roadmap.md (eligibility + subtasks)

Rules:
- Eligible = not done AND all [DEP: ...] are done; STOP if none eligible.
- When finished: run ./scripts/verify.sh, then ./scripts/complete-task.sh "<TASK_ID>" "<agent>" "<notes>".
- If anything is unclear, follow AGENTS.md (already loaded in context).

