# Project Manifest Spec

## One-Sentence Description

The project manifest tracks a project's skill dependencies through a paired manifest and lockfile.

## Overview

The project manifest is Sikil's mechanism for binding a set of skills to a project repository: which skills the project requires, where each skill comes from, and what was actually installed. It uses two TOML files at `<project>/.sikil/manifest.toml` (human-authored declared intent) and `<project>/.sikil/lock.toml` (machine-written resolved state), modeled on `Cargo.toml` + `Cargo.lock`. The manifest enables per-project skill scoping, vendored skills, and reproducible installs across machines and teammates.

## Project Root Discovery

A "project root" is the directory containing `.sikil/manifest.toml` or, in its absence, the directory containing `.git/`.

Discovery walks upward from the current working directory:

1. If `<dir>/.sikil/manifest.toml` exists → `project_root = <dir>`. Stop.
2. Else if `<dir>/.git` exists (file or directory) → `project_root = <dir>`. Stop.
3. Else → continue walking upward to the filesystem root.
4. If no marker found → no project root; the operation runs in **global scope**.

The first marker found wins. Submodules (with their own `.git` file) become their own project roots.

The `--global` flag bypasses discovery entirely (always global scope). The `--project` flag requires a project root to exist (errors with `OutsideProject` if not found).

## Filesystem Layout

```
<project_root>/
├── .sikil/
│   ├── manifest.toml        # declared intent (human-authored)
│   ├── lock.toml            # resolved state (machine-written)
│   └── skills/
│       └── <skill-name>/    # vendored canonical bytes
│           ├── SKILL.md
│           ├── scripts/     # optional
│           └── references/  # optional
├── .claude/skills/<skill-name>     # → ../../.sikil/skills/<skill-name>
├── .opencode/skill/<skill-name>    # → ../../.sikil/skills/<skill-name>
├── .agents/skills/<skill-name>     # → ../../.sikil/skills/<skill-name>
└── .git/
```

Agent symlinks under `.<agent>/skills/<name>` are **relative** so the project remains portable across clones to different absolute paths. The agent-side directory names come from each agent's `workspace_path` in `~/.sikil/config.toml`.

## Manifest Schema (`.sikil/manifest.toml`)

```toml
schema_version = 1

[skill.<skill-name>]
source = "<source>"           # required
rev = "<ref>"                 # optional; defaults to "main"
subdir = "<path>"             # optional; subdirectory inside source
agents = ["<cli-name>", ...]  # optional; defaults to all enabled agents
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema_version` | integer | Yes | Currently `1`; reserved for future migrations |
| `[skill.<name>]` | table key | — | Skill name; matches the SKILL.md `name` field; validated against `^[a-z0-9][a-z0-9_-]{0,63}$` |
| `source` | string | Yes | GitHub short form (`owner/repo`), HTTPS URL, or relative local path |
| `rev` | string | No | Branch, tag, or commit SHA; defaults to `"main"` for git sources; ignored for local sources |
| `subdir` | string | No | Path inside the source repo to the skill directory; defaults to repo root |
| `agents` | array of strings | No | Agent CLI names to materialize symlinks for; defaults to all enabled agents in config |

### Source Forms

| Form | Example | Resolution |
|------|---------|------------|
| GitHub short | `vercel-labs/agent-skills` | `https://github.com/vercel-labs/agent-skills.git` |
| GitHub HTTPS | `https://github.com/owner/repo.git` | Used as-is |
| GitHub HTTPS (no `.git`) | `https://github.com/owner/repo` | Used as-is |
| Local path (relative) | `../shared-skills/foo` | Resolved relative to project root |

SSH URLs, non-GitHub hosts, `file://` URLs, URLs starting with `-`, and URLs containing whitespace or NUL are rejected per [git-operations.md](git-operations.md).

### Multi-Skill Sources

A repo containing multiple skills declares one entry per skill, each with the same `source` and a different `subdir`:

```toml
[skill.frontend-design]
source = "vercel-labs/agent-skills"
subdir = "skills/frontend-design"

[skill.skill-creator]
source = "vercel-labs/agent-skills"
subdir = "skills/skill-creator"
```

Sikil deduplicates the clone internally; the source is fetched once per `install` operation.

### Comments and Ordering

Sikil preserves manifest comments and entry ordering on a best-effort basis using `toml_edit`. Round-trip preservation is not guaranteed for every edit; manifest content authored by humans (comments, blank lines, custom ordering) takes priority over machine-written ordering when conflict-free.

## Lockfile Schema (`.sikil/lock.toml`)

