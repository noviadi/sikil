# Error Handling Spec

## One-Sentence Description

Error handling defines structured error types with actionable messages.

## Overview

Sikil uses a two-tier error handling strategy:
1. **Domain layer** (`src/core/errors.rs`): `SikilError` and `ConfigError` enums using `thiserror` for type-safe, structured errors with rich context
2. **Command layer** (`src/commands/`): Uses `anyhow::Result` to propagate errors up the call stack
3. **CLI layer** (`src/main.rs`): Catches errors, prints to stderr with `eprintln!("Error: {}", e)`, and exits with code 1

## Error Types

### SikilError (Primary Domain Error)

| Variant | Fields | Message Template |
|---------|--------|------------------|
| `InvalidSkillMd` | `path: PathBuf`, `reason: String` | `"Invalid SKILL.md in {path}: {reason}"` |
| `SkillNotFound` | `name: String` | `"Skill not found: {name}"` |
| `DirectoryNotFound` | `path: PathBuf` | `"Directory not found: {path}"` |
| `SymlinkError` | `reason: String`, `source: Option<io::Error>` | `"Symlink error: {reason}"` |
| `GitError` | `reason: String` | `"Git error: {reason}"` |
| `ConfigError` | `reason: String` | `"Configuration error: {reason}"` |
| `AlreadyExists` | `resource: String` | `"Already exists: {resource}"` |
| `PermissionDenied` | `operation: String`, `path: PathBuf` | `"Permission denied: {operation} on {path}"` |
| `ValidationError` | `reason: String` | `"Validation failed: {reason}"` |
| `PathTraversal` | `path: String` | `"Path traversal detected: {path}"` |
| `SymlinkNotAllowed` | `reason: String` | `"Symlink not allowed: {reason}"` |
| `InvalidGitUrl` | `url: String`, `reason: String` | `"Invalid Git URL: {url} - {reason}"` |
| `ConfigTooLarge` | `size: u64` | `"Configuration file too large: {size} bytes (maximum 1048576 bytes)"` |

### ConfigError (Configuration-Specific)

| Variant | Fields | Message Template |
|---------|--------|------------------|
| `FileRead` | `String` | `"Failed to read config file: {0}"` |
| `InvalidToml` | `String` | `"Invalid TOML in config: {0}"` |
| `ConfigTooLarge` | `u64` | `"Configuration file too large: {0} bytes (maximum 1048576 bytes)"` |

## Error Messages

Messages follow a consistent pattern:
- **Prefix**: Category name (e.g., "Invalid SKILL.md", "Permission denied", "Validation failed")
- **Context**: Resource identifiers (paths, names, URLs)
- **Reason**: Specific failure cause embedded in the message

Example: `"Permission denied: write on /protected/file"`

## Error Context

Context is provided through structured fields:
- **Paths**: `PathBuf` for filesystem locations
- **Names**: `String` for skill/resource identifiers
- **Operations**: `String` describing the attempted action
- **Reasons**: `String` with specific failure details
- **Source chains**: Optional `#[source]` annotation on `SymlinkError` for underlying `io::Error`

## Recovery Suggestions

Recovery information is embedded in the `reason` field at construction time. Commands provide actionable context:

```rust
SikilError::ValidationError {
    reason: format!("Unknown agent '{}'. Valid: amp, claude, cursor, windsurf", agent)
}
```

The CLI layer (`main.rs`) additionally provides help for common cases:
```rust
eprintln!("Valid agents: {}", Agent::all().iter().map(|a| a.cli_name()).collect::<Vec<_>>().join(", "));
```

## Acceptance Criteria

- `SikilError::InvalidSkillMd` displays `"Invalid SKILL.md in {path}: {reason}"`
- `SikilError::SkillNotFound` displays `"Skill not found: {name}"`
- `SikilError::DirectoryNotFound` displays `"Directory not found: {path}"`
- `SikilError::SymlinkError` displays `"Symlink error: {reason}"`
- `SikilError::GitError` displays `"Git error: {reason}"`
- `SikilError::ConfigError` displays `"Configuration error: {reason}"`
- `SikilError::AlreadyExists` displays `"Already exists: {resource}"`
- `SikilError::PermissionDenied` displays `"Permission denied: {operation} on {path}"`
- `SikilError::ValidationError` displays `"Validation failed: {reason}"`
- `SikilError::PathTraversal` displays `"Path traversal detected: {path}"`
- `SikilError::SymlinkNotAllowed` displays `"Symlink not allowed: {reason}"`
- `SikilError::InvalidGitUrl` displays `"Invalid Git URL: {url} - {reason}"`
- `SikilError::ConfigTooLarge` displays `"Configuration file too large: {size} bytes (maximum 1048576 bytes)"`
- `ConfigError::FileRead` displays `"Failed to read config file: {0}"`
- `ConfigError::InvalidToml` displays `"Invalid TOML in config: {0}"`
- `ConfigError::ConfigTooLarge` displays `"Configuration file too large: {0} bytes (maximum 1048576 bytes)"`
- `SikilError::SymlinkError` with `source: Some(io_err)` chains to underlying `io::Error`
- CLI prints `"Error: {message}"` to stderr on error
- CLI exits with code 1 on any error

## Dependencies

- `thiserror` crate for derive macros
- `std::path::PathBuf` for path representation
- `std::io::Error` for source chaining

## Used By

| Component | Usage |
|-----------|-------|
| `src/commands/*.rs` | Construct and propagate `SikilError` via `anyhow::Result` |
| `src/core/config.rs` | Uses `ConfigError` for configuration parsing |
| `src/core/scanner.rs` | Returns `SikilError` for scan failures |
| `src/core/parser.rs` | Returns `SikilError::InvalidSkillMd` and `ValidationError` |
| `src/utils/git.rs` | Returns `SikilError::InvalidGitUrl` |
| `src/utils/symlink.rs` | Returns `SikilError::SymlinkError` |
| `src/utils/atomic.rs` | Returns various `SikilError` variants for filesystem operations |
| `src/main.rs` | Catches all errors, prints to stderr, exits with code 1 |
