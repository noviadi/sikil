Implement the next eligible task (exactly ONE task this session).

## Step 1: Determine what's DONE (read STATE.yaml FIRST)

Read `docs/plan/STATE.yaml` and extract ALL task IDs from the `items:` section where `status: done`. These tasks are COMPLETE and must NOT be picked.

## Step 2: Find eligible task from roadmap

Read `specs/implementation_roadmap.md` to find the FIRST task that:
1. Is NOT in the done list from Step 1
2. Has all `[DEP: ...]` dependencies in the done list from Step 1

## Rules
- If no eligible task exists, respond "No eligible tasks" and STOP.
- If any step fails, STOP and fix before proceeding.
- When finished, run:
  ./scripts/finish-task.sh "<TASK_ID>" "<agent>" "<description>" \
    --subtask "S01:<evidence>" --subtask "S02:<evidence>" ...
- Final response MUST include the commit hash.
