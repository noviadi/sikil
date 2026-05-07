# Skill Scanner Spec

## One-Sentence Description

The skill scanner discovers installed skills across agent directories.

## Overview

The `Scanner` struct in [`src/core/scanner.rs`](file:///home/noviadi/Developments/code/rust-playground/sikil/src/core/scanner.rs) provides the skill discovery mechanism. It uses agent configurations from `Config` to determine which directories to scan, parses SKILL.md files to extract metadata, and aggregates results into a `ScanResult` that groups skills by name with their installations.

## Scanning Algorithm

1. Create a `ScanResult` to accumulate discoveries
2. Resolve `project_root` via `find_project_root(cwd)` (per [filesystem-paths.md](filesystem-paths.md)); may be `None`
3. Iterate through all enabled agents in the config
4. For each agent, scan its global path (if exists)
5. For each agent, scan its workspace path relative to `project_root` (if `project_root` is set), otherwise relative to cwd (if exists)
6. If `project_root` is set, scan the project-managed store at `<project_root>/.sikil/skills/`
7. Scan the global managed skills repository (`~/.sikil/repo/`)
8. Merge duplicate skill names, aggregating their installations

## Directory Traversal

Directories are scanned in this order:

1. **Global paths** for each enabled agent (e.g., `~/.claude/skills`, `~/.codeium/windsurf/skills`)
2. **Workspace paths** for each enabled agent, anchored at `project_root` if discovered or cwd otherwise (e.g., `<project_root>/.claude/skills`)
3. **Project-managed store** at `<project_root>/.sikil/skills/` (only when `project_root` is set)
4. **Global managed repository** at `~/.sikil/repo/`

Within each directory, the scanner:
- Reads all directory entries via `fs::read_dir`
- Skips entries that are not directories or symlinks
- Skips hidden directories (names starting with `.`)
- Does not recurse into subdirectories (single-level scan)

## Skill Detection

A valid skill is identified when:

1. The entry is a directory or symlink
2. The directory name does not start with `.`
3. A `SKILL.md` file exists inside the directory
4. The `SKILL.md` contains valid YAML frontmatter between `---` markers
5. The frontmatter contains required fields: `name` and `description`

The parser (`parse_skill_md`) extracts:
- `name`: Skill identifier (validated against naming rules)
- `description`: Human-readable description
- Optional fields: `version`, `author`, `tags`

## Symlink Handling

For each directory entry, the scanner:

1. Checks if the entry is a symlink via `file_type.is_symlink()`
2. If symlink, reads the target with `read_symlink_target()`
3. Classifies the installation type:
   - **Managed**: Symlink target resolves to a path under `~/.sikil/repo/` **or** under any `<project_root>/.sikil/skills/` reachable from the symlink's location (per [symlink-operations.md](symlink-operations.md))
   - **Unmanaged**: Physical directory (not a symlink)
   - **BrokenSymlink**: Symlink target does not exist
   - **ForeignSymlink**: Symlink points outside both managed stores

Classification uses `resolve_realpath()` to canonicalize the symlink target before checking the managed-store prefixes. Project-managed and global-managed installations are both `Managed` for conflict-detection purposes but track distinct `repo_path` values (the canonical store under which the symlink target lives).

## Error Handling

- **Non-existent directories**: Silently skipped; no error returned
- **Unreadable directory entries**: Skipped with `continue`
- **Missing SKILL.md**: Error recorded in `result.parse_errors` but scanning continues
- **Invalid SKILL.md content**: Error recorded in `result.parse_errors` but scanning continues
- **Symlink resolution failures**: Entry treated as `BrokenSymlink`

All errors are non-fatal; the scanner processes all accessible paths and returns a complete `ScanResult` with both discovered skills and accumulated errors.

## Acceptance Criteria

- Directories starting with `.` are skipped during scan
- Symlinks pointing to `~/.sikil/repo/` are classified as managed
- Symlinks pointing outside `~/.sikil/repo/` are classified as foreign symlinks
- Non-existent symlink targets are classified as broken symlinks
- Missing SKILL.md records error in `parse_errors` but continues scanning
- Invalid SKILL.md content records error in `parse_errors` but continues scanning
- Non-existent agent directories are silently skipped (no error returned)
- Unreadable directory entries are skipped without stopping the scan
- Scan iterates global paths before workspace paths for each agent
- Duplicate skill names across directories are merged into a single skill with multiple installations
- Physical directories (not symlinks) are classified as unmanaged
- When invoked inside a project root, workspace paths are anchored at the project root rather than cwd
- When invoked inside a project root, the project-managed store at `<project_root>/.sikil/skills/` is included in the scan
- Symlinks pointing into `<project_root>/.sikil/skills/` are classified as managed (project-managed)
- A skill present in both the project-managed store and the global repo appears as a single skill with installations from both scopes

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `Config` | Provides agent paths (global/workspace) and enabled status |
| `parse_skill_md` | Parses SKILL.md frontmatter into `SkillMetadata` |
| `Cache` | Optional JSON file cache for scan results (SKILL.md mtime invalidation) |
| `symlink::is_symlink` | Detects symlinks without following them |
| `symlink::read_symlink_target` | Reads raw symlink target path |
| `symlink::resolve_realpath` | Canonicalizes symlink to absolute path |
| `paths::get_repo_path` | Returns `~/.sikil/repo/` path |
| `paths::find_project_root` | Discovers project root for workspace anchoring and project-managed store inclusion |
| `paths::get_project_skills_path` | Returns `<project_root>/.sikil/skills/` path |

## Used By

| Consumer | Usage |
|----------|-------|
| `commands::list::execute_list` | Scans all agents to display installed skills |
| `conflicts::detect_conflicts` | Analyzes `ScanResult` for duplicate/conflicting installations |
| Install/uninstall commands | Query existing installations before modifying |