```toml
version = 1

[skill.<skill-name>]
source = "<source-as-declared>"
resolved_url = "<expanded-url-or-absolute-path>"
commit = "<full-sha>"
content_hash = "sha256:<hex>"
fetched_at = "<rfc3339-timestamp>"
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `version` | integer | Lockfile format version; currently `1` |
| `[skill.<name>]` | table key | Mirrors manifest entry name |
| `source` | string | Verbatim copy of manifest `source` at resolution time |
| `resolved_url` | string | Fully expanded clone URL or absolute local path |
| `commit` | string | Full git commit SHA; empty string for local sources |
| `content_hash` | string | SHA-256 of the vendored skill tree (excluding `.git/`); prefixed `sha256:` |
| `fetched_at` | string | RFC 3339 UTC timestamp of the resolution |

The lockfile contains exactly one entry per manifest entry. Stale lockfile entries (no matching manifest entry) are removed by `sikil install`.

## Install Behavior

`sikil install` (no positional source argument), when `project_root` is set, performs **manifest reconciliation**:

1. Load manifest. If absent, error `ManifestNotFound`.
2. Load lockfile. Missing lockfile is treated as empty.
3. For each manifest entry:
   - **Cache hit:** lockfile entry exists, `rev` matches the lockfile resolution, vendored bytes exist, content hash matches → reuse.
   - **Resolve required:** otherwise → fetch source, vendor to `.sikil/skills/<name>/`, compute new `commit` and `content_hash`, update lockfile entry.
4. Remove vendored directories and lockfile entries whose names are absent from the manifest (with confirmation unless `--yes`).
5. For each manifest entry, ensure agent symlinks exist for the entry's `agents` set (defaulting to all enabled agents). Remove agent symlinks pointing at orphaned entries.
6. Atomically write the lockfile.

The operation is **idempotent**: running `sikil install` twice with no manifest changes produces no filesystem changes (other than possibly re-creating missing symlinks). It is **self-healing**: missing vendored bytes or symlinks are restored.

`sikil install <source>`, when `project_root` is set, additionally:
- Validates the source produces a parseable SKILL.md
- Inserts (or updates) a `[skill.<name>]` entry in the manifest
- Then performs manifest reconciliation as above

`sikil install <source> --global` bypasses the project entirely; behavior is unchanged from the existing global-only flow defined in [skill-installation.md](skill-installation.md).

## Update Behavior

`sikil update [<name>...]` re-resolves manifest entries:

1. For each named entry (or all entries if no names given):
   - Re-fetch the source at the manifest's declared `rev` (which may be a moving ref like `main`)
   - Compute new `commit` and `content_hash`
   - If unchanged → report "up to date"
   - If changed → vendor the new bytes (with rollback on failure), update lockfile entry, print one-line diff summary (`<name>: <old-commit> → <new-commit> (+N/-M lines)`)
2. Prompt `[y/N]` once before applying any changes (skipped under `--yes` or `--json`).
3. `--dry-run` prints the diff summary without writing anything.

Update never modifies the manifest's `rev` field. To pin to a different ref, the user edits the manifest.

## Add and Remove Behavior

`sikil add <source>` is sugar for "edit manifest then install"; errors with `OutsideProject` if no project root is found. Equivalent to `sikil install <source> --project`.

`sikil remove <name>` in project context:
1. Removes the manifest entry
2. Removes the lockfile entry
3. Removes the vendored directory at `.sikil/skills/<name>/`
4. Removes agent symlinks at `<workspace_path>/<name>` for the entry's `agents` set
5. Confirmation per [skill-removal.md](skill-removal.md)

## Init Behavior

`sikil init` creates `.sikil/manifest.toml` with a single `schema_version = 1` line at:
- The current working directory if `--here` is passed, or
- The git root if invoked inside a git repo, or
- The current working directory otherwise

If a manifest already exists at the chosen location, `sikil init` is a no-op and reports the existing path. `sikil init` is optional; the first `sikil install <source>` (without `--global`) creates the manifest implicitly.

## Atomicity and Concurrency

- Manifest writes use atomic temp-file + rename via [atomic-operations.md](atomic-operations.md).
- Lockfile writes use atomic temp-file + rename.
- Vendored skill writes use `copy_skill_dir` with reverse-order rollback per [atomic-operations.md](atomic-operations.md).
- No file locking. Concurrent `sikil install` invocations on the same project are not supported in v0.2 (last writer wins); documented as a known limitation.

## Gitignore Modes

The user controls what is committed via `.gitignore`. Two supported patterns:

**Vendored mode (default, recommended):**
```gitignore
# nothing — commit .sikil/manifest.toml, .sikil/lock.toml, .sikil/skills/, and .<agent>/skills/<name> symlinks
```
Clone-and-go: a fresh `git clone` immediately gives the agents access to the skills with no `sikil install` required.

**Lockfile-only mode:**
```gitignore
.sikil/skills/
.claude/skills/
.opencode/skill/
.agents/skills/
# (and other .<agent>/skills/ paths)
```
Clone-and-install: `sikil install` re-vendors from manifest+lock on first use. Network required.

Sikil writes the same files in both modes; gitignore is a git concern.

## Acceptance Criteria

- Project root discovery walks up from cwd, finding `.sikil/manifest.toml` before `.git/`
- A `.git` file (worktree marker) is treated equivalently to a `.git/` directory for root discovery
- `--global` flag forces global scope regardless of project root presence
- `--project` flag in absence of project root returns `OutsideProject` error
- `sikil init` creates `.sikil/manifest.toml` containing `schema_version = 1` at the git root
- `sikil init` is a no-op (with informational message) when manifest already exists
- `sikil install` (no args) inside a project loads manifest, fetches missing sources, vendors them to `.sikil/skills/<name>/`, and creates relative agent symlinks
- `sikil install` (no args) without a manifest returns `ManifestNotFound`
- `sikil install <source>` inside a project adds an entry to the manifest before reconciling
- `sikil install <source> --global` writes to `~/.sikil/repo/` with no manifest interaction
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
- `sikil update <name>` re-resolves only the named entry, leaving others untouched
- `sikil update` with no args re-resolves every manifest entry
- `sikil update --dry-run` prints diff summary without writing manifest, lockfile, or vendored bytes
- `sikil update` displays one-line summary `<name>: <old-commit> → <new-commit>` per changed skill
- `sikil add <source>` outside a project returns `OutsideProject`
- `sikil remove <name>` in project mode removes manifest entry, lockfile entry, vendored bytes, and agent symlinks
- Manifest comments and entry ordering are preserved across `sikil add` on a best-effort basis
- Agent symlinks committed to git remain valid after `git clone` to a different absolute path (because they are relative)
- Manifest writes are atomic (temp file + rename); a crash mid-write leaves the previous manifest intact
- Lockfile writes are atomic (temp file + rename)

## Error Conditions

| Condition | Error Type | Message |
|-----------|------------|---------|
| No manifest found, `--project` required | `OutsideProject` | "no project found; run `sikil init` or use `--global`" |
| `sikil install` (no args) without manifest | `ManifestNotFound` | "no manifest at `<path>/.sikil/manifest.toml`" |
| Manifest TOML invalid or unknown fields | `ManifestParseError` | "invalid manifest: `<reason>`" |
| Manifest size exceeds 1 MB | `ManifestParseError` | "manifest exceeds 1 MB" |
| Lockfile present but `version` unsupported | `LockfileMismatch` | "lockfile version `<n>` is not supported (expected 1)" |
| Source URL invalid (per git-operations) | `InvalidGitUrl` | passed through |
| Local-path source missing | `SourceUnreachable` | "source path `<path>` does not exist" |
| Network failure during clone | `GitError` | passed through; exit code 5 |
| Skill name in manifest fails name validation | `ValidationError` | per skill-model |
| Concurrent install detected | (none) | unsupported in v0.2; last writer wins |

## Dependencies

| Component | Purpose |
|-----------|---------|
| `core::config::Config` | Enabled-agents list and per-agent workspace paths |
| `core::parser::parse_skill_md` | Extracts skill name from fetched source |
| `utils::paths` | Project root discovery, `get_project_root`, `get_manifest_path`, `get_lock_path`, `get_project_skills_path` |
| `utils::atomic::copy_skill_dir` | Vendor skill bytes with rollback |
| `utils::atomic::atomic_write_file` | Atomic manifest and lockfile writes |
| `utils::symlink::create_symlink` | Create relative agent symlinks |
| `utils::git::parse_git_url` | Validate and parse manifest sources |
| `utils::git::clone_repo` | Fetch git sources |
| `utils::git::extract_subdirectory` | Extract `subdir` from clone |
| `toml` | Parse manifest and lockfile |
| `toml_edit` | Preserve comments and ordering on manifest writes |
| `sha2` | Compute `content_hash` |
| `chrono` or equivalent | Format `fetched_at` as RFC 3339 |

## Used By

| Consumer | Usage |
|----------|-------|
| `commands::install` | Reconciles manifest on no-args invocation; updates manifest on source invocation |
| `commands::add` | Inserts manifest entry; delegates to install for materialization |
| `commands::update` | Re-resolves manifest entries |
| `commands::remove` | Removes manifest and lockfile entries when in project scope |
| `commands::init` | Creates the manifest file |
| `commands::list` | Lists project-managed skills alongside global-managed |
| `commands::show` | Shows manifest source and resolved lockfile state for a skill |
| `core::scanner` | Recognizes `.sikil/skills/` as a managed-store under the project root |
