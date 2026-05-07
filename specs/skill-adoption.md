# Skill Adoption Spec

## One-Sentence Description

Skill adoption moves an unmanaged skill into a managed store and replaces the original with a symlink.

## Overview

The `adopt` command takes an existing unmanaged skill (a physical directory containing SKILL.md) from an agent's skills directory, moves it into a managed store, and creates a symlink at the original location pointing at the moved copy. The destination store depends on scope: in **project scope** (default when inside a project root), the destination is `<project_root>/.sikil/skills/<name>/` and the manifest+lockfile are updated per [project-manifest.md](project-manifest.md); in **global scope** (default outside any project, or with `--global`), the destination is `~/.sikil/repo/<name>/` and a provenance sidecar is written per [provenance.md](provenance.md). `--project` and `--global` flags override scope discovery.

## Adoption Prerequisites

1. **Skill must exist**: The skill must be found by name in at least one agent's skills directory
2. **Skill must be unmanaged**: The source path must not be a symlink (symlinks indicate already-managed skills)
3. **Skill name not in destination store**: For project scope, `<project_root>/.sikil/skills/<name>/` must not already exist; for global scope, `~/.sikil/repo/<name>/` must not already exist
4. **Destination store accessible**: The chosen managed store directory must be writable

The destination store is selected from the resolved scope:

| Scope | Destination | Bookkeeping |
|-------|-------------|-------------|
| Project | `<project_root>/.sikil/skills/<name>/` | Add/update `[skill.<name>]` in `.sikil/manifest.toml` with `source = "adopted:<original-path>"`; write lockfile entry with empty `commit`, `content_hash` of moved bytes, `resolved_url = <original-absolute-path>`, `fetched_at = now` |
| Global | `~/.sikil/repo/<name>/` | Write provenance sidecar `~/.sikil/repo/<name>/.sikil-source.toml` with `source = "adopted:<original-path>"`, empty `commit`, `content_hash` of moved bytes, `resolved_url = <original-absolute-path>`, `installed_at = now` |

## Agent Selection

The `--from` flag specifies which agent's skill installation to adopt:

- **Single installation**: If the skill exists in only one agent, `--from` is optional
- **Multiple installations**: If the skill exists in multiple agents, `--from` is required
- **Format**: Uses agent CLI names (e.g., `claude-code`, `windsurf`, `amp`, `opencode`, `kilocode`)
- **Validation**: Agent name is parsed via `Agent::from_cli_name()` which returns `None` for unknown agents

When multiple locations exist without `--from`, the error message lists all locations:
```
skill '<name>' found in multiple locations. Use --from to specify:
<agent1> (<path1>)
<agent2> (<path2>)
```

## Adoption Process

1. **Resolve scope**: project (if inside a project root and `--global` not set) or global (per [project-manifest.md](project-manifest.md))
2. **Parse `--from` agent** if provided, validate it's a known agent
3. **Scan all agents** using `Scanner::scan_all_agents()` to find skill installations
4. **Filter by name** to find all installations matching the requested skill name
5. **Resolve target installation**:
   - If one installation: use it directly
   - If multiple: require `--from` and find matching agent
6. **Validate unmanaged**: Check `source_path.is_symlink()` returns false
7. **Check destination store available**: Ensure the scope-appropriate destination does not exist
8. **Move to managed store**: Call `atomic_move_dir(source_path, dest_path)` where `dest_path` is the project or global store path
9. **Write bookkeeping**:
   - Project scope: insert/update `[skill.<name>]` in manifest with `source = "adopted:<original-absolute-path>"`; write/update lockfile entry
   - Global scope: write `<dest_path>/.sikil-source.toml` per [provenance.md](provenance.md)
10. **Create symlink**: Call `create_symlink(dest_path, source_path)` (relative target in project scope; absolute in global scope)

## Atomic Operations

### Move Operation (`atomic_move_dir`)

1. **Try atomic rename**: Uses `fs::rename()` which is atomic on same filesystem
2. **Fallback to copy+delete**: For cross-filesystem moves:
   - Backs up destination if it exists (to temp directory)
   - Copies source to destination via `copy_skill_dir()`
   - Removes source directory
   - On failure: restores backup and removes partial copy

### Copy Safety (`copy_skill_dir`)

