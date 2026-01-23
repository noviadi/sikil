# Prompt: Implement Task

Pick one task from `IMPLEMENTATION_PLAN.md` and complete it using TDD.

Follow `AGENTS.md` for build commands, validation, and codebase patterns.

## Workflow

1. Read `IMPLEMENTATION_PLAN.md`, pick one uncompleted task
2. Read the linked spec in `specs/` thoroughly
3. **Write tests first** for the task's Acceptance Criteria
4. Run tests — confirm they fail for the right reason (red), not compile errors
5. Implement until tests pass (green)
6. Refactor if needed
7. Run `./scripts/verify.sh`
8. Confirm Definition of Done (below)
9. Update the task:
   - Set `Completed: true`
   - Populate `Tests:` field with test file(s) and name(s)
   - Add `Notes:` if relevant
10. Commit all changes (tests + code + plan update)
11. Exit

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
