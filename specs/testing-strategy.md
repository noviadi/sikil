# Testing Strategy

Testing strategy defines how Sikil validates behavior through automated tests.

## Overview

Sikil uses a multi-layered testing approach combining unit tests, integration tests, and snapshot testing. Tests are organized to match the codebase structure with unit tests inline in source files and integration tests in a dedicated `tests/` directory.

## Test Locations

| Layer | Location | Purpose |
|-------|----------|---------|
| Unit Tests | `src/core/*.rs` (`#[cfg(test)]` modules) | Test individual functions and types |
| Integration Tests | `tests/*_test.rs` | Test command behavior and multi-component flows |
| E2E Tests | `tests/e2e_test.rs` | Test complete user workflows |
| Test Helpers | `tests/common/mod.rs` | Shared utilities for integration tests |
| Fixtures | `tests/fixtures/` | Static test data files |
| Snapshots | `src/core/snapshots/`, `tests/snapshots/` | Insta snapshot files |

## Unit Tests

Unit tests are placed inline within source files using `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_classify_installation_unmanaged() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("my-skill");
        fs::create_dir(&skill_dir).unwrap();
        
        let result = classify_installation(&skill_dir);
        assert_eq!(result, InstallationType::Unmanaged);
    }
}
```

Key patterns:
- Use `tempfile::TempDir` for isolated filesystem tests
- Use `fs_err` (not `std::fs`) for better error messages
- Prefer testable functions with explicit parameters over relying on environment

## Integration Tests

Integration tests live in `tests/` and follow naming convention `*_test.rs`:

| File | Tests |
|------|-------|
| `cli_framework_test.rs` | CLI argument parsing and help output |
| `config_command_test.rs` | Configuration management |
| `conflict_detection_test.rs` | Conflict identification |
| `e2e_test.rs` | Full user workflow scenarios |
| `error_handling_test.rs` | Error message formatting |
| `filesystem_utils_test.rs` | Path and symlink utilities |
| `install_command_test.rs` | Skill installation |
| `list_command_test.rs` | Skill listing with filters |
| `parser_integration_test.rs` | SKILL.md parsing with fixtures |
| `performance_test.rs` | Performance benchmarks |
| `show_command_test.rs` | Skill detail display |
| `unmanage_command_test.rs` | Converting managed to unmanaged |
| `validate_command_test.rs` | SKILL.md validation |

### CLI Testing Pattern

Use `assert_cmd` with the `sikil_cmd!` macro for consistent binary invocation:

```rust
mod common;

use predicates::str::contains;
use tempfile::TempDir;

#[test]
fn test_list_with_no_skills() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    let mut cmd = sikil_cmd!();
    cmd.env("HOME", temp_dir.path())
        .arg("list")
        .arg("--no-cache")
        .assert()
        .success()
        .stdout(contains("No skills found"));
}
```

The `sikil_cmd!` macro (defined in `tests/common/mod.rs`) uses `cargo_bin_cmd!` for compatibility with custom build directories.

## Snapshot Testing with Insta

