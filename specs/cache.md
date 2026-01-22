# Cache Spec

## One-Sentence Description

The cache system stores parsed skill metadata to accelerate repeated directory scans.

## Overview

The cache module provides a SQLite-based caching layer that avoids repeated filesystem scanning by storing skill scan results along with metadata for invalidation (mtime, size, content hash). It uses WAL mode for better concurrent access and supports schema versioning for future migrations.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `SCHEMA_VERSION` | 1 | Current schema version; increment to trigger migration/recreation |
| `MAX_HASH_SIZE` | 64 | Maximum content hash length (SHA256 = 64 hex chars) |

## Storage Format

SQLite database with a single table `scan_cache`:

| Column         | Type    | Description                                |
|----------------|---------|--------------------------------------------|
| path           | TEXT    | Primary key, absolute path to scanned dir  |
| mtime          | INTEGER | Last modification time (unix seconds)      |
| size           | INTEGER | Directory size in bytes                    |
| content_hash   | TEXT    | SHA256 hash (max 64 chars) for content-based invalidation |
| cached_at      | INTEGER | Timestamp when entry was cached            |
| skill_name     | TEXT    | Parsed skill name from SKILL.md (nullable) |
| is_valid_skill | INTEGER | 1 if path contains a valid skill, 0 otherwise |

Additional index: `idx_skill_name` on `skill_name` column.

## Cache Location

`~/.sikil/cache.sqlite` (via `get_cache_path()` in `src/utils/paths.rs`)

## Cache Keys

The **absolute path** to the skill directory serves as the primary key. Each directory that has been scanned gets one cache entry.

## Invalidation

Entries are invalidated when:

1. **mtime mismatch**: On `get()`, the current directory mtime is compared to the cached mtime. If different, `None` is returned to trigger a rescan.
2. **Path no longer exists**: On `get()`, if the path doesn't exist, `None` is returned.
3. **Manual invalidation**: `invalidate(path)` explicitly removes an entry.
4. **Stale cleanup**: `clean_stale()` removes all entries for paths that no longer exist on the filesystem.
5. **Full clear**: `clear()` removes all entries.
6. **Schema migration**: If `SCHEMA_VERSION` changes (currently v1), the table is dropped and recreated.

## Cache Operations

Defined by the `Cache` trait:

| Operation      | Description                                              |
|----------------|----------------------------------------------------------|
| `get(path)`    | Returns cached entry if valid (mtime matches), else None |
| `put(entry)`   | Inserts or replaces an entry (validates hash length ≤64) |
| `invalidate(path)` | Deletes entry for a specific path                   |
| `clean_stale()` | Removes entries for non-existent paths, returns count  |
| `clear()`      | Deletes all entries                                      |

Additional internal operations:
- `open()` / `open_at(path)`: Opens or creates the database, runs migrations
- `migrate()`: Checks schema version, recreates table if version mismatch

## Dependencies

- `rusqlite`: SQLite database driver
- `fs-err`: Filesystem operations with better error messages
- `serde`: Serialization for `ScanEntry`
- `crate::core::SikilError`: Error types
- `crate::utils::paths::{ensure_dir_exists, get_cache_path}`: Path utilities

## Used By

- `src/core/scanner.rs`: The `Scanner` uses the cache to avoid re-parsing unchanged skill directories. Cache usage can be disabled via `--no-cache` flag (`with_cache(config, false)`).
