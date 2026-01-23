# Sikil Development Guide

Rust CLI for managing Agent Skills across AI coding agents.

## Build & Run

```bash
cargo build                     # Debug build
cargo build --release           # Optimized release build
cargo run -- <args>             # Run CLI (e.g., cargo run -- list)
./scripts/build.sh              # Release build with optional cross-compilation
```

## Validation

Run after implementing to get immediate feedback:

```bash
./scripts/verify.sh             # Full validation (tests + clippy + fmt)
cargo test                      # Run all tests
cargo test --test e2e_test      # Single integration test file
cargo test test_name            # Specific test
cargo clippy -- -D warnings     # Lint
cargo fmt --check               # Format check
cargo insta review              # Review snapshot changes
```

## Task Completion

Complete one task at a time using:
```bash
./scripts/finish-task.sh "TASK_ID" "agent" "description" --subtask "S01:evidence"
```

This runs verify, updates STATE.yaml/LOG.md, and commits atomically.

## Operational Notes

### Architecture

Dependencies flow: `cli/` → `commands/` → `core/` → `utils/`

| Layer | Location | Error Handling |
|-------|----------|----------------|
| CLI | `src/cli/` | Exit codes (0,2,3,4,5) |
| Commands | `src/commands/<cmd>.rs` | `anyhow` |
| Core | `src/core/` | `thiserror` |
| Utils | `src/utils/` | Propagate up |

### Codebase Patterns

- **Filesystem**: Use `fs-err` (not `std::fs`); atomic symlinks via temp + rename
- **Serde**: `rename_all = "kebab-case"`; `deny_unknown_fields` for config
- **CLI output**: Return serializable structs; `--json` flag; use `Vec`/`BTreeMap` for determinism
- **Git**: GitHub-only URLs, `--depth=1`, disable hooks
- **Testing**: `assert_cmd` + `predicates` for CLI; fixtures in `tests/fixtures/`; `insta` for snapshots

### File Conventions

- Domain types: `src/core/{skill,agent}.rs`
- Commands: `src/commands/<cmd>.rs` with `Args` struct + `execute_*` fn
- Utils: `src/utils/{paths,symlink,atomic,git}.rs`

### Reference Documents

**For agents**

| Document | Purpose |
|----------|---------|
| `specs/*.md` | Source of truth on how the cli works and implemented  |
| `docs/coding-practices.md` | Detailed patterns |

STRICTLY USE specs/*.md only as the source of truth. DO NOT refer to other document unless commanded explicitly.
Whenever required, study the docs using subagents.

