# Skill Installation Spec

## One-Sentence Description

Skill installation copies skills from local paths or Git repositories into a managed store and creates symlinks from agent skill directories to that store.

## Overview

The install command operates in two scopes — **project** and **global** — and supports two source kinds: local directories and Git URLs. Scope is determined per [project-manifest.md](project-manifest.md): inside a project root, project scope is the default; outside, global is the default; `--global`/`--project` flags override discovery. In project scope, the canonical store is `<project_root>/.sikil/skills/<name>/` and the manifest+lockfile track declared and resolved state. In global scope, the canonical store is `~/.sikil/repo/<name>/` and a per-skill provenance sidecar records origin per [provenance.md](provenance.md). Both flows validate the source skill (requiring a valid SKILL.md), copy bytes via `copy_skill_dir` (rejecting symlinks), and create symlinks at one or more agent skill directories. Rollback is performed on partial failure.

## Installation Scopes

| Scope | Canonical store | Provenance | Triggered when |
|-------|----------------|------------|----------------|
| **Project** | `<project_root>/.sikil/skills/<name>/` | Lockfile entry in `<project_root>/.sikil/lock.toml` | Inside a project root, or `--project` passed |
| **Global** | `~/.sikil/repo/<name>/` | Sidecar at `<repo>/<name>/.sikil-source.toml` | Outside any project, or `--global` passed |

`sikil install` (no positional source argument) is valid only in project scope; it performs manifest reconciliation per [project-manifest.md](project-manifest.md). Global scope requires a positional source.

## Installation Sources

1. **Local path** (`execute_install_local`)
   - Absolute or relative filesystem paths
   - Relative paths resolved against current working directory (global) or project root (project)
   - Must be an existing directory containing SKILL.md

2. **Git URL** (`execute_install_git`)
   - Short form: `owner/repo` or `owner/repo/path/to/skill`
   - HTTPS URL: `https://github.com/owner/repo.git`
   - HTTPS URL without .git suffix: `https://github.com/owner/repo`

## Git URL Parsing

Implemented in `src/utils/git.rs`:

| Format | Example | Expansion |
|--------|---------|-----------|
| Short form | `owner/repo` | `https://github.com/owner/repo.git` |
| Short with subdir | `owner/repo/skills/my-skill` | Clone repo, extract `skills/my-skill` |
| HTTPS | `https://github.com/owner/repo.git` | Used directly |
| HTTPS without .git | `https://github.com/owner/repo` | Used directly |

**Security validations:**
- Only GitHub.com URLs allowed
- `file://` protocol rejected
- URLs starting with `-` rejected (argument injection protection)
- URLs with whitespace or NUL characters rejected
- Subdirectory paths with `..` rejected (path traversal protection)

## Installation Process

### Local Installation Flow (global scope)

1. Parse source path (resolve relative to cwd if needed)
2. Validate source exists and is a directory
3. Validate SKILL.md exists and parse metadata to extract skill name
4. Determine target agents from `--to` flag or interactive prompt
5. Check `~/.sikil/repo/<name>` does not already exist
6. Check each agent skill directory for existing skill (dir or symlink)
7. Copy skill to `~/.sikil/repo/<name>/` using `copy_skill_dir` (rejects symlinks)
8. Create absolute symlinks from each agent's global skill directory to the repo copy
9. Write provenance sidecar at `~/.sikil/repo/<name>/.sikil-source.toml` per [provenance.md](provenance.md); sidecar write failure is non-fatal (warning only)
10. On failure during symlink creation, rollback: remove created symlinks and copied skill (sidecar is removed if present)

### Git Installation Flow (global scope)

1. Parse Git URL to extract clone_url, owner, repo, and optional subdirectory
2. Clone repository to temp directory with `--depth=1` (shallow clone)
3. If subdirectory specified, extract it to separate temp location
4. Validate SKILL.md exists and parse metadata
5. Determine target agents
6. Check repo and agent destinations for conflicts
7. Clean up clone (remove `.git/` directory)
8. Copy skill to `~/.sikil/repo/<name>/` using `copy_skill_dir`
9. Create absolute symlinks to agents' global skill directories
10. Write provenance sidecar at `~/.sikil/repo/<name>/.sikil-source.toml` recording the original `source` argument, `resolved_url`, `commit`, and `content_hash` per [provenance.md](provenance.md)
11. On failure, rollback (remove symlinks, copied skill, sidecar) and clean up temp directories

