# Implementation Plan

## General Verification

`./scripts/verify.sh`

## Spec Issues

None - all specs reviewed have complete Acceptance Criteria.

## Tasks

> Tasks below close v0.2 gaps introduced by `specs/project-manifest.md`,
> `specs/provenance.md`, and the 13 specs updated in the v0.2 refactor. They
> are grouped by dependency order: foundational utilities first, then the
> manifest/provenance layers, then scanner/conflict-detection updates, then
> CLI schema and command implementations.

---

### Foundation: Layer 1 — utilities

### Add v0.2 error variants and exit-code mapping
- **Spec:** error-handling.md
- **Gap:** `SikilError` lacks `ManifestNotFound`, `ManifestParseError`, `LockfileMismatch`, `OutsideProject`, and `SourceUnreachable`. Exit-code mapping for these variants does not exist.
- **Completed:** true
- **Acceptance Criteria:**
  - `SikilError::ManifestNotFound` displays `"Manifest not found at {path}"`
  - `SikilError::ManifestParseError` displays `"Invalid manifest at {path}: {reason}"`
  - `SikilError::LockfileMismatch` displays `"Lockfile mismatch: {reason}"`
  - `SikilError::OutsideProject` displays the literal `"No project found; run \`sikil init\` or use --global"`
  - `SikilError::SourceUnreachable` displays `"Source unreachable: {source} - {reason}"`
  - `SikilError::ManifestNotFound` exits with code 3 (analogous to `SkillNotFound`)
  - `SikilError::ManifestParseError` exits with code 2 (validation)
  - `SikilError::LockfileMismatch` exits with code 2 (validation)
  - `SikilError::OutsideProject` exits with code 2 (validation)
  - `SikilError::SourceUnreachable` exits with code 5 (network) for git sources, code 3 for missing local paths
- **Tests:** src/core/errors.rs — `test_error_display_manifest_not_found`, `test_error_display_manifest_parse_error`, `test_error_display_lockfile_mismatch`, `test_error_display_outside_project`, `test_error_display_source_unreachable`, `test_exit_code_manifest_not_found`, `test_exit_code_manifest_parse_error`, `test_exit_code_lockfile_mismatch`, `test_exit_code_outside_project`, `test_exit_code_source_unreachable_git_url`, `test_exit_code_source_unreachable_short_form_git`, `test_exit_code_source_unreachable_local_path`, `test_exit_code_source_unreachable_relative_path`
- **Location:** src/core/errors.rs, src/main.rs
- **Notes:** The `SourceUnreachable` struct field is named `src` (not `source`) to avoid thiserror's automatic `#[source]` detection on fields named `source`. The display output matches the spec exactly. No main.rs changes needed — the existing `get_exit_code()` already delegates to `SikilError::exit_code()`.

### Add project-root discovery utilities
- **Spec:** filesystem-paths.md
- **Gap:** `find_project_root`, `get_project_root`, `get_manifest_path`, `get_lock_path`, and `get_project_skills_path` do not exist in `src/utils/paths.rs`.
- **Completed:** false
- **Acceptance Criteria:**
  - `find_project_root` returns the directory containing `.sikil/manifest.toml` when present, walking upward from the start path
  - `find_project_root` returns the directory containing `.git` (file or directory) when no manifest is found higher up
  - `find_project_root` prefers `.sikil/manifest.toml` over `.git` when both are present at different levels (manifest wins regardless of depth)
  - `find_project_root` returns `None` when neither marker is found before reaching the filesystem root
  - `find_project_root` treats a `.git` file (worktree marker) equivalently to a `.git/` directory
  - `get_manifest_path` returns `<project_root>/.sikil/manifest.toml`
  - `get_lock_path` returns `<project_root>/.sikil/lock.toml`
  - `get_project_skills_path` returns `<project_root>/.sikil/skills/`
- **Tests:**
- **Location:** src/utils/paths.rs
- **Notes:**

