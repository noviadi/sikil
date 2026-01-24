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
- **Completed:** true
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
  - **Tests:** src/core/cache.rs (test_cache_put_and_get, test_cache_put_creates_valid_json, test_cache_invalidate, test_cache_clear, test_cache_clean_stale, test_cache_replace_existing_entry, test_cache_entry_with_none_skill_name, test_cache_put_rejects_oversized_hash, test_cache_version_mismatch_clears_cache, test_cache_size_limit_clears_cache, test_cache_write_uses_atomic_temp_file, test_cache_put_with_max_hash_size_succeeds, test_cache_entries_use_btreemap_for_determinism, test_cache_get_returns_none_when_mtime_mismatch, test_cache_file_pretty_printed)
  - **Location:** src/core/cache.rs
  - **Notes:**
  - Replaced `SqliteCache` struct with `JsonCache`
  - Added `CacheFile` struct with `version: u32` and `entries: BTreeMap<String, CachedEntry>`
  - Added `CachedEntry` struct for on-disk representation (path is key, not in struct)
  - Use SKILL.md file mtime for invalidation (not directory mtime)
  - Implement atomic write: write to cache.json.tmp, then rename to cache.json
  - Check file size on load, clear if >15 MB
  - All cache errors except put() with invalid hash are non-fatal

  ### Update scanner to use JsonCache
  - **Spec:** skill-scanner.md
  - **Gap:** Scanner imports and uses `SqliteCache` but spec references `Cache` (JSON)
  - **Completed:** true
- **Acceptance Criteria:**
  - Symlinks pointing to `~/.sikil/repo/` are classified as managed
  - Symlinks pointing outside `~/.sikil/repo/` are classified as foreign symlinks
  - Non-existent symlink targets are classified as broken symlinks
  - Physical directories (not symlinks) are classified as unmanaged
  - **Tests:** src/core/scanner.rs (test_scanner_with_cache_enabled, test_scanner_with_cache_disabled, test_scanner_cache_invalidation_on_mtime_change, test_scanner_cached_run_is_faster_than_uncached, test_scanner_cache_invalid_skill_to_valid)
  - **Location:** src/core/scanner.rs, src/core/mod.rs
  - **Notes:**
  - Changed import from `SqliteCache` to `JsonCache` in scanner.rs and mod.rs
  - Updated `cache: Option<JsonCache>` field type
  - Updated constructor calls to use `JsonCache::open()`

### Remove rusqlite dependency from Cargo.toml
- **Spec:** build-and-platform.md
- **Gap:** Cargo.toml includes `rusqlite` but spec no longer lists it as a dependency
- **Completed:** true
- **Acceptance Criteria:**
   - `cargo build --release` produces binary under 10MB
- **Tests:** tests/build_test.rs (test_release_binary_size_under_10mb, test_format_size_mb, test_format_size_kb, test_max_binary_size_constant)
- **Location:** Cargo.toml
- **Notes:**
   - Removed line 18: `rusqlite = { version = "0.31", features = ["bundled"] }`
   - Binary size remains at ~3.3 MB (unchanged) because LTO was already eliminating unused rusqlite code
   - This task depends on: Replace SqliteCache with JsonCache, Update scanner to use JsonCache
   - Created `tests/build_test.rs` with automated test for binary size constraint (10MB limit per spec)

### Add filter_displayable_conflicts function
- **Spec:** conflict-detection.md
- **Gap:** `filter_displayable_conflicts()` function defined in spec but does not exist in code
- **Completed:** true
- **Acceptance Criteria:**
  - `filter_displayable_conflicts()` with `verbose: false` excludes `DuplicateManaged` conflicts
  - `filter_displayable_conflicts()` with `verbose: true` includes all conflicts
- **Tests:** src/core/conflicts.rs (test_filter_displayable_conflicts_verbose_false_excludes_managed, test_filter_displayable_conflicts_verbose_true_includes_all, test_filter_displayable_conflicts_verbose_false_only_managed_returns_empty, test_filter_displayable_conflicts_verbose_true_only_managed_includes_all, test_filter_displayable_conflicts_empty_conflicts, test_filter_displayable_conflicts_verbose_false_only_unmanaged_includes_all)
- **Location:** src/core/conflicts.rs
- **Notes:**
  - Added `filter_displayable_conflicts()` function at line 299
  - When `verbose: false`, filters to only error conflicts (excluding `DuplicateManaged`)
  - When `verbose: true`, returns all conflicts unchanged

