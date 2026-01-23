# Prompt: Implement Task

Pick one task from `IMPLEMENTATION_PLAN.md` and complete it using TDD.

Follow `AGENTS.md` for build commands, validation, and codebase patterns.

## Workflow

1. Read `IMPLEMENTATION_PLAN.md`
2. **If no uncompleted tasks exist:**
   - Review `Notes:` fields of completed tasks for unresolved bugs or issues
   - Run `./scripts/verify.sh` to confirm everything passes
   - If issues found: create a new task for the bug/issue, then continue to step 3
   - If all clean: report "All tasks complete. Verification passed." and exit
3. Pick one uncompleted task
4. Study the linked spec thoroughly (use subagents for related specs)
5. **Write tests first** for the task's Acceptance Criteria
6. Run tests — confirm they fail for the right reason (red), not compile errors
7. Implement until tests pass (green)
8. Refactor if needed
9. Run `./scripts/verify.sh`
10. Confirm Definition of Done (below)
11. Update the task:
    - Set `Completed: true`
    - Populate `Tests:` field with test file(s) and name(s)
    - Add `Notes:` if relevant
12. Commit all changes (tests + code + plan update)
13. Exit

## Definition of Done

A task is complete when:

- [ ] Automated tests written for all automatable Acceptance Criteria
- [ ] `Tests:` field populated with test file(s) and name(s)
- [ ] Manual criteria verified and noted (only if automation not possible, with justification)
- [ ] All tests pass
- [ ] Implementation matches spec behavior
- [ ] `./scripts/verify.sh` passes
- [ ] Task marked `Completed: true`

## Rules

### Do Not Invent Acceptance Criteria

The task's AC are copied verbatim from the spec. Implement exactly what's listed — no more, no less.

### Handle Discoveries

If you discover unspecified behavior:

1. Do NOT add it to the task or tests
2. Note it in the task's `Notes:` field
3. Add the spec to `IMPLEMENTATION_PLAN.md` Spec Issues section:
   ```markdown
   ## Spec Issues
   
   - **[spec.md]**: [What's missing or unclear]
   ```
4. Complete the task with only its original AC
5. Exit — spec issue blocks further tasks for that spec until resolved

### One Task Per Session

Complete one task, then exit. Do not chain multiple tasks.
