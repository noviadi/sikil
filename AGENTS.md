# Agent Execution Guidance

## Source of Truth

You MUST use these documents as the authoritative plan:
- [PRD.md](specs/PRD.md) - Product requirements, functional specs, user workflows
- [TRD.md](specs/TRD.md) - Technical specs, domain model, security contracts
- [implementation_roadmap.md](specs/implementation_roadmap) - Epic/task breakdown
- [use_cases.md](specs/use_cases.md) - Acceptance criteria
- [traceability_matrix.md](specs/traceability_matrix.md) - Requirements coverage

The `specs/research_archive/` directory is background context only. DO NOT use it as execution requirements.

## Project Context

**Sikil** is a Rust CLI for managing Agent Skills across 5 AI coding agents.
- Tech: Rust 2021, clap 4, serde, rusqlite, anyhow/thiserror
- Targets: macOS (Intel/ARM), Linux (x86_64/aarch64)
- Architecture: `cli/` → `commands/` → `core/` → `utils/`

## Before Implementation

You MUST gather context before writing code:
1. Check `src/` for existing modules and patterns
2. Read relevant TRD sections: Domain Model, Command Specifications, Security
3. Trace requirements in traceability_matrix.md to locate related code
4. Run `cargo check` to verify current compilation state

## Implementation Rules

ALWAYS follow these constraints:

**Architecture**
- Dependencies flow downward only: `cli/` → `commands/` → `core/` → `utils/`
- `thiserror` in `core/`, `anyhow` in `commands/`
- `fs-err` over `std::fs`

**File Locations**
- Commands: `src/commands/<cmd>.rs`
- Domain types: `src/core/{skill,agent}.rs`
- Utilities: `src/utils/{paths,symlink,atomic,git}.rs`

Read [docs/coding-practices.md](docs/coding-practices.md) for detailed patterns.

## Completing the Job

You MUST pass ALL checks before considering work complete:
1. `cargo test` - MUST pass
2. `cargo clippy -- -D warnings` - MUST have zero warnings
3. `cargo fmt` - MUST be formatted
4. Manual verification of the specific command flow

IF implementing new features:
- Add unit tests in `src/**/tests.rs`
- Add integration tests in `tests/*.rs`
- Update traceability_matrix.md

Commit messages SHOULD reference FR/NFR IDs when applicable.