Sikil uses [insta](https://insta.rs/) for snapshot testing to verify JSON serialization and complex output.

### Snapshot Locations

- `src/core/snapshots/` - Unit test snapshots (e.g., scanner output)
- `tests/snapshots/` - Integration test snapshots (e.g., parsed metadata)

### Snapshot File Naming

Snapshots follow the pattern: `{crate}__{module}__{test_function}.snap`

Examples:
- `sikil__core__scanner__tests__snapshot_scan_result_json.snap`
- `parser_integration_test__snapshot_metadata_minimal.snap`

### Creating Snapshots

```rust
#[test]
fn test_snapshot_metadata_minimal() {
    let fixture_path = fixtures_dir().join("valid").join("minimal-skill.md");
    let metadata = parse_skill_md(&fixture_path).unwrap();
    
    let json = serde_json::to_string_pretty(&metadata).unwrap();
    insta::assert_snapshot!(json);
}
```

### Path Redaction

Dynamic paths (temp directories, user home) must be redacted before snapshotting:

```rust
#[test]
fn test_snapshot_scan_result_json() {
    // ... create temp_dir and scan ...
    
    let json = serde_json::to_string_pretty(&result).unwrap();
    
    // Redact dynamic paths
    let skills_dir_str = skills_dir.to_string_lossy();
    let redacted = json.replace(&*skills_dir_str, "[SKILLS_DIR]");
    
    insta::assert_snapshot!(redacted);
}
```

Redaction markers:
- `[SKILLS_DIR]` - Skills directory path
- `[HOME]` - User home directory
- `[REPO_DIR]` - Managed repository path

### Reviewing Snapshots

```bash
cargo insta review    # Interactive review of pending changes
cargo insta accept    # Accept all pending changes
cargo insta reject    # Reject all pending changes
```

## Test Fixtures

Test fixtures are static files in `tests/fixtures/`:

```
tests/fixtures/
└── skills/
    ├── valid/
    │   ├── complete-skill.md    # All optional fields
    │   ├── complex-metadata.md  # Multi-line descriptions
    │   ├── minimal-skill.md     # Only required fields
    │   └── simple-skill.md      # Basic valid skill
    └── invalid/
        ├── frontmatter-not-at-start.md
        ├── invalid-yaml.md
        ├── missing-description.md
        ├── missing-name.md
        ├── no-frontmatter.md
        └── single-delimiter.md
```

Usage in tests:

```rust
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("skills")
}

#[test]
fn test_parse_valid_skill_minimal() {
    let fixture_path = fixtures_dir().join("valid").join("minimal-skill.md");
    let result = parse_skill_md(&fixture_path);
    assert!(result.is_ok());
}
```

## Test Helpers

The `tests/common/mod.rs` module provides reusable utilities:

| Helper | Purpose |
|--------|---------|
| `sikil_cmd!()` | Creates Command for sikil binary |
| `setup_temp_skill_dir()` | Creates TempDir for tests |
| `create_skill_dir(base, name)` | Creates skill directory structure |
| `create_skill_md(dir, content)` | Writes SKILL.md with custom content |
| `create_minimal_skill_md(dir, name, desc)` | Creates minimal valid SKILL.md |
| `create_complete_skill_md(dir, name, desc)` | Creates SKILL.md with all fields |

## Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test e2e_test

# Specific test function
cargo test test_parse_valid_skill_minimal

# With output
cargo test -- --nocapture

# Only unit tests
cargo test --lib

# Only integration tests
cargo test --test '*'
```

## Test Environment Setup

Integration tests commonly set up isolated environments:

```rust
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create agent skills directory
    let agent_skills = temp_dir.path().join("agents");
    fs::create_dir_all(&agent_skills).expect("Failed to create agent skills dir");
    
    // Create repo directory
    let repo_dir = temp_dir.path().join(".sikil").join("repo");
    fs::create_dir_all(&repo_dir).expect("Failed to create repo dir");
    
    // Write config file
    let config_dir = temp_dir.path().join(".sikil");
    fs::write(config_dir.join("config.toml"), config_content)
        .expect("Failed to write config");
    
    temp_dir
}
```

Key practices:
- Always use `TempDir` for filesystem isolation
- Set `HOME` environment variable to temp directory
- Create minimal config files pointing to temp paths
- Clean up is automatic when `TempDir` is dropped

## Dependencies

- `tempfile` - Temporary directory management
- `assert_cmd` - CLI testing utilities
- `predicates` - Assertion matchers for stdout/stderr
- `insta` - Snapshot testing

## Architecture Mapping

```
Testing Infrastructure
├── Unit Tests          → src/core/*.rs (#[cfg(test)] modules)
├── Integration Tests   → tests/*_test.rs
├── Test Helpers        → tests/common/mod.rs
├── Fixtures            → tests/fixtures/skills/{valid,invalid}/
└── Snapshots           → src/core/snapshots/, tests/snapshots/
```
