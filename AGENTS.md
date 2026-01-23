# Sikil Development Guide

Rust CLI for managing Agent Skills across AI coding agents.

## Build & Run

```bash
cargo build                     # Debug build
cargo build --release           # Release build
cargo run -- <args>             # Run CLI (e.g., cargo run -- list)
```

## Validation (Loopback)

Run after every change to get immediate feedback:

```bash
./scripts/verify.sh             # Full suite: tests + clippy + fmt
cargo test                      # All tests
cargo test test_name            # Single test
cargo clippy -- -D warnings     # Lint (warnings = errors)
cargo fmt --check               # Format check
cargo insta review              # Review snapshot changes
```

## Task Workflow

1. Pick one uncompleted task from `IMPLEMENTATION_PLAN.md`
2. Read linked spec in `specs/`
3. Write tests first (TDD), run to confirm red
4. Implement until green
5. Run `./scripts/verify.sh`
6. Update task: `Completed: true`, populate `Tests:`
7. Commit all changes
8. Exit

See `docs/prompts/implement-task.md` for full workflow and rules.

## Architecture

Dependencies flow: `cli/` → `commands/` → `core/` → `utils/`

| Layer | Location | Error Handling |
|-------|----------|----------------|
| CLI | `src/cli/` | Exit codes (0,2,3,4,5) |
| Commands | `src/commands/<cmd>.rs` | `anyhow` |
| Core | `src/core/` | `thiserror` |
| Utils | `src/utils/` | Propagate up |

## Codebase Patterns

- **Filesystem**: `fs-err` (not `std::fs`); atomic symlinks via temp + rename
- **Serde**: `rename_all = "kebab-case"`; `deny_unknown_fields` for config
- **CLI output**: Serializable structs; `--json` flag; `Vec`/`BTreeMap` for determinism
- **Git**: GitHub-only URLs, `--depth=1`, disable hooks
- **Testing**: `assert_cmd` + `predicates`; fixtures in `tests/fixtures/`; `insta` for snapshots

## TDD in Rust

When writing tests first, create minimal stubs (`todo!()`, `unimplemented!()`) so tests compile. First test run should fail on assertion or `todo!()` panic, not missing symbols.

## Source of Truth

`specs/*.md` — Source of truth for behavior/requirements. Workflow docs (`AGENTS.md`, `docs/`) govern process.