### Add atomic_write_file utility
- **Spec:** atomic-operations.md
- **Gap:** `atomic_write_file` is required by manifest, lockfile, and sidecar writes but is not present in `src/utils/atomic.rs`.
- **Completed:** false
- **Acceptance Criteria:**
  - `atomic_write_file` writes to a sibling temp file in the destination's parent directory
  - `atomic_write_file` calls `sync_all()` on the temp file before renaming
  - `atomic_write_file` leaves the destination unchanged if the rename fails
  - `atomic_write_file` removes the temp file on any failure (no orphaned `.tmp.*` files)
  - `atomic_write_file` creates the parent directory if it does not exist
  - `atomic_write_file` returns `SikilError::PermissionDenied` when the destination directory is not writable
- **Tests:**
- **Location:** src/utils/atomic.rs
- **Notes:**

### Recognize project store in is_managed_symlink and create_symlink
- **Spec:** symlink-operations.md
- **Gap:** `is_managed_symlink` only checks `~/.sikil/repo/`; spec requires it to also accept any reachable `<project_root>/.sikil/skills/`. `create_symlink` must record relative `src` verbatim for portable project symlinks.
- **Completed:** false
- **Acceptance Criteria:**
  - `is_managed_symlink` returns true if symlink target is under `~/.sikil/repo/`
  - `is_managed_symlink` returns true if symlink target resolves to a path under `<project_root>/.sikil/skills/` for the project root reachable from the symlink's location
  - `is_managed_symlink` returns false for broken symlinks
  - `is_managed_symlink` returns false if symlink target is outside both managed stores
  - `create_symlink` with a relative `src` records that relative path verbatim as the symlink target
- **Tests:**
- **Location:** src/utils/symlink.rs
- **Notes:** depends on filesystem-paths task (needs `find_project_root`, `get_project_skills_path`)

---

### Foundation: Layer 2 — manifest and provenance layers

### Implement manifest read/write/reconcile layer
- **Spec:** project-manifest.md
- **Gap:** No `src/core/manifest.rs` (or equivalent) implementing manifest schema parsing, lockfile schema parsing, or the reconcile algorithm.
- **Completed:** false
- **Acceptance Criteria:**
  - Project root discovery walks up from cwd, finding `.sikil/manifest.toml` before `.git/`
  - A `.git` file (worktree marker) is treated equivalently to a `.git/` directory for root discovery
  - `--global` flag forces global scope regardless of project root presence
  - `--project` flag in absence of project root returns `OutsideProject` error
  - `sikil install` (no args) inside a project loads manifest, fetches missing sources, vendors them to `.sikil/skills/<name>/`, and creates relative agent symlinks
  - `sikil install` (no args) without a manifest returns `ManifestNotFound`
  - `sikil install <source>` inside a project adds an entry to the manifest before reconciling
  - Lockfile entry's `commit` matches the resolved git commit SHA exactly
  - Lockfile entry's `content_hash` is `sha256:<hex>` of the vendored tree excluding `.git/`
  - Manifest entries with no matching lockfile entry trigger fresh resolution
  - Lockfile entries with no matching manifest entry trigger removal of vendored bytes and symlinks (with confirmation unless `--yes`)
  - Manifest entry with `agents = ["claude-code"]` creates a symlink only at the claude-code workspace path
  - Manifest entry without `agents` field creates symlinks at every enabled agent's workspace path
  - Multi-skill source (same `source`, different `subdir`) clones the source once per install operation
  - Local-path source resolves relative to project root and records absolute path in lockfile
  - Local-path source where the path does not exist returns `SourceUnreachable`
  - Manifest schema_version other than `1` returns `ManifestParseError`
  - Manifest with unknown top-level fields returns `ManifestParseError` (deny_unknown_fields)
  - Manifest exceeding 1 MB returns `ManifestParseError` (size cap matches config)
  - Re-running `sikil install` immediately after a successful install produces no filesystem changes
  - Manifest comments and entry ordering are preserved across `sikil add` on a best-effort basis
  - Agent symlinks committed to git remain valid after `git clone` to a different absolute path (because they are relative)
  - Manifest writes are atomic (temp file + rename); a crash mid-write leaves the previous manifest intact
  - Lockfile writes are atomic (temp file + rename)
