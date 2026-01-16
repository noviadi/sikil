# Sikil - Traceability Matrix

## A. Overview

This document provides traceability from PRD requirements to Use Cases and Acceptance Criteria. Task-level traceability is embedded directly in [implementation_roadmap.md](implementation_roadmap.md) via `**Traces:**` lines.

### Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully covered |
| ⚠️ | Partially covered / deferred |
| 🔧 | Covered by design constraint |

### Related Documents

| Document | Purpose |
|----------|---------|
| [PRD.md](PRD.md) | Product requirements and scope |
| [use_cases.md](use_cases.md) | Use cases and acceptance criteria |
| [implementation_roadmap.md](implementation_roadmap.md) | Tasks with embedded AC/NFR traces |
| [TRD.md](TRD.md) | Technical specifications and architecture |

### Traceability Approach

- **PRD → UC/AC**: Documented in this file (Section C)
- **AC/NFR → Tasks**: Embedded in `implementation_roadmap.md` via `**Traces:** I: <ACs> | V: <NFRs>`
- **Link Types**:
  - `I:` = Implements (delivers the AC behavior)
  - `V:` = Validates (verifies NFR compliance)

---

## B. Coverage Dashboard

| Domain | Total | Covered | Partial | Gap | Coverage |
|--------|-------|---------|---------|-----|----------|
| Functional Requirements (FR) | 57 | 57 | 0 | 0 | 100% ✅ |
| Non-Functional Requirements (NFR) | 26 | 23 | 0 | 0 | 100% ✅ (3 by design) |
| Acceptance Criteria (AC) | 130 | 129 | 1 | 0 | 99% ✅ |

**Notes:**
- 1 partial: UC-04-03 (SSH URL support) deferred to v1.1 for security hardening
- 3 NFRs covered by design: NFR-04 (binary size), NFR-19 (user success metric), NFR-20 (no script execution)

### Task Coverage by Type

| Task Type | Count | Has Traces | Notes |
|-----------|-------|------------|-------|
| Implementation | 49 | ✅ Yes | Linked to ACs/NFRs |
| Test `[TEST]` | 19 | — | Verify implementations |
| Wire `[WIRE]` | 10 | — | Connect logic to CLI |
| Doc `[DOC]` | 4 | — | Documentation tasks |
| **Total** | **82** | | |

---

## C. PRD Functional Requirements → Use Cases / Acceptance Criteria

