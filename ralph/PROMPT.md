Implement the next eligible task (exactly ONE task this session).

Select the task by reading:
- docs/plan/STATE.yaml (what's done / current focus)
- specs/implementation_roadmap.md (eligibility + subtasks)

Rules:
- Eligible = not done AND all [DEP: ...] are done.
- If no eligible task, respond "No eligible tasks" and STOP.
- If any step fails, STOP and fix before proceeding.
- When finished:
  1. ./scripts/verify.sh (must pass before continuing)
  2. ./scripts/complete-task.sh "<TASK_ID>" "<agent>" "<notes>"
  3. git add -A && git commit (include implementation + STATE.yaml + LOG.md)
- Final response MUST include the commit hash.