- **Tests:**
- **Location:** src/core/manifest.rs (new), src/commands/install.rs
- **Notes:** This is the largest task in the plan. Implementing it requires the new commands `install` (no-args reconciliation) to be wired in `src/main.rs`. AC bullets that belong to other commands (`init`, `update`, `add`, `remove`) are split out into their own tasks below.

### Implement provenance sidecar layer
- **Spec:** provenance.md
- **Gap:** No `src/core/provenance.rs` (or equivalent) implementing sidecar read/write, content-hash computation, or unknown-origin classification.
- **Completed:** false
- **Acceptance Criteria:**
  - After `sikil install <source> --global`, `~/.sikil/repo/<name>/.sikil-source.toml` exists with all required fields populated
  - `source` field in the sidecar matches verbatim the argument passed to `sikil install`
  - `resolved_url` field is the expanded clone URL or an absolute local path
  - `commit` field is the full git commit SHA for git sources, empty string for local-path sources
  - `content_hash` field is `sha256:<hex>` and verifiably matches the SHA-256 of the skill bytes (excluding `.git/` and the sidecar)
  - `installed_at` field is a valid RFC 3339 UTC timestamp
  - `installer` field is `sikil/<version>` matching the running binary's version
  - Sidecar is written atomically via temp file + rename
  - Sidecar write failure after successful content copy is non-fatal: install reports a warning and exits 0
  - Sidecar write failure produces a warning containing the substring "provenance sidecar"
  - `sikil show <name>` displays source, commit, and installed_at when sidecar present
  - `sikil show <name>` displays `(unknown origin)` when sidecar absent
  - `sikil list --json` per-installation entry contains `"source": "<source>"` when sidecar present
  - `sikil list --json` per-installation entry contains `"source": null` when sidecar absent
  - `sikil update <name>` on unknown-origin skill returns `ValidationError` with the substring "source is unknown"
  - `sikil update <name>` on a known-origin skill re-fetches from `resolved_url` at the original `source`'s ref-style (recorded by re-reading sidecar's `source` field; semver-style refs are not implemented in v0.2)
  - Sidecar with `version` other than `1` returns `ValidationError` with substring "unsupported sidecar version"
  - Sidecar with unknown fields returns `ValidationError` (deny_unknown_fields)
  - Sidecar excluded from `content_hash` computation (the hash is stable across reads even if the sidecar changes timestamp)
  - `.git/` directory excluded from `content_hash` computation at any depth
  - Content hash computed for the same skill bytes is identical between global sidecar and project lockfile
- **Tests:**
- **Location:** src/core/provenance.rs (new)
- **Notes:** Some AC bullets are observed via `show`, `list --json`, and `update` — those commands must be updated to read the sidecar. The wiring is part of the consumer command tasks below; this task implements the read/write/hash core.

---

### Foundation: Layer 3 — scanner & conflict-detection updates

### Make scanner project-aware
- **Spec:** skill-scanner.md
- **Gap:** Scanner does not call `find_project_root`, does not anchor workspace paths at project root, and does not include `<project_root>/.sikil/skills/` in the scan.
- **Completed:** false
- **Acceptance Criteria:**
  - When invoked inside a project root, workspace paths are anchored at the project root rather than cwd
  - When invoked inside a project root, the project-managed store at `<project_root>/.sikil/skills/` is included in the scan
  - Symlinks pointing into `<project_root>/.sikil/skills/` are classified as managed (project-managed)
  - A skill present in both the project-managed store and the global repo appears as a single skill with installations from both scopes
- **Tests:**
- **Location:** src/core/scanner.rs
- **Notes:** depends on filesystem-paths task and symlink-operations task