| PRD Req | Requirement | UC | AC IDs |
|---------|-------------|-----|--------|
| FR-01.1 | Scan all 5 agent global paths | UC-01 | UC-01-01 |
| FR-01.2 | Scan workspace paths relative to CWD | UC-01 | UC-01-02 |
| FR-01.3 | Parse SKILL.md YAML frontmatter | UC-01, UC-02 | UC-01-03, UC-02-01 |
| FR-01.4 | Classify skills as managed/unmanaged | UC-01 | UC-01-04, UC-01-05 |
| FR-01.5 | Detect broken symlinks | UC-01 | UC-01-10 |
| FR-01.6 | Filter by agent, managed status | UC-01 | UC-01-06, UC-01-07, UC-01-08 |
| FR-01.7 | Display summary with counts | UC-01 | UC-01-12 |
| FR-02.1 | Display metadata (name, description, version, author) | UC-02 | UC-02-01 |
| FR-02.2 | Show all installation locations | UC-02 | UC-02-02, UC-02-05 |
| FR-02.3 | Display file tree within skill | UC-02 | UC-02-06 |
| FR-02.4 | Show total size on disk | UC-02 | UC-02-07 |
| FR-03.1 | Install from local directory | UC-03 | UC-03-01, UC-03-02 |
| FR-03.2 | Install from Git URL (GitHub, HTTPS, SSH) | UC-04 | UC-04-01, UC-04-02, UC-04-03 |
| FR-03.3 | Support subdirectory extraction from Git repos | UC-04 | UC-04-04, UC-04-06 |
| FR-03.4 | Copy to `~/.sikil/repo/` and create symlinks | UC-03, UC-04 | UC-03-02, UC-03-05, UC-04-08, UC-04-09 |
| FR-03.5 | Validate skill before installation | UC-03, UC-04 | UC-03-01, UC-03-10, UC-04-07 |
| FR-03.6 | Select target agents with `--to` flag | UC-03, UC-04 | UC-03-03, UC-03-04 |
| FR-03.7 | Create agent directories if missing | UC-03 | UC-03-11 |
| FR-03.8 | Fail gracefully if skill exists (no overwrite) | UC-03, UC-04 | UC-03-06, UC-03-07, UC-03-08 |
| FR-04.1 | Move unmanaged skill to managed repo | UC-05 | UC-05-04 |
| FR-04.2 | Replace original with symlink | UC-05 | UC-05-05 |
| FR-04.3 | Require `--from` if multiple locations | UC-05 | UC-05-02, UC-05-03 |
| FR-04.4 | Atomic operation with rollback | UC-05 | UC-05-09 |
| FR-05.1 | Copy from repo back to agent locations | UC-06 | UC-06-04 |
| FR-05.2 | Remove symlinks, create physical copies | UC-06 | UC-06-04 |
| FR-05.3 | Support per-agent or all-agents unmanage | UC-06 | UC-06-02, UC-06-03 |
| FR-05.4 | Require confirmation (bypass with `--yes`) | UC-06 | UC-06-08, UC-06-09 |
| FR-06.1 | Remove from specific agent(s) | UC-07 | UC-07-02, UC-07-03 |
| FR-06.2 | Remove from all agents + repo with `--all` | UC-07 | UC-07-04 |
| FR-06.3 | Support managed and unmanaged skills | UC-07 | UC-07-05, UC-07-06 |
| FR-06.4 | Require confirmation (bypass with `--yes`) | UC-07 | UC-07-07, UC-07-08 |
| FR-07.1 | Sync skill to agents that don't have it | UC-08 | UC-08-02, UC-08-03 |
| FR-07.2 | Sync all managed skills with `--all` | UC-08 | UC-08-06 |
| FR-07.3 | Limit sync to specific agents with `--to` | UC-08 | UC-08-07 |
| FR-07.4 | Fail if unmanaged skill blocks (suggest adopt) | UC-08 | UC-08-05 |
| FR-08.1 | Validate SKILL.md exists | UC-09 | UC-09-02 |
| FR-08.2 | Validate YAML frontmatter syntax | UC-09 | UC-09-03 |
| FR-08.3 | Validate required fields (name, description) | UC-09 | UC-09-04, UC-09-05 |
| FR-08.4 | Validate name format constraints | UC-09 | UC-09-06, UC-09-07 |
| FR-08.5 | Warn on missing optional fields | UC-09 | UC-09-08 |
| FR-08.6 | Exit code 0 on pass, non-zero on fail | UC-09 | UC-09-11 |
| FR-09.1 | Detect duplicate-unmanaged | UC-10 | UC-10-01 |
| FR-09.2 | Detect duplicate-managed | UC-10 | UC-10-02 |
| FR-09.3 | Report conflicts with recommendations | UC-10 | UC-10-05, UC-10-06 |
| FR-09.4 | Filter with `--conflicts` and `--duplicates` | UC-10 | UC-10-03, UC-10-04 |
| FR-10.1 | Config file at `~/.sikil/config.toml` | UC-11 | UC-11-01 |
| FR-10.2 | Override agent global/workspace paths | UC-11 | UC-11-05, UC-11-06 |
| FR-10.3 | Enable/disable specific agents | UC-11 | UC-11-07 |
| FR-10.4 | Display current config | UC-11 | UC-11-02 |
| FR-10.5 | Edit config with `--edit` | UC-11 | UC-11-03 |
| FR-10.6 | Set individual values with `config set` | UC-11 | UC-11-04 |
| FR-11.1 | Human-readable colored output (default) | UC-12 | (implicit) |
| FR-11.2 | JSON output with `--json` flag | UC-12 | UC-12-01, UC-12-02 |
| FR-11.3 | Respect `NO_COLOR` environment variable | UC-12 | (via NFR-16) |
| FR-12.1 | Generate bash completions | UC-13 | UC-13-01 |
| FR-12.2 | Generate zsh completions | UC-13 | UC-13-01 |
| FR-12.3 | Generate fish completions | UC-13 | UC-13-01 |