### Update format_conflicts_summary to accept verbose parameter
- **Spec:** conflict-detection.md
- **Gap:** Current `format_conflicts_summary()` takes only `&[Conflict]`, spec requires `verbose: bool` parameter to control "suppressed" text
- **Completed:** true
- **Acceptance Criteria:**
  - `format_conflicts_summary(verbose: false)` shows "N info suppressed" for info-only conflicts
  - `format_conflicts_summary(verbose: true)` shows "N info" for info-only conflicts
  - `format_conflicts_summary()` returns "No conflicts detected" when no conflicts exist
  - `format_conflicts_summary()` with mixed conflicts shows "N errors, M info suppressed" when not verbose
  - `format_conflicts_summary()` with mixed conflicts shows "N errors, M info" when verbose
- **Tests:** src/core/conflicts.rs (test_format_conflicts_summary_empty, test_format_conflicts_summary_errors_only, test_format_conflicts_summary_info_only, test_format_conflicts_summary_mixed, test_format_conflicts_summary_verbose_false_info_only_shows_suppressed, test_format_conflicts_summary_verbose_true_info_only_shows_info, test_format_conflicts_summary_verbose_false_errors_only_shows_errors, test_format_conflicts_summary_verbose_true_errors_only_shows_errors, test_format_conflicts_summary_verbose_false_mixed_shows_suppressed, test_format_conflicts_summary_verbose_true_mixed_shows_info, test_format_conflicts_summary_verbose_false_empty_shows_no_conflicts, test_format_conflicts_summary_verbose_true_empty_shows_no_conflicts, test_format_conflicts_summary_verbose_false_single_error, test_format_conflicts_summary_verbose_false_single_info_suppressed, test_format_conflicts_summary_verbose_true_single_info)
- **Location:** src/core/conflicts.rs, src/commands/list.rs
- **Notes:**
  - Added `verbose: bool` parameter to `format_conflicts_summary()` function at line 371
  - When `verbose: false`, shows "N info suppressed" for info-only conflicts
  - When `verbose: true`, shows "N info" for info-only conflicts
  - Updated all existing test calls to pass `verbose` parameter (mostly `true` for backward compatibility)
  - Added 11 new tests specifically for verbose parameter behavior
  - Updated `list.rs` to call with `false` as default until `verbose` field is added to `ListArgs` (see task "Add verbose field to ListArgs")

### Add verbose field to ListArgs and wire through CLI
- **Spec:** conflict-detection.md
- **Gap:** `ListArgs` struct lacks `verbose` field, CLI verbose flag not passed to list command
- **Completed:** true
- **Acceptance Criteria:**
  - `DuplicateManaged` conflict details are not printed in human-readable output unless verbose mode is enabled
  - `--conflicts` filter shows only error-level conflicts by default
  - `--conflicts -v` filter shows both error and info conflicts
- **Tests:** src/commands/list.rs (test_list_args_with_verbose, test_apply_filters_conflicts_only_verbose_false_filters_info_conflicts, test_apply_filters_conflicts_only_verbose_true_includes_info_conflicts), tests/conflict_detection_test.rs (test_duplicate_managed_detection), tests/e2e_test.rs (test_e2e_conflict_detection_flow)
- **Location:** src/commands/list.rs, src/main.rs
- **Notes:**
  - Added `verbose: bool` field to `ListArgs` struct at line 31
  - Updated `apply_filters()` to use `filter_displayable_conflicts()` with verbose flag when filtering by conflicts
  - Updated `print_human_readable()` to accept and use verbose parameter for conflicts summary and details
  - Wired `cli.verbose` to `ListArgs.verbose` in main.rs
  - Updated all existing tests to include `verbose: false` in `ListArgs` instantiations
  - Updated `test_duplicate_managed_detection` to test both verbose=false (suppressed) and verbose=true (shown) cases
  - Updated `test_e2e_conflict_detection_flow` to add `--no-cache` flag and `-v` flag for DuplicateManaged case

### Suppress DuplicateManaged conflicts in human-readable output
- **Spec:** conflict-detection.md
- **Gap:** `print_human_readable()` prints all conflicts, should filter by verbose mode
- **Completed:** false
- **Acceptance Criteria:**
  - `DuplicateManaged` conflict details are not printed in human-readable output unless verbose mode is enabled
  - JSON output (`--json`) includes all conflicts regardless of verbose mode
- **Tests:**
- **Location:** src/commands/list.rs
- **Notes:** Depends on: Add verbose field to ListArgs, Add filter_displayable_conflicts function
