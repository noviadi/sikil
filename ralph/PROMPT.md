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