### Project Installation Flow

`sikil install <source> --project` (or default behavior inside a project) performs:

1. Resolve `project_root` (per [filesystem-paths.md](filesystem-paths.md))
2. Parse and validate the source as in local/git flows above
3. Extract skill name from SKILL.md
4. Add or update `[skill.<name>]` entry in `<project_root>/.sikil/manifest.toml`:
   - `source` = the original argument verbatim
   - `rev` = `--rev` if provided (git sources only); otherwise omitted (default `"main"`)
   - `subdir` = `--subdir` if provided; otherwise omitted
   - `agents` = `--to` if provided; otherwise omitted (default = all enabled)
5. Reconcile the manifest per [project-manifest.md](project-manifest.md):
   - Fetch / re-use source per cache rules
   - Vendor to `<project_root>/.sikil/skills/<name>/`
   - Compute `content_hash` and update `<project_root>/.sikil/lock.toml`
   - Create relative symlinks at each target agent's `workspace_path` (e.g., `.claude/skills/<name>` → `../../.sikil/skills/<name>`)
6. On failure: revert manifest edit, remove vendored bytes, remove created symlinks; lockfile is restored from its prior state via atomic write semantics

`sikil install` with no positional source argument inside a project loads the manifest and lockfile, then performs reconciliation only (no manifest edit). Outside a project, this form returns `OutsideProject`.

## Agent Targeting

The `--to` flag determines which agents receive symlinks:

| Value | Behavior |
|-------|----------|
| Not specified | Interactive prompt (or all enabled in JSON mode) |
| `all` | All enabled agents from config |
| `claude-code` | Single agent by name |
| `claude-code,amp` | Comma-separated list of agents |

Implementation: `parse_agent_selection()` and `prompt_agent_selection()` in `src/commands/mod.rs`.

## Symlink Creation

Implemented in `src/utils/symlink.rs`:

1. Ensure parent directory exists (create if missing)
2. Remove existing symlink at destination if present
3. Create Unix symlink: `std::os::unix::fs::symlink(src, dest)`

**Symlink direction and form:**
- **Global scope:** `<global_agent_path>/<name>` → `<absolute>~/.sikil/repo/<name>/`
- **Project scope:** `<project_root>/<workspace_path>/<name>` → `<relative>../../.sikil/skills/<name>` (relative target enables portability across `git clone` to different absolute paths per [project-manifest.md](project-manifest.md))

**Helper functions:**
- `is_symlink(path)` - Check if path is a symlink
- `read_symlink_target(path)` - Read symlink target
- `resolve_realpath(path)` - Canonicalize path following symlinks
- `is_managed_symlink(path)` - Check if symlink points to `~/.sikil/repo/`

## Error Conditions

| Condition | Error Type | Message/Behavior |
|-----------|------------|------------------|
| Source path not found | `DirectoryNotFound` | Path does not exist |
| Source not a directory | `ValidationError` | "source path is not a directory" |
| Missing SKILL.md | `InvalidSkillMd` | "SKILL.md not found" |
| Invalid SKILL.md content | `InvalidSkillMd` | Parsing failure details |
| No agents selected | `ValidationError` | "no agents selected" |
| Skill already in repo | `AlreadyExists` | "skill '<name>' in repository" |
| Destination is physical dir | `AlreadyExists` | Suggests `sikil adopt` |
| Destination is symlink | `AlreadyExists` | Suggests `sikil sync` |
| Source contains symlinks | `SymlinkNotAllowed` | Symlinks not permitted in skills |
| Git not installed | `GitError` | "git is not installed" |
| Clone failure | `GitError` | stderr from git |
| Subdirectory not found | `DirectoryNotFound` | Path within clone not found |
| Path traversal attempt | `PathTraversal` | Subdirectory contains `..` |
| Permission denied | `PermissionDenied` | Cannot create directories |

