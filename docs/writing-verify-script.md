# Writing the Verification Script

This document defines how to create `./scripts/verify.sh`, the standardized entry point for project validation.

## Overview

The verification script is a **prerequisite** for implementation planning. It provides a single command that runs all quality checks (tests, linting, formatting, type checking) regardless of the underlying toolchain.

```
./scripts/verify.sh
    ↓
[Project-specific tools: tests, linter, formatter, type checker]
    ↓
Exit 0 (success) or non-zero (failure)
```

## Why a Wrapper Script?

| Benefit | Description |
|---------|-------------|
| **Consistency** | Same command across all projects |
| **Discoverability** | Agents and humans know where to look |
| **Composability** | Can run multiple tools in sequence |
| **CI alignment** | Script can be reused in CI pipelines |

## Script Requirements

### Location

Always: `./scripts/verify.sh`

### Behavior

1. Run all quality checks in sequence
2. Exit with code 0 if all pass
3. Exit with non-zero code on first failure
4. Print clear output indicating what's running

### Checks to Include

| Check Type | Purpose | Examples |
|------------|---------|----------|
| **Tests** | Verify behavior | `cargo test`, `npm test`, `go test`, `pytest` |
| **Linting** | Catch bugs/style issues | `cargo clippy`, `eslint`, `golangci-lint`, `flake8` |
| **Formatting** | Ensure consistent style | `cargo fmt --check`, `prettier --check`, `gofmt`, `black --check` |
| **Type checking** | Catch type errors | `tsc --noEmit`, `mypy`, `pyright` |

Not all projects need all checks. Include what's relevant.

### Script Constraints

The script must be:

| Constraint | Reason |
|------------|--------|
| **Non-interactive** | No prompts, confirmations, or user input; must run unattended |
| **Fail-fast** | Exit on first failure (`set -e`); don't continue after errors |
| **CI-safe** | Must work in CI environments without local dependencies |
| **Deterministic** | Same code state → same result; no flaky tests |
| **No network dependency** | Unless explicitly documented; offline-first |

**Recommended additions:**
- Print tool versions at start (helps debug CI vs local differences)
- Use `set -euo pipefail` for strict error handling
- Treat warnings as errors where applicable (e.g., `cargo clippy -- -D warnings`)

## Examples by Project Type

### Rust

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "==> Running tests..."
cargo test

echo "==> Running clippy..."
cargo clippy -- -D warnings

echo "==> Checking formatting..."
cargo fmt --check

echo "==> All checks passed!"
```

### Node.js / TypeScript

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "==> Type checking..."
npm run typecheck  # or: npx tsc --noEmit

echo "==> Running tests..."
npm test

echo "==> Linting..."
npm run lint  # or: npx eslint .

echo "==> Checking formatting..."
npx prettier --check .

echo "==> All checks passed!"
```

### Go

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "==> Running tests..."
go test ./...

echo "==> Running linter..."
golangci-lint run

echo "==> Checking formatting..."
test -z "$(gofmt -l .)" || { echo "Files need formatting"; exit 1; }

echo "==> All checks passed!"
```

### Python

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "==> Running tests..."
pytest

echo "==> Type checking..."
mypy .

echo "==> Linting..."
flake8

echo "==> Checking formatting..."
black --check .

echo "==> All checks passed!"
```

### Multi-language / Monorepo

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "==> Backend (Rust)..."
(cd backend && cargo test && cargo clippy -- -D warnings)

echo "==> Frontend (TypeScript)..."
(cd frontend && npm test && npm run lint)

echo "==> All checks passed!"
```

## Script Template

```bash
#!/usr/bin/env bash
set -euo pipefail

# Verify script for [PROJECT_NAME]
# Runs all quality checks required before committing code.

echo "==> Running tests..."
# TODO: Add test command

echo "==> Running linter..."
# TODO: Add lint command

echo "==> Checking formatting..."
# TODO: Add format check command

echo "==> All checks passed!"
```

## Creating the Script

1. Create `scripts/` directory if it doesn't exist
2. Create `verify.sh` using the template above
3. Fill in project-specific commands
4. Make executable: `chmod +x scripts/verify.sh`
5. Test it: `./scripts/verify.sh`
6. Commit the script

## Prerequisite for Plan Writing

Before running gap analysis or writing `IMPLEMENTATION_PLAN.md`:

1. Check if `./scripts/verify.sh` exists
2. If missing, **stop and notify**:
   ```
   ⚠️ Verification script not found.
   
   Before creating an implementation plan, create ./scripts/verify.sh
   See docs/writing-verify-script.md for instructions.
   ```
3. Once the script exists and passes, proceed with plan writing

## Checklist

Before using the verification script:

**Setup:**
- [ ] Script exists at `./scripts/verify.sh`
- [ ] Script is executable (`chmod +x`)
- [ ] Script uses `set -euo pipefail`

**Checks included:**
- [ ] Runs tests
- [ ] Runs linter (if project has one)
- [ ] Checks formatting (if project has formatter)
- [ ] Runs type checker (if applicable)

**Constraints met:**
- [ ] Non-interactive (no prompts or user input)
- [ ] Fails fast on first error
- [ ] CI-safe (no local-only dependencies)
- [ ] Deterministic (no flaky behavior)
- [ ] Exits 0 on success, non-zero on failure
