# Conflict Detection Spec

## One-Sentence Description

Conflict detection identifies duplicate skills and inconsistencies across agent installations.

## Overview

The conflict detection module (`src/core/conflicts.rs`) analyzes `ScanResult` data to identify when multiple installations of the same skill exist across agents. It distinguishes between problematic duplicates (unmanaged physical directories) and acceptable duplicates (managed symlinks pointing to the same repository).

## Conflict Types

| Type | Enum Value | Is Error | Description |
|------|------------|----------|-------------|
| Duplicate Unmanaged | `DuplicateUnmanaged` | Yes | Multiple physical directories with the same skill name across different agent locations. Requires user resolution. |
| Duplicate Managed | `DuplicateManaged` | No | Multiple symlinks pointing to the same managed skill in the repository. This is normal behavior and informational only. |

## Detection Algorithm

The `detect_conflicts()` function iterates through each skill in the `ScanResult`:

1. **Group installations by management type**:
   - An installation is considered "managed" if:
     - `is_symlink == Some(true)` AND
     - `symlink_target` starts with or equals the skill's `repo_path`
   - Otherwise, it's "unmanaged"

2. **Detect DuplicateUnmanaged conflicts**:
   - If there are multiple unmanaged installations (>1)
   - AND they have different paths (checked via HashSet deduplication)
   - Then create a `DuplicateUnmanaged` conflict

3. **Detect DuplicateManaged conflicts**:
   - If there are multiple managed installations (>1)
   - AND all point to the same single repo path
   - Then create a `DuplicateManaged` conflict (informational)

## Conflict Data Structures

### ConflictType

```rust
pub enum ConflictType {
    DuplicateUnmanaged,  // Error - requires resolution
    DuplicateManaged,    // Info - normal behavior
}
```

Methods:
- `description()` → human-readable explanation
- `is_error()` → `true` for `DuplicateUnmanaged`, `false` for `DuplicateManaged`

### Conflict

```rust
pub struct Conflict {
    pub skill_name: String,
    pub locations: Vec<ConflictLocation>,
    pub conflict_type: ConflictType,
}
```

Methods:
- `summary()` → one-line summary (e.g., "skill-name: duplicate unmanaged at 2 location(s)")
- `recommendations()` → resolution suggestions for unmanaged conflicts

### ConflictLocation

```rust
pub struct ConflictLocation {
    pub agent: String,
    pub path: PathBuf,
    pub is_managed: bool,
    pub repo_path: Option<PathBuf>,
}
```

Constructors:
- `from_installation()` → creates from an `Installation` struct
- `new()` → manual construction

## Reporting

### Functions

| Function | Purpose |
|----------|---------|
| `format_conflict()` | Formats a single conflict with status indicator (✗ for error, ℹ for info), locations, and repo paths |
| `format_conflicts_summary()` | Returns summary like "2 errors, 1 info suppressed" or "No conflicts detected". Accepts `verbose: bool` parameter. |
| `filter_error_conflicts()` | Filters to return only error-level conflicts (DuplicateUnmanaged) |
| `filter_displayable_conflicts()` | Filters conflicts for display based on verbose mode. When `verbose: false`, excludes `DuplicateManaged` conflicts. |

### Output Format

```
✗ skill-name (duplicate unmanaged)
  Multiple physical directories with the same skill name...
  Locations:
    1. claude-code (unmanaged) @ /path/to/skill
    2. windsurf (unmanaged) @ /another/path/to/skill
```

## Info Suppression

`DuplicateManaged` conflicts are informational-only (normal behavior for managed skills installed to multiple agents). By default, these are suppressed from human-readable display to reduce noise.

### Scope

- **Human-readable output**: Suppression applies to `sikil list` terminal output
- **JSON output**: All conflicts are included regardless of verbose mode (consumers can filter as needed)
- **`--conflicts` filter**: Shows only error-level conflicts by default; with `-v`, also shows `DuplicateManaged` info

### Behavior

| Verbose Mode | Summary Fragment | Detail Display |
|--------------|------------------|----------------|
| `false` (default) | "N info suppressed" | Not shown |
| `true` (`-v/--verbose`) | "N info" | Shown with full details |

### Summary Format

`format_conflicts_summary(verbose)` returns a fragment for the header line:

| Conflicts Present | verbose=false | verbose=true |
|-------------------|---------------|--------------|
| None | "No conflicts detected" | "No conflicts detected" |
| 2 errors only | "2 errors" | "2 errors" |
| 3 info only | "3 info suppressed" | "3 info" |
| 2 errors + 1 info | "2 errors, 1 info suppressed" | "2 errors, 1 info" |

The list command prepends this to form: `Found N skills (X managed, Y unmanaged) - {summary}`

### Rationale

When a managed skill is installed to multiple agents (e.g., 5 agents), all symlinks point to the same repository location. This is expected behavior, not a problem. Showing detailed info for every such skill creates noise without actionable information.

### Example Output

**Default (verbose=false):**
```
Found 2 skills (1 managed, 1 unmanaged) - 1 info suppressed
```

**Verbose (verbose=true):**
```
Found 2 skills (1 managed, 1 unmanaged) - 1 info

ℹ context7 (duplicate managed)
  Multiple symlinks pointing to the same managed skill...
  Locations:
    1. windsurf (managed) @ /home/user/.custom/windsurf/skills/context7
       → repo: /home/user/.sikil/repo/context7
    ...
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `crate::core::scanner::ScanResult` | Input data containing all scanned skills and installations |
| `crate::core::skill::Installation` | Installation metadata (agent, path, symlink info) |
| `serde::{Serialize, Deserialize}` | JSON output support |
| `std::path::PathBuf` | Path handling |

## Used By

| Consumer | Usage |
|----------|-------|
| `src/commands/list.rs` | Detects conflicts and filters skills with `--conflicts` flag |
| `src/main.rs` | CLI flag passthrough for conflicts filtering |

## Acceptance Criteria

### Conflict Detection
- Multiple unmanaged installations with different paths create a `DuplicateUnmanaged` conflict
- Multiple managed symlinks pointing to the same repo path create a `DuplicateManaged` conflict
- `DuplicateUnmanaged` conflicts have `is_error()` returning `true`
- `DuplicateManaged` conflicts have `is_error()` returning `false`
- Installation is classified as managed only when `is_symlink == Some(true)` AND `symlink_target` starts with repo path
- Two managed installs are considered duplicates when their resolved `repo_path` is identical

### Filtering
- `filter_error_conflicts()` returns only `DuplicateUnmanaged` conflicts
- `filter_displayable_conflicts()` with `verbose: false` excludes `DuplicateManaged` conflicts
- `filter_displayable_conflicts()` with `verbose: true` includes all conflicts

### Summary Formatting
- `format_conflicts_summary(verbose: false)` shows "N info suppressed" for info-only conflicts
- `format_conflicts_summary(verbose: true)` shows "N info" for info-only conflicts
- `format_conflicts_summary()` returns "No conflicts detected" when no conflicts exist
- `format_conflicts_summary()` with mixed conflicts shows "N errors, M info suppressed" when not verbose
- `format_conflicts_summary()` with mixed conflicts shows "N errors, M info" when verbose

### Display Output
- Conflict output shows `✗` indicator for error conflicts and `ℹ` for info conflicts
- `recommendations()` returns resolution suggestions for unmanaged conflicts
- `DuplicateManaged` conflict details are not printed in human-readable output unless verbose mode is enabled
- JSON output (`--json`) includes all conflicts regardless of verbose mode
- `--conflicts` filter shows only error-level conflicts by default
- `--conflicts -v` filter shows both error and info conflicts
