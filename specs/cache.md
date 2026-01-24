# Cache Spec

## One-Sentence Description

The cache system stores parsed skill metadata to accelerate repeated directory scans.

## Overview

The cache module provides a JSON file-based caching layer that avoids repeated filesystem scanning by storing skill scan results along with metadata for invalidation (mtime, size, content hash). It uses atomic writes (temp file + rename) and accepts last-writer-wins semantics for simplicity.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `CACHE_VERSION` | 1 | Cache format version; version mismatch triggers full rebuild |
| `MAX_HASH_SIZE` | 64 | Maximum content hash length (SHA256 = 64 hex chars) |
| `MAX_CACHE_SIZE` | 15 MB | Maximum cache file size; exceeding triggers full clear |

## Storage Format

JSON file with structure:

```json
{
  "version": 1,
  "entries": {
    "/absolute/path/to/skill": {
      "mtime": 1706000000,
      "size": 4096,
      "content_hash": "abc123...",
      "cached_at": 1706000000,
      "skill_name": "my-skill",
      "is_valid_skill": true
    }
  }
}
```

### CacheFile

| Field | Type | Description |
|-------|------|-------------|
| `version` | `u32` | Cache format version for compatibility |
| `entries` | `BTreeMap<String, ScanEntry>` | Path-keyed cache entries (absolute path strings, sorted for determinism) |

### ScanEntry (API type)

| Field | Type | Description |
|-------|------|-------------|
| `path` | `PathBuf` | Absolute path to the skill directory |
| `mtime` | `u64` | SKILL.md file modification time (unix seconds) |
| `size` | `u64` | SKILL.md file size in bytes |
| `content_hash` | `String` | SHA256 hash of SKILL.md (max 64 chars) |
| `cached_at` | `u64` | Timestamp when entry was cached |
| `skill_name` | `Option<String>` | Parsed skill name from SKILL.md |
| `is_valid_skill` | `bool` | Whether path contains a valid skill |

Note: On disk, `path` is stored as the map key (string), not inside the entry.

## Cache Location

`~/.sikil/cache.json` (via `get_cache_path()` in `src/utils/paths.rs`)

## Cache Keys

The **absolute path** to the skill directory serves as the key. Each scanned directory gets one cache entry.

## Invalidation

Entries are invalidated when:

1. **mtime mismatch**: On `get()`, the current SKILL.md file mtime is compared to cached mtime. If different, `None` is returned.
2. **Path no longer exists**: On `get()`, if the SKILL.md file doesn't exist, `None` is returned.
3. **Manual invalidation**: `invalidate(path)` explicitly removes an entry.
4. **Stale cleanup**: `clean_stale()` removes entries for paths that no longer exist.
5. **Full clear**: `clear()` removes all entries.
6. **Version mismatch**: If `version` differs from `CACHE_VERSION`, all entries are discarded.
7. **Size limit exceeded**: If cache file exceeds `MAX_CACHE_SIZE`, all entries are cleared.

## Cache Operations

Defined by the `Cache` trait:

| Operation | Description |
|-----------|-------------|
| `get(path)` | Returns cached entry if valid (mtime matches), else None |
| `put(entry)` | Inserts or replaces an entry (validates hash length ≤64) |
| `invalidate(path)` | Deletes entry for a specific path |
| `clean_stale()` | Removes entries for non-existent paths, returns count |
| `clear()` | Deletes all entries |

### Write Semantics

- **Atomic writes**: Write to `cache.json.tmp`, then rename to `cache.json`
- **Concurrency**: Last writer wins (no locking)
- **Error tolerance**: Cache read/write failures are non-fatal; treat as cache miss

## Acceptance Criteria

- `get(path)` returns cached entry when SKILL.md mtime matches cached mtime
- `get(path)` returns `None` when SKILL.md mtime differs from cached mtime
- `get(path)` returns `None` when SKILL.md file no longer exists
- `get(path)` returns `Ok(None)` on cache read/parse failure (non-fatal)
- `put(entry)` inserts new entry with path as key
- `put(entry)` replaces existing entry when path matches
- `put(entry)` rejects content hash exceeding 64 characters
- `invalidate(path)` removes entry for the specified path
- `clean_stale()` removes all entries for non-existent paths
- `clean_stale()` returns count of removed entries
- `clear()` removes all entries from cache
- Cache version mismatch triggers full cache clear
- Cache file exceeding 15 MB triggers full cache clear
- Write uses atomic temp file + rename pattern
- Cache read failure is treated as cache miss (non-fatal)
- Cache write failure is non-fatal (operation continues)
- `--no-cache` flag bypasses cache for scan operations
- Cache file is created at `~/.sikil/cache.json`

## Error Handling

| Error | Handling |
|-------|----------|
| Cache file not found | Treat as empty cache, create on first write |
| JSON parse failure | Return `Ok(None)` from `get()`, rebuild on next write |
| Version mismatch | Clear all entries, continue with empty cache |
| Size limit exceeded | Clear all entries, continue with empty cache |
| Write failure | Silent (best-effort), continue without caching |
| Hash too long | Return `Err(SikilError::ValidationError)` from `put()` |

Only `put()` with invalid hash returns an error. All other failures are treated as cache misses to ensure cache never blocks operations.

## Dependencies

- `serde`: Serialization/deserialization
- `serde_json`: JSON format
- `fs-err`: Filesystem operations with better error messages
- `crate::core::SikilError`: Error types
- `crate::utils::paths::{ensure_dir_exists, get_cache_path}`: Path utilities

## Used By

- `src/core/scanner.rs`: The `Scanner` uses the cache to avoid re-parsing unchanged skill directories. Cache usage can be disabled via `--no-cache` flag (`with_cache(config, false)`).