### Recognize project-managed installations in conflict detection
- **Spec:** conflict-detection.md
- **Gap:** `detect_conflicts` only treats `~/.sikil/repo/` as managed; project-managed installations from `<project_root>/.sikil/skills/` are not recognized as a separate scope.
- **Completed:** false
- **Acceptance Criteria:**
  - Installation is classified as managed only when `is_symlink == Some(true)` AND `symlink_target` starts with the global repo path or the project-managed store path
  - Two managed installs are considered duplicates when their resolved `repo_path` is identical
  - A managed installation in `<project>/.sikil/skills/` and a managed installation in `~/.sikil/repo/` for the same skill name are not considered a conflict (they belong to different scopes)
- **Tests:**
- **Location:** src/core/conflicts.rs
- **Notes:**

---

### CLI schema: scope flags & new commands

### Add --global and --project scope flags
- **Spec:** cli-schema.md
- **Gap:** No `--global`/`--project` flags exist on `install`, `add`, `update`, `remove`, `adopt`, `unmanage`, `sync`, `list`, or `show`. Mutual exclusivity not enforced.
- **Completed:** false
- **Acceptance Criteria:**
  - `--global` and `--project` flags are available on `install`, `add`, `update`, `remove`, `adopt`, `unmanage`, `sync`, `list`, `show`
  - `--global` and `--project` together returns argument-parser error
- **Tests:**
- **Location:** src/cli/app.rs, src/main.rs
- **Notes:**

### Implement sikil init command
- **Spec:** project-manifest.md
- **Gap:** No `init` subcommand or `src/commands/init.rs` exists.
- **Completed:** false
- **Acceptance Criteria:**
  - `sikil init` creates `.sikil/manifest.toml` containing `schema_version = 1` at the git root
  - `sikil init` is a no-op (with informational message) when manifest already exists
  - `sikil init` succeeds in any directory and creates `.sikil/manifest.toml` with `schema_version = 1`
- **Tests:**
- **Location:** src/commands/init.rs (new), src/cli/app.rs, src/main.rs
- **Notes:** also adds `--here` flag per cli-schema.md

### Implement sikil add command
- **Spec:** project-manifest.md
- **Gap:** No `add` subcommand or `src/commands/add.rs` exists. Spec defines `sikil add` as project-scope sugar for `install <source> --project`.
- **Completed:** false
- **Acceptance Criteria:**
  - `sikil add <source>` outside a project returns `OutsideProject`
  - `sikil add` outside any project returns `OutsideProject` (exit code 2)
- **Tests:**
- **Location:** src/commands/add.rs (new), src/cli/app.rs, src/main.rs
- **Notes:** delegates materialization to install; `--rev` and `--subdir` are validated by the manifest layer task

### Implement sikil update command
- **Spec:** project-manifest.md
- **Gap:** No `update` subcommand or `src/commands/update.rs` exists.
- **Completed:** false
- **Acceptance Criteria:**
  - `sikil update <name>` re-resolves only the named entry, leaving others untouched
  - `sikil update` with no args re-resolves every manifest entry
  - `sikil update --dry-run` prints diff summary without writing manifest, lockfile, or vendored bytes
  - `sikil update` displays one-line summary `<name>: <old-commit> → <new-commit>` per changed skill
- **Tests:**
- **Location:** src/commands/update.rs (new), src/cli/app.rs, src/main.rs
- **Notes:**

### Add --rev / --subdir to install/add and --dry-run/--yes to update
- **Spec:** cli-schema.md
- **Gap:** install/add lack `--rev` and `--subdir`; update lacks `--dry-run` and `--yes`; install lacks `--yes`. Required for v0.2 manifest workflow.
- **Completed:** false
- **Acceptance Criteria:**
  - `sikil install --rev <ref>` inside a project records `rev = "<ref>"` in the manifest entry
  - `sikil install --subdir <path>` inside a project records `subdir = "<path>"` in the manifest entry
  - `sikil install --to <agents>` inside a project records `agents = [...]` in the manifest entry
- **Tests:**
- **Location:** src/cli/app.rs, src/commands/install.rs, src/commands/add.rs
- **Notes:**