- Rejects symlinks in source tree (`SymlinkNotAllowed` error)
- Excludes `.git` directory
- Tracks all copied files/directories for rollback
- On failure: removes all copied files in reverse order

### Symlink Creation Rollback

If `create_symlink()` fails after the move succeeds:
1. Removes the destination directory in repository
2. Attempts to move it back (though source no longer exists, so this is best-effort)

```rust
match create_symlink(&dest_path, source_path) {
    Ok(()) => { /* success */ }
    Err(e) => {
        let _ = fs::remove_dir_all(&dest_path);
        let _ = atomic_move_dir(&dest_path, source_path);
        return Err(e.into());
    }
}
```

## Acceptance Criteria

- Adopting a skill in **project scope** moves it from the agent directory to `<project_root>/.sikil/skills/<name>/`
- Adopting a skill in **global scope** (with `--global` or outside any project) moves it to `~/.sikil/repo/<name>/`
- A symlink is created at the original location pointing to the moved copy
- Project-scope adoption creates a **relative** symlink (target like `../../.sikil/skills/<name>`); global-scope adoption creates an **absolute** symlink
- Project-scope adoption inserts a `[skill.<name>]` entry in `.sikil/manifest.toml` with `source = "adopted:<original-absolute-path>"`
- Project-scope adoption writes a lockfile entry with `content_hash` of the moved bytes and empty `commit`
- Global-scope adoption writes a sidecar `~/.sikil/repo/<name>/.sikil-source.toml` per [provenance.md](provenance.md)
- Skill with only one installation can be adopted without `--from` flag
- Skill with multiple installations requires `--from` flag
- Missing `--from` with multiple installations lists all locations in error message
- Adopting a symlink (already managed skill) returns `ValidationError` with "already managed" message
- Skill name already in destination store returns `AlreadyExists` error
- Unknown agent name in `--from` returns `ValidationError`
- Skill not found in specified agent returns `ValidationError`
- Cross-filesystem moves use copy+delete fallback
- Symlink creation failure after move attempts rollback to restore original location
- Manifest/lockfile/sidecar write failure after successful move triggers rollback to restore the original location

## Error Conditions

| Error | Condition | Type |
|-------|-----------|------|
| `SkillNotFound` | No skill with given name exists | `SikilError::SkillNotFound` |
| Invalid agent name | `--from` value not recognized | `SikilError::ValidationError` |
| Multiple locations | Multiple agents have skill, no `--from` | `SikilError::ValidationError` |
| Agent mismatch | `--from` agent doesn't have the skill | `SikilError::ValidationError` |
| Already managed | Source path is a symlink | `SikilError::ValidationError` |
| Already in destination store | `~/.sikil/repo/<name>/` (global) or `<project_root>/.sikil/skills/<name>/` (project) exists | `SikilError::AlreadyExists` |
| Permission denied | Cannot create repo directory | `SikilError::PermissionDenied` |
| Move failure | `atomic_move_dir` fails | Various `SikilError` |
| Symlink failure | `create_symlink` fails | `SikilError::SymlinkError` |

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `core::config::Config` | Access agent configurations |
| `core::scanner::Scanner` | Find skills across agents via `scan_all_agents()` |
| `core::skill::Agent` | Parse agent CLI names |
| `core::errors::SikilError` | Error types |
| `utils::atomic::atomic_move_dir` | Atomic directory move with rollback |
| `utils::symlink::create_symlink` | Create symlink at original location |
| `utils::paths::get_repo_path` | Get `~/.sikil/repo/` path (global scope) |
| `utils::paths::find_project_root` | Discover project root (project scope) |
| `utils::paths::get_project_skills_path` | Get `<project_root>/.sikil/skills/` path (project scope) |
| `utils::paths::ensure_dir_exists` | Create destination store directory |
| Manifest layer | per [project-manifest.md](project-manifest.md) | Manifest+lockfile updates (project scope) |
| Provenance layer | per [provenance.md](provenance.md) | Sidecar write (global scope) |
| `cli::output::Output` | User-facing messages |

## Used By

| Consumer | Location |
|----------|----------|
| CLI main | `src/main.rs` - routes `adopt` subcommand to `execute_adopt` |
| Commands module | `src/commands/mod.rs` - re-exports `execute_adopt` and `AdoptArgs` |
