# PLAN

**State**: [`plan/STATE.yaml`](plan/STATE.yaml)  
**Log**: [`plan/LOG.md`](plan/LOG.md)  
**Roadmap**: [`../specs/implementation_roadmap.md`](../specs/implementation_roadmap.md)

---

## Task Selection

1. A task is **eligible** if:
   - Not in STATE.yaml as `done`
   - All `[DEP: ...]` dependencies are `done`
2. Pick the **smallest eligible task ID**

---

## Quick View

| Status | Tasks |
|--------|-------|
| **Next** | M1-E05-T02 |
| **In progress** | — |
| **Blocked** | — |
| **Recent** | M1-E05-T01, M1-E04-T04, M1-E04-T03 |
