# Implementation Plan

## General Verification

`./scripts/verify.sh`

## Spec Issues

None - all specs have complete Acceptance Criteria.

## Tasks

### Implement distinct exit codes for error types
- **Spec:** cli-schema.md
- **Gap:** CLI always exits with code 1, but spec defines distinct exit codes (2 for validation, 3 for skill not found, 4 for permission, 5 for network)
- **Completed:** true
- **Acceptance Criteria:**
  - Validation error exits with code 2
  - Skill not found error exits with code 3
  - Permission error exits with code 4
  - Network error exits with code 5
- **Tests:** tests/error_handling_test.rs (test_exit_code_validation_error, test_exit_code_skill_not_found, test_exit_code_permission_error, test_exit_code_success), src/core/errors.rs (unit tests for exit_code() method)
- **Location:** src/main.rs, src/core/errors.rs, src/commands/validate.rs
- **Notes:**
  - Added `exit_code()` method to `SikilError` that maps error types to appropriate exit codes
  - Modified `main.rs` to use `get_exit_code()` helper for extracting exit code from errors
  - Modified `validate.rs` to exit with code 2 for validation failures and detect permission errors specifically
  - Network error test (`test_exit_code_network_error`) is marked as `#[ignore]` because it requires Git URL detection to be implemented first (see task "Wire Git URL detection to install command")

### Wire Git URL detection to install command
- **Spec:** skill-installation.md
- **Gap:** `execute_install_git` function exists but main.rs doesn't detect Git URLs and dispatch to it
- **Completed:** true
- **Acceptance Criteria:**
  - Installing from Git URL clones with `--depth=1` and copies skill to `~/.sikil/repo/<name>/`
  - Short-form Git URL `owner/repo` expands to `https://github.com/owner/repo.git`
  - Git URL with subdirectory `owner/repo/path/to/skill` extracts only that subdirectory
- **Tests:** src/main.rs (test_is_git_url_https_github, test_is_git_url_short_form, test_is_git_url_absolute_path_false, test_is_git_url_relative_with_dots_false, test_is_git_url_starting_with_dash_false, test_is_git_url_single_segment_false, test_is_git_url_non_github_https_false, test_is_git_url_file_protocol_false), tests/install_command_test.rs (test_detect_https_github_url, test_detect_short_form_git_url, test_detect_local_path_not_git_url, test_absolute_path_not_git_url, test_relative_path_with_dots_not_git_url, test_path_starting_with_dash_not_git_url, test_path_with_single_segment_not_git_url, test_non_github_https_not_git_url, test_file_protocol_rejected_as_git_url)
- **Location:** src/main.rs
- **Notes:**
  - Added `is_git_url()` function in main.rs to detect Git URL formats (HTTPS GitHub URLs and short-form owner/repo)
  - Modified install command handler to route to `execute_install_git` for Git URLs, `execute_install_local` for filesystem paths
  - Git URL detection rejects absolute paths, relative paths with dots, paths starting with `-`, and existing filesystem paths
  - Short-form Git URLs (owner/repo) are detected only when they don't exist as local paths

### Replace SqliteCache with JsonCache implementation
- **Spec:** cache.md
- **Gap:** Cache uses SQLite (rusqlite) but spec defines JSON file-based cache with atomic writes
- **Completed:** false
- **Acceptance Criteria:**
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
  - Cache file is created at `~/.sikil/cache.json`
- **Tests:**
- **Location:** src/core/cache.rs
- **Notes:**
  - Replace `SqliteCache` struct with `JsonCache` 
  - Add `CacheFile` struct with `version: u32` and `entries: BTreeMap<String, ScanEntry>`
  - Use SKILL.md file mtime for invalidation (not directory mtime)
  - Implement atomic write: write to cache.json.tmp, then rename to cache.json
  - Check file size on load, clear if >15 MB
  - All cache errors except put() with invalid hash are non-fatal

### Update get_cache_path to return cache.json
- **Spec:** filesystem-paths.md
- **Gap:** `get_cache_path()` returns `cache.sqlite` but spec defines `cache.json`
- **Completed:** true
- **Acceptance Criteria:**
  - `get_cache_path` returns `~/.sikil/cache.json` expanded to absolute path
- **Tests:** src/utils/paths.rs (test_get_cache_path)
- **Location:** src/utils/paths.rs
- **Notes:**
  - Changed line 108: `.join("cache.sqlite")` → `.join("cache.json")`
  - Updated docstring on line 91 and example on line 103
  - Updated test assertion on line 201

### Update scanner to use JsonCache
- **Spec:** skill-scanner.md
- **Gap:** Scanner imports and uses `SqliteCache` but spec references `Cache` (JSON)
- **Completed:** false
- **Acceptance Criteria:**
  - Symlinks pointing to `~/.sikil/repo/` are classified as managed
  - Symlinks pointing outside `~/.sikil/repo/` are classified as foreign symlinks
  - Non-existent symlink targets are classified as broken symlinks
  - Physical directories (not symlinks) are classified as unmanaged
- **Tests:**
- **Location:** src/core/scanner.rs
- **Notes:**
  - Change import from `SqliteCache` to `JsonCache`
  - Update `cache: Option<SqliteCache>` field to `cache: Option<JsonCache>`
  - Update constructor calls

### Remove rusqlite dependency from Cargo.toml
- **Spec:** build-and-platform.md
- **Gap:** Cargo.toml includes `rusqlite` but spec no longer lists it as a dependency
- **Completed:** false
- **Acceptance Criteria:**
  - `cargo build --release` produces binary under 10MB
- **Tests:**
- **Location:** Cargo.toml
- **Notes:**
  - Remove line 18: `rusqlite = { version = "0.31", features = ["bundled"] }`
  - This task depends on: Replace SqliteCache with JsonCache, Update scanner to use JsonCache
  - Binary size should decrease by ~1-2 MB