---

## D. NFR Coverage Summary

| NFR ID | Category | Requirement | Coverage | Notes |
|--------|----------|-------------|----------|-------|
| NFR-01 | Performance | `sikil list` < 500ms | ✅ M5-E04-T03 | Benchmark test |
| NFR-02 | Performance | Cached scan < 100ms | ✅ M1-E07-T01, M2-E01-T05 | Cache system |
| NFR-03 | Performance | Cache invalidation by mtime | ✅ M1-E07-T01 | Cache logic |
| NFR-04 | Performance | Binary size < 10MB | 🔧 Build config | Release profile |
| NFR-05 | Reliability | Destructive ops require confirmation | ✅ M3-E04-T02, M3-E05-T02 | Confirmation prompts |
| NFR-06 | Reliability | Failed ops restore original state | ✅ M1-E05-T03, M3-E01-T02 | Atomic operations |
| NFR-07 | Reliability | Graceful permission error handling | ✅ M1-E02-T03 | Error types |
| NFR-08 | Reliability | Graceful missing directory handling | ✅ M1-E02-T03 | Error types |
| NFR-09 | Reliability | Broken symlink detection | ✅ M1-E05-T02 | Symlink utilities |
| NFR-10 | Compatibility | macOS Intel support | ✅ M5-E03-T04 | Cross-platform build |
| NFR-11 | Compatibility | macOS Apple Silicon support | ✅ M5-E03-T04 | Cross-platform build |
| NFR-12 | Compatibility | Linux x86_64 support | ✅ M5-E03-T04 | Cross-platform build |
| NFR-13 | Compatibility | Linux aarch64 support | ✅ M5-E03-T04 | Cross-platform build |
| NFR-14 | Compatibility | Windows explicitly unsupported | 🔧 Design | Out of scope |
| NFR-15 | Compatibility | Minimum Git version 2.0+ | ✅ M3-E02-T02 | Git clone logic |
| NFR-16 | Usability | Colored terminal output | ✅ M1-E06-T02 | Output formatting |
| NFR-17 | Usability | Clear error messages | ✅ M1-E02-T03, M2-E04-T02, M5-E04-T02 | Error handling |
| NFR-18 | Usability | Progress indicators | ✅ M1-E06-T02, M3-E01-T01, M3-E02-T02 | Progress helpers |
| NFR-19 | Usability | First-time user success < 5 min | 🔧 M5-E02 | Documentation |
| NFR-20 | Security | No execution of skill scripts | 🔧 Design | Read-only constraint |
| NFR-21 | Security | Path traversal prevention | ✅ M1-E04-T03 | Name validation |
| NFR-22 | Security | Git clone over HTTPS only | ✅ M3-E02-T01 | URL parsing |
| NFR-23 | Security | Symlink rejection in skills | ✅ M1-E05-T03 | Copy validation |
| NFR-24 | Security | GitHub-only URL validation | ✅ M3-E02-T01 | URL parsing |
| NFR-25 | Security | Config file hardening | ✅ M1-E03-T02 | Size limit, deny unknown |
| NFR-26 | Security | SQLite WAL mode | ✅ M1-E07-T01 | Cache implementation |

---

## E. Open Gaps & Deferrals

| ID | Source | Requirement | Status | Target | Notes |
|----|--------|-------------|--------|--------|-------|
| GAP-001 | UC-04-03 | SSH URL support | ⚠️ Deferred | v1.1 | Security hardening needed |

---

## F. How to Verify Task Coverage

To find which tasks implement a specific AC, search `implementation_roadmap.md`:

```bash
# Find tasks implementing UC-03-01
grep -n "UC-03-01" implementation_roadmap.md
```

To list all tasks with traces:
```bash
grep -n "^\*\*Traces:\*\*" implementation_roadmap.md
```

---

**Document Version**: 5.0  
**Created**: January 16, 2026  
**Updated**: January 16, 2026  
**Status**: ✅ Complete - Simplified (task traces moved to roadmap)