**Rollback behavior:** On partial failure during symlink creation, all created symlinks are removed and the copied skill directory is deleted. In project scope, the manifest edit is reverted and the lockfile is restored to its prior on-disk state (atomic temp + rename ensures a crash leaves the previous lockfile intact).

**Additional project-scope errors:**

| Condition | Error Type | Message |
|-----------|------------|---------|
| `sikil install` (no source) outside any project | `OutsideProject` | "no project found; run `sikil init` or use `--global`" |
| Manifest entry name conflict (different `source` for same skill name) | `AlreadyExists` | "skill '<name>' already declared with different source" |
| Lockfile version unsupported | `LockfileMismatch` | per [project-manifest.md](project-manifest.md) |
| Manifest write failure | `PermissionDenied` | "cannot write `<path>`" |

## Acceptance Criteria

### Global scope

- Installing from local path with `--global` copies skill to `~/.sikil/repo/<name>/`
- Installing from Git URL with `--global` clones with `--depth=1` and copies skill to `~/.sikil/repo/<name>/`
- Symlinks are created from each target agent's global skill directory to the repo copy
- Provenance sidecar `~/.sikil/repo/<name>/.sikil-source.toml` is written after successful global install per [provenance.md](provenance.md)
- Sidecar write failure after content copy is non-fatal (install exits 0 with warning)
- Short-form Git URL `owner/repo` expands to `https://github.com/owner/repo.git`
- Git URL with subdirectory `owner/repo/path/to/skill` extracts only that subdirectory
- Source containing symlinks returns `SymlinkNotAllowed` error
- Source path not found returns `DirectoryNotFound` error
- Missing SKILL.md returns `InvalidSkillMd` error with "SKILL.md not found"
- Skill already in repo returns `AlreadyExists` error with message "skill '<name>' in repository"
- Destination already exists as directory returns `AlreadyExists` error suggesting `sikil adopt`
- Destination already exists as symlink returns `AlreadyExists` error suggesting `sikil sync`
- Git URL with `file://` protocol is rejected
- Git URL starting with `-` is rejected
- Subdirectory path containing `..` returns `PathTraversal` error
- Partial failure during symlink creation removes all created symlinks, copied skill, and sidecar
- `.git/` directory is removed from copied skills

### Project scope

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

## Dependencies

| Component | Location | Purpose |
|-----------|----------|---------|
| `Config` | `src/core/config.rs` | Agent configuration and paths |
| `parse_skill_md` | `src/core/parser.rs` | Extract skill name from SKILL.md |
| `copy_skill_dir` | `src/utils/atomic.rs` | Atomic copy rejecting symlinks |
| `atomic_write_file` | `src/utils/atomic.rs` | Atomic manifest, lockfile, and sidecar writes |
| `create_symlink` | `src/utils/symlink.rs` | Unix symlink creation |
| `parse_git_url` | `src/utils/git.rs` | Git URL parsing |
| `clone_repo` | `src/utils/git.rs` | Git clone execution |
| `extract_subdirectory` | `src/utils/git.rs` | Subdirectory extraction |
| `cleanup_clone` | `src/utils/git.rs` | Remove .git directory |
| `get_repo_path` | `src/utils/paths.rs` | Returns `~/.sikil/repo/` |
| `find_project_root` | `src/utils/paths.rs` | Project root discovery (project scope) |
| `get_manifest_path`, `get_lock_path`, `get_project_skills_path` | `src/utils/paths.rs` | Project file locations |
| `ensure_dir_exists` | `src/utils/paths.rs` | Create directories |
| Manifest layer | per [project-manifest.md](project-manifest.md) | Manifest+lockfile read/write/reconcile |
| Provenance layer | per [provenance.md](provenance.md) | Global sidecar write |

## Used By

| Command | Usage |
|---------|-------|
| `sikil install <path>` | Install from local directory (project scope by default in repo, global outside) |
| `sikil install <git-url>` | Install from Git repository (project scope by default in repo, global outside) |
| `sikil install` (no args) | Reconcile project manifest + lockfile (project only) |
| `sikil install --global` | Force global scope regardless of cwd |
| `sikil install --project` | Force project scope; error if not in project |
| `sikil add <source>` | Project-only sugar that delegates here |