### Allow sikil install with no positional source
- **Spec:** cli-schema.md
- **Gap:** `Install { source, .. }` requires a positional `source`. Spec allows omission for project-mode reconciliation.
- **Completed:** false
- **Acceptance Criteria:**
  - `sikil install` with no source argument outside any project returns `OutsideProject` (exit code 2)
- **Tests:**
- **Location:** src/cli/app.rs, src/main.rs
- **Notes:** behavioral wiring (no-args = reconciliation) is implemented by the manifest layer task; this task only changes the CLI schema to make `source` optional and routes correctly.

---

### Extend existing commands for project scope

### Extend install command for project scope
- **Spec:** skill-installation.md
- **Gap:** `execute_install_local` and `execute_install_git` only target the global repo; project-scope install (vendor to `<project_root>/.sikil/skills/`, write manifest+lockfile entry, create relative symlinks) is not implemented.
- **Completed:** false
- **Acceptance Criteria:**
  - Default scope inside a project root is project (no `--global` needed)
  - `sikil install <source>` inside a project adds a `[skill.<name>]` entry to `.sikil/manifest.toml` and reconciles
  - `sikil install` (no source) inside a project loads manifest+lockfile and reconciles without modifying the manifest
  - `sikil install` (no source) outside any project returns `OutsideProject` with exit code 2
  - Project install vendors skill to `<project_root>/.sikil/skills/<name>/` (not the global repo)
  - Project install creates **relative** symlinks at each agent's `workspace_path` (e.g., `.claude/skills/<name>` → `../../.sikil/skills/<name>`)
  - Project install writes a lockfile entry with `commit` (full SHA), `content_hash` (`sha256:<hex>`), `resolved_url`, and `fetched_at` (RFC 3339)
  - `sikil install --rev <ref>` inside a project records `rev = "<ref>"` in the manifest entry
  - `sikil install --subdir <path>` inside a project records `subdir = "<path>"` in the manifest entry
  - `sikil install --to <agents>` inside a project records `agents = [...]` in the manifest entry
  - Project install failure during symlink creation reverts manifest edit and removes vendored bytes
  - Manifest entry already exists with different `source` returns `AlreadyExists`
  - Lockfile written atomically (temp + rename); a crash mid-write leaves the prior lockfile intact
  - Manifest written atomically (temp + rename); a crash mid-write leaves the prior manifest intact
- **Tests:**
- **Location:** src/commands/install.rs
- **Notes:** the global-scope install AC are already implemented (per prior plan history); this task extends to project scope.

### Wire provenance sidecar write into global install
- **Spec:** skill-installation.md
- **Gap:** `execute_install_local` / `execute_install_git` do not write the `.sikil-source.toml` sidecar after copy; spec requires it for global installs.
- **Completed:** false
- **Acceptance Criteria:**
  - Provenance sidecar `~/.sikil/repo/<name>/.sikil-source.toml` is written after successful global install per [provenance.md](provenance.md)
  - Sidecar write failure after content copy is non-fatal (install exits 0 with warning)
  - Partial failure during symlink creation removes all created symlinks, copied skill, and sidecar
- **Tests:**
- **Location:** src/commands/install.rs
- **Notes:** depends on provenance layer task

### Extend adopt command for project scope
- **Spec:** skill-adoption.md
- **Gap:** `adopt` always targets `~/.sikil/repo/` and never updates the manifest; spec requires project-scope adoption to vendor into `.sikil/skills/` and update manifest+lockfile, and global-scope adoption to write a sidecar.
- **Completed:** false
- **Acceptance Criteria:**
  - Adopting a skill in **project scope** moves it from the agent directory to `<project_root>/.sikil/skills/<name>/`
  - Adopting a skill in **global scope** (with `--global` or outside any project) moves it to `~/.sikil/repo/<name>/`
  - A symlink is created at the original location pointing to the moved copy
  - Project-scope adoption creates a **relative** symlink (target like `../../.sikil/skills/<name>`); global-scope adoption creates an **absolute** symlink
  - Project-scope adoption inserts a `[skill.<name>]` entry in `.sikil/manifest.toml` with `source = "adopted:<original-absolute-path>"`
  - Project-scope adoption writes a lockfile entry with `content_hash` of the moved bytes and empty `commit`
  - Global-scope adoption writes a sidecar `~/.sikil/repo/<name>/.sikil-source.toml` per [provenance.md](provenance.md)
  - Manifest/lockfile/sidecar write failure after successful move triggers rollback to restore the original location
