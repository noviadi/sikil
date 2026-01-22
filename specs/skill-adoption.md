# Skill Adoption Spec

## One-Sentence Description

Skill adoption moves unmanaged skills into the managed repository and replaces them with symlinks.

## Overview

The `adopt` command takes an existing unmanaged skill (a physical directory containing SKILL.md) from an agent's skills directory, moves it to the central Sikil repository (`~/.sikil/repo/<name>/`), and creates a symlink at the original location pointing to the repository copy. This converts an unmanaged skill into a managed skill without losing any files.

## Adoption Prerequisites

1. **Skill must exist**: The skill must be found by name in at least one agent's skills directory
2. **Skill must be unmanaged**: The source path must not be a symlink (symlinks indicate already-managed skills)
3. **Skill name not in repository**: `~/.sikil/repo/<name>/` must not already exist
4. **Repository directory accessible**: `~/.sikil/repo/` must be writable

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

1. **Parse `--from` agent** if provided, validate it's a known agent
2. **Scan all agents** using `Scanner::scan_all_agents()` to find skill installations
3. **Filter by name** to find all installations matching the requested skill name
4. **Resolve target installation**:
   - If one installation: use it directly
   - If multiple: require `--from` and find matching agent
5. **Validate unmanaged**: Check `source_path.is_symlink()` returns false
6. **Check repository available**: Ensure `~/.sikil/repo/<name>/` does not exist
7. **Move to repository**: Call `atomic_move_dir(source_path, dest_path)`
8. **Create symlink**: Call `create_symlink(dest_path, source_path)`

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

## Error Conditions

| Error | Condition | Type |
|-------|-----------|------|
| `SkillNotFound` | No skill with given name exists | `SikilError::SkillNotFound` |
| Invalid agent name | `--from` value not recognized | `SikilError::ValidationError` |
| Multiple locations | Multiple agents have skill, no `--from` | `SikilError::ValidationError` |
| Agent mismatch | `--from` agent doesn't have the skill | `SikilError::ValidationError` |
| Already managed | Source path is a symlink | `SikilError::ValidationError` |
| Already in repo | `~/.sikil/repo/<name>/` exists | `SikilError::AlreadyExists` |
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
| `utils::paths::get_repo_path` | Get `~/.sikil/repo/` path |
| `utils::paths::ensure_dir_exists` | Create repository directory |
| `cli::output::Output` | User-facing messages |

## Used By

| Consumer | Location |
|----------|----------|
| CLI main | `src/main.rs` - routes `adopt` subcommand to `execute_adopt` |
| Commands module | `src/commands/mod.rs` - re-exports `execute_adopt` and `AdoptArgs` |
