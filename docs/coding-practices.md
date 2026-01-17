# Sikil Coding Practices

## 1. Error Handling

- **core/**: Use `thiserror`. Return `Result<T, core::Error>`. Every variant includes context fields (paths, names, URLs).
- **commands/**: Use `anyhow`. Wrap core calls with `.with_context(|| "action")`.
- **cli/**: Map errors to stable exit codes (0=success, 2=validation, 3=IO, 4=git, 5=conflict).

## 2. Serde Patterns

- Enums: `#[serde(rename_all = "kebab-case")]`
- Optional fields: `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Config/frontmatter: `#[serde(deny_unknown_fields)]`

## 3. Filesystem Operations

- Use `fs-err` over `std::fs` for better error messages.
- Use `tempfile::tempdir_in(parent)` for same-filesystem atomicity.

### Symlinks

- **Create**: Symlink to `.name.tmp.{pid}` in same directory, then `rename()` to final.
- **Delete**: `rename()` to `.name.deleted.{pid}`, then `remove_file()`.
- **Validate**: Use `symlink_metadata()`. Only delete if target resolves under `~/.sikil/repo/`.

## 4. Git Subprocess

```rust
Command::new("git")
    .arg("-c").arg("protocol.file.allow=never")
    .arg("-c").arg("core.hooksPath=/dev/null")
    .arg("--depth=1")
    .arg("--")  // before URL
    .arg(url).arg(dest)
    .env("GIT_TERMINAL_PROMPT", "0")
```

Validate GitHub-only URLs before calling git. Reject whitespace, NUL, leading `-`, `file://`.

## 5. YAML Frontmatter

- File MUST start with `---\n`. Find closing `---` line. Hard-fail if malformed.
- Cap: 1MB file, 64KB frontmatter.
- Validate `name` with skill-name regex immediately after parsing.

## 6. SQLite Cache

- Key: `(path, mtime, size, content_hash)`
- Fast path: skip parse if `(mtime, size)` match.
- Use `PRAGMA user_version` for schema versioning; drop/recreate on mismatch.
- WAL mode. Wrap scan updates in single transaction.
- Delete stale entries (path no longer exists) after each scan.

## 7. CLI Output

- Commands return serializable structs—never print directly.
- JSON mode (`--json`): stdout is always valid JSON, including errors as `{"error": {"code": "...", "message": "..."}}`.
- Human mode: output to stdout, errors to stderr.
- Use `Vec`/`BTreeMap` for deterministic ordering.