- **Tests:**
- **Location:** src/commands/adopt.rs
- **Notes:** depends on manifest and provenance layer tasks

### Extend unmanage command for project scope
- **Spec:** skill-unmanagement.md
- **Gap:** `unmanage` only recognizes the global repo; project-managed symlinks are not detected and the project store is never cleaned.
- **Completed:** false
- **Acceptance Criteria:**
  - Symlinks targeting `<project_root>/.sikil/skills/` are recognized as project-managed and copy from that store
  - Symlinks targeting `~/.sikil/repo/` are recognized as global-managed and copy from that store
  - When all installations of a project-managed skill are unmanaged, the manifest entry, lockfile entry, and vendored bytes are removed
  - When all installations of a global-managed skill are unmanaged, the global repo entry (including its sidecar) is removed
  - Manifest/lockfile write failure during cleanup is non-fatal: warning is emitted, vendored bytes remain in place for recovery
- **Tests:**
- **Location:** src/commands/unmanage.rs
- **Notes:** depends on manifest layer task

### Extend sync command for project scope
- **Spec:** skill-synchronization.md
- **Gap:** `sync` always sources from `~/.sikil/repo/` and creates absolute symlinks at global agent paths; spec requires it to source from `<project_root>/.sikil/skills/` with relative symlinks in project scope.
- **Completed:** false
- **Acceptance Criteria:**
  - In project scope, source is `<project_root>/.sikil/skills/<name>/` and symlinks are relative
  - In global scope, source is `~/.sikil/repo/<name>/` and symlinks are absolute
  - Default `--to` targets all enabled agents (further restricted by manifest `agents` field in project scope)
  - `--project` outside any project root returns `OutsideProject` error
- **Tests:**
- **Location:** src/commands/sync.rs
- **Notes:** depends on manifest layer task and scope-flag task

### Extend remove command for project scope
- **Spec:** skill-removal.md
- **Gap:** `remove --all` deletes only `~/.sikil/repo/<name>/`; spec requires manifest+lockfile entry removal and vendored-bytes cleanup for project-managed skills.
- **Completed:** false
- **Acceptance Criteria:**
  - In project scope, `--all` removes the `[skill.<name>]` manifest entry, the lockfile entry, and the vendored bytes at `<project_root>/.sikil/skills/<name>/`
  - In global scope, `--all` removes `~/.sikil/repo/<name>/` including its `.sikil-source.toml` sidecar
  - Manifest and lockfile rewrites use atomic temp + rename; a crash mid-write leaves the prior file intact
  - Orphaned canonical-store entry after `--agent` removal triggers deletion prompt
  - Orphan cleanup in project scope removes manifest and lockfile entries
- **Tests:**
- **Location:** src/commands/remove.rs
- **Notes:** depends on manifest layer task

---

### Cross-cutting

### Add manifest-driven agent targeting in project scope
- **Spec:** agent-targeting.md
- **Gap:** `parse_agent_selection` does not consult the manifest's per-skill `agents` field; in project scope the manifest must replace the interactive prompt as the targeting default.
- **Completed:** false
- **Acceptance Criteria:**
  - In project scope, the manifest's per-skill `agents` field is used as the targeting default, replacing the interactive prompt
  - `--to` overrides manifest `agents` for a single operation without modifying the manifest
- **Tests:**
- **Location:** src/commands/agent_selection.rs, src/commands/install.rs, src/commands/sync.rs
- **Notes:** depends on manifest layer task
