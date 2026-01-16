# Sikil - Technical Requirements Document

**Version**: 1.0  
**Created**: January 16, 2026  
**Status**: Draft  
**Author**: Engineering Team

---

## Table of Contents

1. [Document Overview](#document-overview)
2. [Technical Goals & Constraints](#technical-goals--constraints)
3. [System Overview](#system-overview)
4. [Domain Model](#domain-model)
5. [Filesystem Contract](#filesystem-contract)
6. [Command Specifications](#command-specifications)
7. [Architecture & Module Boundaries](#architecture--module-boundaries)
8. [Data Flows & Algorithms](#data-flows--algorithms)
9. [State & Caching](#state--caching)
10. [Security & Safety](#security--safety)
11. [Observability & UX](#observability--ux)
12. [Testing Strategy](#testing-strategy)
13. [Technical Risks & Mitigations](#technical-risks--mitigations)
14. [Open Questions & Deferred Decisions](#open-questions--deferred-decisions)

---

## Document Overview

### Purpose

This Technical Requirements Document (TRD) defines the technical implementation details for **Sikil**, a Rust CLI tool for managing Agent Skills across multiple AI coding agents. It translates the PRD functional requirements into concrete technical specifications, contracts, and architectural decisions.

### Related Documents

| Document | Purpose |
|----------|---------|
| [PRD.md](PRD.md) | Product requirements and scope |
| [use_cases.md](use_cases.md) | Use cases and acceptance criteria |
| [implementation_roadmap.md](implementation_roadmap.md) | Epic/task breakdown |
| [traceability_matrix.md](traceability_matrix.md) | Requirements coverage |

### Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-16 | Initial draft |
| 1.1 | 2026-01-16 | Security hardening: skill name validation, symlink policy, GitHub-only URLs, config hardening |

---

## Technical Goals & Constraints

### Technical Goals

| Goal | Metric | Rationale |
|------|--------|-----------|
| **Safety** | Zero data loss | All destructive operations are atomic with rollback |
| **Predictability** | Deterministic output | Scripting requires stable, consistent behavior |
| **Performance** | < 500ms for `list` | Fast feedback for interactive use |
| **Portability** | macOS + Linux | Support primary developer platforms |
| **Simplicity** | Single binary | Easy installation, no runtime dependencies |

### Technical Constraints

| Constraint | Rationale |
|------------|-----------|
| **No Windows support (v1)** | Symlink semantics differ; requires platform abstraction |
| **No script execution** | Security: skills are read-only configuration |
| **No telemetry** | Privacy: no network calls except git clone |
| **Shell out to git** | Leverage existing auth, avoid libgit2 complexity |
| **Single-threaded (v1)** | Simplicity; performance targets are modest |

### Language & Toolchain

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | 2021 edition |
| Minimum Rust | 1.75+ | Required for newer clap features |
| Target platforms | x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu | |

---

## System Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           USER / SCRIPTS                            │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ CLI invocation
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         CLI LAYER (clap)                            │
│  • Argument parsing                                                 │
│  • Command dispatch                                                 │
│  • Output formatting (human/JSON)                                   │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       COMMAND LAYER                                 │
│  • Orchestration logic                                              │
│  • Plan execution                                                   │
│  • Error handling & user messaging                                  │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         CORE LAYER                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │ Scanner  │  │ Planner  │  │ Validator│  │ Conflicts│            │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                          │
│  │  Parser  │  │  Config  │  │  Cache   │                          │
│  └──────────┘  └──────────┘  └──────────┘                          │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        UTILS LAYER                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐            │
│  │   Paths  │  │ Symlinks │  │  Atomic  │  │   Git    │            │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘            │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     EXTERNAL SYSTEMS                                │
│  • Filesystem (agent dirs, repo, workspace)                         │
│  • Git CLI                                                          │
│  • Config file                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### IO Boundaries

| Boundary | Type | Description |
|----------|------|-------------|
| Agent directories | Read/Write | Scan, create/remove symlinks |
| Managed repository | Read/Write | `~/.sikil/repo/` skill storage |
| Config file | Read/Write | `~/.sikil/config.toml` |
| Cache database | Read/Write | `~/.sikil/cache.db` |
| Git CLI | Subprocess | Clone operations |
| stdout | Write | Primary output (human/JSON) |
| stderr | Write | Errors, warnings, progress |

---

## Domain Model

### Core Types

#### Agent

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    ClaudeCode,
    Windsurf,
    OpenCode,
    KiloCode,
    Amp,
}

impl Agent {
    pub fn all() -> &'static [Agent] {
        &[Agent::ClaudeCode, Agent::Windsurf, Agent::OpenCode, 
          Agent::KiloCode, Agent::Amp]
    }
    
    pub fn cli_name(&self) -> &'static str {
        match self {
            Agent::ClaudeCode => "claude-code",
            Agent::Windsurf => "windsurf",
            Agent::OpenCode => "opencode",
            Agent::KiloCode => "kilo-code",
            Agent::Amp => "amp",
        }
    }
}
```

#### Agent Paths (Canonical)

| Agent | Global Path | Workspace Path |
|-------|-------------|----------------|
| ClaudeCode | `~/.claude/skills/` | `.claude/skills/` |
| Windsurf | `~/.codeium/windsurf/skills/` | `.windsurf/skills/` |
| OpenCode | `~/.config/opencode/skill/` | `.opencode/skill/` |
| KiloCode | `~/.kilocode/skills/` | `.kilocode/skills/` |
| Amp | `~/.config/agents/skills/` | `.agents/skills/` |

#### Scope

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Global,
    Workspace,
}
```

#### SkillMetadata

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Primary identifier (required)
    pub name: String,
    
    /// Human-readable description (required)
    pub description: String,
    
    /// Optional version string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// Optional author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    /// Optional license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}
```

#### Installation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    /// Which agent this installation belongs to
    pub agent: Agent,
    
    /// Absolute path to the installation
    pub path: PathBuf,
    
    /// Global or workspace scope
    pub scope: Scope,
    
    /// Whether this is a symlink
    pub is_symlink: bool,
    
    /// If symlink, the resolved target path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<PathBuf>,
    
    /// Whether the symlink points to managed repo
    pub is_managed: bool,
    
    /// Whether the symlink is broken
    pub is_broken: bool,
}
```

#### Skill

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Metadata from SKILL.md
    pub metadata: SkillMetadata,
    
    /// Directory name (may differ from metadata.name)
    pub directory_name: String,
    
    /// All installations across agents
    pub installations: Vec<Installation>,
    
    /// Whether this skill is managed (exists in ~/.sikil/repo/)
    pub is_managed: bool,
    
    /// Path in managed repo (if managed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<PathBuf>,
}
```

### Skill Identity Rules

| Rule | Description |
|------|-------------|
| **Primary identity** | YAML `name` field in SKILL.md frontmatter |
| **Name format** | Lowercase, alphanumeric + hyphens, max 64 chars |
| **No leading/trailing hyphens** | `my-skill` ✓, `-skill` ✗ |
| **No consecutive hyphens** | `my-skill` ✓, `my--skill` ✗ |
| **Directory mismatch** | Allowed; display as `name (dir:dirname)` |

### Conflict Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Same skill name exists as physical directories in multiple locations
    DuplicateUnmanaged,
    
    /// Same skill name exists as symlinks pointing to same repo location
    DuplicateManaged,
    
    /// Mix of managed symlinks and unmanaged physical directories
    MixedManagedUnmanaged,
    
    /// Symlink exists but target is missing
    BrokenSymlink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub skill_name: String,
    pub conflict_type: ConflictType,
    pub locations: Vec<Installation>,
    pub recommendation: String,
}
```

---

## Filesystem Contract

### Directory Structure

```
~/.sikil/                          # Sikil root directory
├── config.toml                     # Configuration file
├── cache.db                        # SQLite cache (optional)
└── repo/                           # Managed skill repository
    ├── git-workflow/               # Skill: git-workflow
    │   ├── SKILL.md
    │   ├── scripts/
    │   └── references/
    ├── code-review/                # Skill: code-review
    │   └── SKILL.md
    └── .../
```

### Symlink Rules

| Rule | Specification |
|------|---------------|
| **Symlink type** | Symbolic link (not hard link) |
| **Target path** | Absolute path to `~/.sikil/repo/<skill-name>/` |
| **Creation** | `std::os::unix::fs::symlink(target, link)` |
| **Detection** | `path.symlink_metadata()?.file_type().is_symlink()` |
| **Managed check** | Target path starts with expanded `~/.sikil/repo/` |

### Symlink Example

```
~/.claude/skills/git-workflow  →  ~/.sikil/repo/git-workflow/
                 (symlink)              (physical directory)
```

### Atomicity Contract

#### Atomic Operations

| Operation | Atomicity Requirement | Rollback Behavior |
|-----------|----------------------|-------------------|
| `install` | All-or-nothing | Remove partial copies and symlinks |
| `adopt` | All-or-nothing | Restore original directory |
| `unmanage` | All-or-nothing | Restore symlinks |
| `remove` | All-or-nothing | Restore removed items |
| `sync` | Per-agent atomic | Partial success allowed |

#### Implementation Pattern

```rust
// Atomic copy pattern
fn atomic_copy_dir(src: &Path, dest: &Path) -> Result<()> {
    // 1. Create temp directory on SAME filesystem as dest
    let temp_dir = tempfile::tempdir_in(dest.parent()?)?;
    let temp_dest = temp_dir.path().join(dest.file_name()?);
    
    // 2. Copy to temp location
    copy_dir_recursive(src, &temp_dest)?;
    
    // 3. Atomic rename (same filesystem guarantees atomicity)
    std::fs::rename(&temp_dest, dest)?;
    
    Ok(())
}

// Atomic move pattern
fn atomic_move_dir(src: &Path, dest: &Path) -> Result<()> {
    // Try rename first (fast path, same filesystem)
    match std::fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
            // Cross-filesystem: copy then remove
            atomic_copy_dir(src, dest)?;
            std::fs::remove_dir_all(src)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
```

### Workspace vs Global Handling

| Aspect | Behavior |
|--------|----------|
| **Scanning** | Scan both global and workspace (relative to CWD) |
| **Reporting** | Show scope label: `[global]` or `[workspace]` |
| **Priority** | Agent-specific; informational only in v1 |
| **Modification** | Allowed for both; workspace uses `--scope workspace` |

---

## Command Specifications

### Common Behavior

| Aspect | Specification |
|--------|---------------|
| **Exit code 0** | Success |
| **Exit code 1** | Error (operation failed) |
| **Exit code 2** | Invalid usage (bad arguments) |
| **`--json` mode** | JSON to stdout, messages to stderr |
| **`--help`** | Show command help |
| **`--version`** | Show version (root only) |

### Command: `list`

```
sikil list [OPTIONS]

OPTIONS:
    --agent <AGENT>     Filter by agent (claude-code, windsurf, etc.)
    --managed           Show only managed skills
    --unmanaged         Show only unmanaged skills
    --conflicts         Show only skills with conflicts
    --json              Output as JSON
```

| Precondition | None |
|--------------|------|
| **Postcondition** | Displays skills matching filters |
| **Idempotent** | Yes |

**Output Contract (JSON)**:
```json
{
  "schema_version": 1,
  "skills": [
    {
      "name": "string",
      "directory_name": "string",
      "description": "string",
      "version": "string|null",
      "is_managed": "boolean",
      "repo_path": "string|null",
      "installations": [
        {
          "agent": "string",
          "path": "string",
          "scope": "global|workspace",
          "is_symlink": "boolean",
          "symlink_target": "string|null",
          "is_managed": "boolean",
          "is_broken": "boolean"
        }
      ]
    }
  ],
  "summary": {
    "total": "number",
    "managed": "number",
    "unmanaged": "number",
    "conflicts": "number"
  }
}
```

### Command: `show`

```
sikil show <NAME> [OPTIONS]

ARGS:
    <NAME>    Skill name to show

OPTIONS:
    --json    Output as JSON
```

| Precondition | Skill with given name exists |
|--------------|------------------------------|
| **Postcondition** | Displays skill details |
| **Error: not found** | Exit 1, "Skill '<name>' not found" |

### Command: `validate`

```
sikil validate <PATH> [OPTIONS]

ARGS:
    <PATH>    Path to skill directory or installed skill name

OPTIONS:
    --json    Output as JSON
```

| Precondition | Path exists or skill name is installed |
|--------------|----------------------------------------|
| **Postcondition** | Displays validation results |
| **Exit 0** | All validations pass |
| **Exit 1** | One or more validations fail |

**Validation Rules**:

| Check | Severity | Rule |
|-------|----------|------|
| SKILL.md exists | Error | File must exist |
| Frontmatter present | Error | Must have `---` delimiters |
| YAML syntax valid | Error | Must parse as YAML |
| `name` present | Error | Required field |
| `name` format | Error | Matches naming regex |
| `description` present | Error | Required field |
| `description` length | Error | 1-1024 characters |
| `version` present | Warning | Optional but recommended |
| `author` present | Warning | Optional but recommended |

### Command: `install`

```
sikil install <SOURCE> [OPTIONS]

ARGS:
    <SOURCE>    Local path or Git URL

OPTIONS:
    --to <AGENTS>    Target agents (comma-separated or 'all')
```

| Precondition | Source is valid skill |
|--------------|----------------------|
| **Postcondition** | Skill in repo, symlinks created |
| **Error: exists** | Exit 1, "Skill already exists in repo" |
| **Error: blocked** | Exit 1, "Physical directory exists, use adopt" |

**Source Detection**:
```rust
enum InstallSource {
    LocalPath(PathBuf),
    GitUrl { url: String, subdir: Option<String> },
}

fn detect_source(input: &str) -> Result<InstallSource> {
    if input.starts_with("github.com/") ||
       input.starts_with("https://") ||
       input.starts_with("git@") {
        parse_git_url(input)
    } else {
        Ok(InstallSource::LocalPath(PathBuf::from(input)))
    }
}
```

### Command: `adopt`

```
sikil adopt <NAME> [OPTIONS]

ARGS:
    <NAME>    Skill name to adopt

OPTIONS:
    --from <AGENT>    Source agent (required if multiple locations)
```

| Precondition | Skill exists as unmanaged |
|--------------|---------------------------|
| **Postcondition** | Skill in repo, original replaced with symlink |
| **Error: managed** | Exit 1, "Skill is already managed" |
| **Error: ambiguous** | Exit 1, "Multiple locations, specify --from" |

### Command: `unmanage`

```
sikil unmanage <NAME> [OPTIONS]

ARGS:
    <NAME>    Skill name to unmanage

OPTIONS:
    --agent <AGENT>    Unmanage only for specific agent
    --yes              Skip confirmation
```

| Precondition | Skill is managed |
|--------------|------------------|
| **Postcondition** | Symlinks replaced with copies |
| **Confirmation** | Required unless `--yes` |

### Command: `remove`

```
sikil remove <NAME> [OPTIONS]

ARGS:
    <NAME>    Skill name to remove

OPTIONS:
    --agent <AGENT>    Remove from specific agent only
    --all              Remove from all agents and repo
    --yes              Skip confirmation
```

| Precondition | Skill exists |
|--------------|--------------|
| **Postcondition** | Skill removed from specified locations |
| **Confirmation** | Required unless `--yes` |

### Command: `sync`

```
sikil sync [NAME] [OPTIONS]

ARGS:
    [NAME]    Skill name (optional, syncs all if omitted)

OPTIONS:
    --to <AGENTS>    Target agents (default: all)
    --all            Sync all managed skills
```

| Precondition | Skill is managed (or --all) |
|--------------|-----------------------------|
| **Postcondition** | Symlinks created for missing agents |
| **Error: blocked** | Exit 1, "Physical directory blocks sync" |
| **Idempotent** | Yes |

### Command: `config`

```
sikil config [SUBCOMMAND]

SUBCOMMANDS:
    show              Display current config (default)
    edit              Open config in $EDITOR
    set <KEY> <VAL>   Set configuration value
```

---

## Architecture & Module Boundaries

### Module Layout

```
src/
├── main.rs                 # Entry point, clap setup
├── lib.rs                  # Library exports
│
├── cli/                    # CLI layer
│   ├── mod.rs
│   ├── args.rs             # Argument structs (derive(Parser))
│   └── output.rs           # Output formatting (human/JSON)
│
├── commands/               # Command orchestration
│   ├── mod.rs
│   ├── list.rs
│   ├── show.rs
│   ├── validate.rs
│   ├── install.rs
│   ├── adopt.rs
│   ├── unmanage.rs
│   ├── remove.rs
│   ├── sync.rs
│   └── config.rs
│
├── core/                   # Domain logic
│   ├── mod.rs
│   ├── skill.rs            # Skill, SkillMetadata, Installation
│   ├── agent.rs            # Agent enum, paths
│   ├── config.rs           # Config loading/saving
│   ├── parser.rs           # SKILL.md parsing
│   ├── validator.rs        # Validation logic
│   ├── scanner.rs          # Directory scanning
│   ├── conflicts.rs        # Conflict detection
│   ├── planner.rs          # Operation planning
│   ├── cache.rs            # SQLite cache
│   └── errors.rs           # Error types (thiserror)
│
└── utils/                  # Utilities
    ├── mod.rs
    ├── paths.rs            # Path expansion, repo paths
    ├── symlink.rs          # Symlink operations
    ├── atomic.rs           # Atomic file operations
    ├── git.rs              # Git clone, URL parsing
    └── format.rs           # Size formatting, colors
```

### Dependency Rules

```
cli/  ──────►  commands/  ──────►  core/  ──────►  utils/
                                     │
                                     ▼
                                  (std, external crates)
```

| Layer | May Depend On | Must Not Depend On |
|-------|---------------|-------------------|
| `cli/` | `commands/`, `core/` types | - |
| `commands/` | `core/`, `utils/` | `cli/` |
| `core/` | `utils/`, std | `cli/`, `commands/` |
| `utils/` | std, external crates | `cli/`, `commands/`, `core/` |

### External Dependencies

| Crate | Version | Purpose | Layer |
|-------|---------|---------|-------|
| clap | 4.x | CLI parsing | cli |
| clap_complete | 4.x | Shell completions | cli |
| serde | 1.x | Serialization | core |
| serde_yaml | 0.9.x | YAML parsing | core |
| serde_json | 1.x | JSON output | core |
| toml | 0.8.x | Config parsing | core |
| walkdir | 2.x | Directory traversal | core |
| rusqlite | 0.31.x | Cache database | core |
| anyhow | 1.x | Error context | commands |
| thiserror | 1.x | Error types | core |
| shellexpand | 3.x | Path expansion | utils |
| directories | 5.x | Standard directories | utils |
| fs-err | 2.x | Better fs errors | utils |
| tempfile | 3.x | Temp directories | utils |
| anstream | 0.6.x | Colored output | cli |
| anstyle | 1.x | Style definitions | cli |

---

## Data Flows & Algorithms

### Scanner Algorithm

```
Input: Config, CWD
Output: ScanResult { skills: Vec<Skill>, conflicts: Vec<Conflict> }

1. For each enabled agent in config:
   a. Resolve global path (expand ~)
   b. Resolve workspace path (relative to CWD)
   c. For each path that exists:
      i.   List subdirectories
      ii.  For each subdirectory:
           - Check if symlink
           - If symlink, resolve target, check if managed
           - Parse SKILL.md (or mark as invalid)
           - Create Installation record
      
2. Group installations by skill name (metadata.name)

3. For each group:
   a. Create Skill with all installations
   b. Determine is_managed (any installation is managed symlink)
   c. Detect conflicts:
      - Multiple physical dirs → DuplicateUnmanaged
      - All symlinks to same target → DuplicateManaged (not really conflict)
      - Mix of physical and symlinks → MixedManagedUnmanaged
      - Broken symlinks → BrokenSymlink

4. Return ScanResult
```

### Install Planner Algorithm

```
Input: Source path, Target agents, Config
Output: InstallPlan { skill_name, repo_dest, symlinks: Vec<(Agent, Path)> }

1. Validate source skill
   - SKILL.md exists and valid
   - Name matches constraints

2. Extract skill_name from metadata

3. Check repo_dest = ~/.sikil/repo/<skill_name>/
   - If exists → Error: already exists

4. For each target agent:
   a. Resolve agent global path
   b. Check dest = agent_path/<skill_name>/
      - If symlink exists → Error: already installed
      - If physical dir exists → Error: use adopt instead
   c. Add to symlinks list

5. Return InstallPlan
```

### Conflict Detection Algorithm

```
Input: Vec<Skill>
Output: Vec<Conflict>

For each skill:
  Let physical_count = installations.filter(|i| !i.is_symlink).count()
  Let symlink_count = installations.filter(|i| i.is_symlink).count()
  Let managed_count = installations.filter(|i| i.is_managed).count()
  Let broken_count = installations.filter(|i| i.is_broken).count()
  
  If broken_count > 0:
    → Conflict::BrokenSymlink
  
  If physical_count > 1:
    → Conflict::DuplicateUnmanaged
  
  If physical_count > 0 && managed_count > 0:
    → Conflict::MixedManagedUnmanaged
  
  (DuplicateManaged is not a real conflict - expected state)
```

### Git URL Parsing (GitHub-Only)

Sikil v1 only supports GitHub repositories for security and simplicity.

#### Accepted URL Formats

| Format | Example | Clone URL | Subdir |
|--------|---------|-----------|--------|
| Short form | `user/repo` | `https://github.com/user/repo.git` | None |
| Short with subdir | `user/repo/skills/my-skill` | `https://github.com/user/repo.git` | `skills/my-skill` |
| HTTPS with .git | `https://github.com/user/repo.git` | As-is | None |
| HTTPS without .git | `https://github.com/user/repo` | Append `.git` | None |

#### Rejected URL Formats

| Format | Reason |
|--------|--------|
| `file://` | Local file protocol - security risk |
| `git@github.com:...` | SSH format - defer to v1.1 |
| Non-GitHub HTTPS | Only GitHub allowed in v1 |
| URLs with whitespace/NUL | Injection prevention |
| URLs starting with `-` | Argument injection prevention |

#### URL Validation Algorithm

```rust
use regex::Regex;
use once_cell::sync::Lazy;

/// Matches: user/repo or user/repo/optional/subdir
static SHORT_FORM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([a-zA-Z0-9_-]+)/([a-zA-Z0-9._-]+)(?:/(.+))?$").unwrap()
});

/// Matches: https://github.com/user/repo or https://github.com/user/repo.git
static GITHUB_HTTPS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https://github\.com/([a-zA-Z0-9_-]+)/([a-zA-Z0-9._-]+?)(\.git)?$").unwrap()
});

#[derive(Debug)]
pub struct ParsedGitUrl {
    pub clone_url: String,
    pub subdir: Option<String>,
}

pub fn parse_git_url(input: &str) -> Result<ParsedGitUrl> {
    // Security: reject dangerous patterns
    if input.contains('\0') || input.contains(' ') || input.contains('\t') {
        return Err(SikilError::InvalidGitUrl { 
            url: input.to_string(),
            reason: "URL contains invalid characters".to_string(),
        });
    }
    
    if input.starts_with('-') {
        return Err(SikilError::InvalidGitUrl { 
            url: input.to_string(),
            reason: "URL cannot start with '-' (argument injection)".to_string(),
        });
    }
    
    if input.starts_with("file://") {
        return Err(SikilError::InvalidGitUrl { 
            url: input.to_string(),
            reason: "file:// protocol not allowed".to_string(),
        });
    }
    
    // Try short form: user/repo or user/repo/subdir
    if let Some(caps) = SHORT_FORM_REGEX.captures(input) {
        let user = caps.get(1).unwrap().as_str();
        let repo = caps.get(2).unwrap().as_str();
        let subdir = caps.get(3).map(|m| m.as_str().to_string());
        
        return Ok(ParsedGitUrl {
            clone_url: format!("https://github.com/{}/{}.git", user, repo),
            subdir,
        });
    }
    
    // Try GitHub HTTPS URL
    if let Some(caps) = GITHUB_HTTPS_REGEX.captures(input) {
        let user = caps.get(1).unwrap().as_str();
        let repo = caps.get(2).unwrap().as_str();
        
        return Ok(ParsedGitUrl {
            clone_url: format!("https://github.com/{}/{}.git", user, repo),
            subdir: None,
        });
    }
    
    Err(SikilError::InvalidGitUrl { 
        url: input.to_string(),
        reason: "Must be GitHub URL: user/repo, user/repo/subdir, or https://github.com/user/repo".to_string(),
    })
}
```

#### Git Clone Safety

```rust
use std::process::Command;

pub fn clone_repo(url: &ParsedGitUrl, dest: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("clone")
        .arg("--depth=1")           // Shallow clone
        .arg("-c").arg("protocol.file.allow=never")  // Block file protocol
        .arg("--")                  // End of options (prevent URL as flag)
        .arg(&url.clone_url)
        .arg(dest)
        .output()?;
    
    if !output.status.success() {
        return Err(SikilError::GitCloneFailed {
            url: url.clone_url.clone(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    
    Ok(())
}
```

---

## State & Caching

### Cache Schema (SQLite)

```sql
-- Database initialization with WAL mode for crash safety
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;

-- Skill cache table
CREATE TABLE IF NOT EXISTS skill_cache (
    path TEXT PRIMARY KEY,           -- Absolute path to skill directory
    skill_name TEXT NOT NULL,        -- YAML name field
    metadata_json TEXT NOT NULL,     -- Full metadata as JSON
    mtime INTEGER NOT NULL,          -- Modification time (seconds)
    size INTEGER NOT NULL,           -- SKILL.md file size (bytes)
    content_hash TEXT NOT NULL,      -- SHA256 of SKILL.md content
    scanned_at INTEGER NOT NULL      -- Cache entry timestamp
);

CREATE INDEX IF NOT EXISTS idx_skill_name ON skill_cache(skill_name);
```

### Database Initialization

```rust
use rusqlite::Connection;
use sha2::{Sha256, Digest};

pub fn init_cache_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    
    // Enable WAL mode for better crash recovery
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
    ")?;
    
    // Create schema
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS skill_cache (
            path TEXT PRIMARY KEY,
            skill_name TEXT NOT NULL,
            metadata_json TEXT NOT NULL,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL,
            content_hash TEXT NOT NULL,
            scanned_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_skill_name ON skill_cache(skill_name);
    ")?;
    
    Ok(conn)
}

/// Compute SHA256 hash of file content
pub fn hash_file_content(path: &Path) -> Result<String> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}
```

### Cache Invalidation Strategy

| Check | Method | Rationale |
|-------|--------|-----------|
| mtime changed | `fs::metadata().modified()` | Fast, catches most edits |
| size changed | `fs::metadata().len()` | Catches truncation/expansion |
| content hash | SHA256 of SKILL.md | Catches in-place edits with same mtime |

```rust
fn is_cache_valid(entry: &CacheEntry, path: &Path) -> bool {
    let meta = fs::metadata(path.join("SKILL.md")).ok()?;
    let mtime = meta.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_secs();
    let size = meta.len();
    
    entry.mtime == mtime && entry.size == size
    // Note: content_hash check only if mtime/size match but want extra safety
}
```

### Cache Bypass

- `--no-cache` flag bypasses cache entirely
- Cache is invalidated on any write operation (install, adopt, etc.)
- Cache file is not critical; deleted cache triggers full rescan

---

## Security & Safety

### Path Traversal Prevention

```rust
/// Validates a path is within a root directory.
/// Uses lexical checks first, then canonicalization for existing paths.
fn validate_path_within(path: &Path, root: &Path) -> Result<()> {
    // Lexical check: reject obvious traversal attempts
    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(SikilError::PathTraversal {
                path: path.to_path_buf(),
                root: root.to_path_buf(),
            });
        }
    }
    
    // If path exists, verify canonical path is within root
    if path.exists() {
        let canonical_path = path.canonicalize()?;
        let canonical_root = root.canonicalize()?;
        
        if !canonical_path.starts_with(&canonical_root) {
            return Err(SikilError::PathTraversal {
                path: path.to_path_buf(),
                root: root.to_path_buf(),
            });
        }
    }
    
    Ok(())
}

/// Validates a destination path before creation.
/// For non-existent paths, uses lexical validation only.
fn validate_dest_path(skill_name: &str, root: &Path) -> Result<PathBuf> {
    // First validate skill name (prevents path injection)
    validate_skill_name(skill_name)?;
    
    // Construct destination - skill_name is now safe
    let dest = root.join(skill_name);
    
    // If dest already exists, verify it's within root
    if dest.exists() {
        validate_path_within(&dest, root)?;
    }
    
    Ok(dest)
}
```

### Symlink Policy (No Symlinks in Skills)

Skills must not contain symlinks. This prevents:
- Path traversal via symlinks to sensitive files (e.g., `/etc/passwd`)
- Escape from skill directory via `../../` relative symlinks
- Agents following malicious symlinks when reading skill content

```rust
/// Copies a skill directory, rejecting any symlinks found.
/// This is the ONLY safe copy function for untrusted skill content.
fn copy_skill_dir(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        let file_name = entry.file_name();
        
        // Skip .git directory
        if file_name == ".git" {
            continue;
        }
        
        let file_type = entry.file_type()?;
        
        if file_type.is_symlink() {
            // SECURITY: Reject all symlinks in skill content
            return Err(SikilError::SymlinkNotAllowed {
                path: src_path,
                reason: "Skills cannot contain symlinks for security reasons".to_string(),
            });
        } else if file_type.is_dir() {
            copy_skill_dir(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }
    
    // Best-effort permission preservation
    if let Ok(meta) = fs::metadata(src) {
        let _ = fs::set_permissions(dest, meta.permissions());
    }
    
    Ok(())
}
```

### Git Clone Safety

| Rule | Implementation |
|------|----------------|
| Clone to temp dir | Use `tempfile::tempdir()` |
| Validate subdir | Check extracted path is within clone root |
| No .git in output | Exclude `.git/` when copying to repo |
| **Reject symlinks** | Error if any symlink found in skill content |
| Use `--` separator | Prevents URL from being parsed as git options |
| Block file protocol | `-c protocol.file.allow=never` |

### No Script Execution

- Sikil never executes any file within a skill
- `scripts/` directory is just copied (no symlinks allowed)
- Agents may execute scripts; Sikil does not

### Configuration File Hardening

```rust
use serde::Deserialize;

/// Maximum config file size (1 MB)
const MAX_CONFIG_SIZE: u64 = 1_048_576;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]  // Reject typos and unknown keys
pub struct Config {
    #[serde(default)]
    pub agents: AgentsConfig,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AgentsConfig {
    #[serde(default, rename = "claude-code")]
    pub claude_code: Option<AgentConfig>,
    #[serde(default)]
    pub windsurf: Option<AgentConfig>,
    #[serde(default)]
    pub opencode: Option<AgentConfig>,
    #[serde(default, rename = "kilo-code")]
    pub kilo_code: Option<AgentConfig>,
    #[serde(default)]
    pub amp: Option<AgentConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub global_path: Option<String>,
    pub workspace_path: Option<String>,
}

fn default_true() -> bool { true }

pub fn load_config(path: &Path) -> Result<Config> {
    // Check file size before reading
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_CONFIG_SIZE {
        return Err(SikilError::ConfigTooLarge {
            size: metadata.len(),
            max: MAX_CONFIG_SIZE,
        });
    }
    
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    
    Ok(config)
}
```

---

## Observability & UX

### Output Styling

| Element | Style |
|---------|-------|
| Skill name | Bold |
| Success indicator | Green `✓` |
| Error indicator | Red `✗` |
| Warning indicator | Yellow `⚠` |
| Paths | Dim |
| Agent names | Cyan |

### NO_COLOR Support

```rust
fn init_colors() {
    if std::env::var("NO_COLOR").is_ok() {
        // Disable all colors
        anstream::ColorChoice::Never
    } else if atty::is(atty::Stream::Stdout) {
        anstream::ColorChoice::Auto
    } else {
        anstream::ColorChoice::Never
    }
}
```

### Progress Indicators

| Operation | Indicator |
|-----------|-----------|
| Git clone | Spinner with "Cloning repository..." |
| Large copy | Progress bar with bytes/total |
| Scanning | Spinner with "Scanning skills..." (if > 100ms) |

### Error Messages

Format:
```
Error: <what went wrong>

<context/explanation>

<how to fix it>
```

Example:
```
Error: Cannot install 'git-workflow' - skill already exists

The skill 'git-workflow' already exists in the managed repository.
Location: ~/.sikil/repo/git-workflow/

To update the skill, first remove it:
  sikil remove git-workflow --all

Or use a different name when installing.
```

---

## Testing Strategy

### Test Categories

| Category | Location | Purpose |
|----------|----------|---------|
| Unit tests | `src/**/tests.rs` | Module-level logic |
| Integration tests | `tests/*.rs` | Full command flows |
| Snapshot tests | `tests/snapshots/` | Output stability |

### Test Helpers

```rust
// tests/common/mod.rs

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub home_dir: PathBuf,
    pub cwd: PathBuf,
    pub skills_repo: PathBuf,
}

impl TestEnv {
    pub fn new() -> Self { ... }
    
    pub fn create_skill(&self, name: &str, agent: Agent, scope: Scope) -> PathBuf { ... }
    
    pub fn create_managed_skill(&self, name: &str) -> PathBuf { ... }
    
    pub fn run_sikil(&self, args: &[&str]) -> Output { ... }
}
```

### Test Fixtures

```
tests/fixtures/
├── valid-skill/
│   └── SKILL.md              # Valid minimal skill
├── full-skill/
│   ├── SKILL.md              # All optional fields
│   ├── scripts/
│   │   └── helper.sh
│   └── references/
│       └── doc.md
├── invalid-no-frontmatter/
│   └── SKILL.md              # Missing --- delimiters
├── invalid-no-name/
│   └── SKILL.md              # Missing name field
└── invalid-bad-name/
    └── SKILL.md              # Name with invalid chars
```

### Snapshot Tests

```rust
#[test]
fn test_list_output() {
    let env = TestEnv::new();
    env.create_skill("git-workflow", Agent::ClaudeCode, Scope::Global);
    env.create_skill("code-review", Agent::Windsurf, Scope::Global);
    
    let output = env.run_sikil(&["list"]);
    
    insta::assert_snapshot!(output.stdout);
}

#[test]
fn test_list_json() {
    let env = TestEnv::new();
    env.create_skill("git-workflow", Agent::ClaudeCode, Scope::Global);
    
    let output = env.run_sikil(&["list", "--json"]);
    
    insta::assert_json_snapshot!(serde_json::from_str::<Value>(&output.stdout).unwrap());
}
```

### CI Matrix

| Platform | Rust Version | Notes |
|----------|--------------|-------|
| ubuntu-latest | stable | Primary |
| ubuntu-latest | beta | Compatibility |
| macos-latest | stable | macOS Intel/ARM |

---

## Technical Risks & Mitigations

### Risk: Symlink Failures

| Scenario | Detection | Mitigation |
|----------|-----------|------------|
| Network drive | `symlink()` returns EOPNOTSUPP | Error with explanation, suggest local install |
| Sandboxed home | `symlink()` returns EPERM | Error with workaround instructions |
| Windows WSL edge cases | Various errors | Document limitation |

### Risk: Atomic Operation Failures

| Scenario | Detection | Mitigation |
|----------|-----------|------------|
| Cross-filesystem move | EXDEV from `rename()` | Fall back to copy+delete |
| Disk full mid-copy | ENOSPC | Rollback temp dir, clear error |
| Permission denied | EACCES | Clear error, check permissions |
| Interrupted (Ctrl+C) | Signal handler | Best-effort cleanup |

### Risk: Git Clone Issues

| Scenario | Detection | Mitigation |
|----------|-----------|------------|
| Auth failure | Non-zero exit from git | Surface stderr, suggest fixes |
| Network timeout | Git error | Retry suggestion |
| Subdir not found | Path doesn't exist | Clear error with repo contents |

### Risk: Cache Correctness

| Scenario | Detection | Mitigation |
|----------|-----------|------------|
| Stale cache | Incorrect results | mtime + size + hash validation |
| Corrupted cache | SQLite errors | Auto-delete and rebuild |
| Cache too large | Disk usage | Periodic cleanup of old entries |

### Risk: Binary Size

| Factor | Impact | Mitigation |
|--------|--------|------------|
| rusqlite bundled | ~2-3MB | Monitor in CI, remove if needed |
| clap features | ~500KB | Use minimal features |
| Total target | < 10MB | Build with `--release`, strip symbols |

---

## Open Questions & Deferred Decisions

### Deferred to v1.1

| Question | Current Decision | Future Consideration |
|----------|------------------|----------------------|
| XDG paths on Linux | Use `~/.sikil/` | Add XDG support with fallback |
| Relative vs absolute symlinks | Absolute | Relative for portability |
| Windows support | Not supported | Requires platform abstraction |
| Version comparison | Not implemented | Semantic version diffing |
| SSH Git URLs | HTTPS only | Add `git@github.com:` support |
| Non-GitHub repos | GitHub only | Evaluate GitLab, Bitbucket support |

### Open Questions

| Question | Options | Recommendation |
|----------|---------|----------------|
| Cache implementation | SQLite vs JSON file | SQLite for query flexibility |
| Parallel scanning | Single-threaded vs rayon | Single-threaded (simpler) |
| Config format | TOML vs YAML | TOML (Rust ecosystem standard) |
| Progress bars | indicatif vs custom | indicatif if size allows |

---

## Appendix: SKILL.md Format Reference

```yaml
---
# Required fields
name: skill-name                    # lowercase, alphanumeric + hyphens
description: "What this skill does" # 1-1024 characters

# Optional fields
version: 1.0                        # Semver recommended
author: Your Name                   # Free text
license: MIT                        # SPDX identifier recommended
---

# Skill Title

Instructions for the AI agent go here...
```

### Name Validation Regex

```rust
use once_cell::sync::Lazy;
use regex::Regex;

/// Skill names must:
/// - Start with lowercase letter or digit
/// - Contain only lowercase letters, digits, hyphens, and underscores
/// - Be 1-64 characters long
/// - Not contain path separators or traversal sequences
static SKILL_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9][a-z0-9_-]{0,63}$").unwrap()
});

fn validate_skill_name(name: &str) -> Result<()> {
    // Reject path traversal attempts
    if name.contains('/') || name.contains('\\') || name == ".." || name == "." {
        return Err(SikilError::PathTraversal { 
            name: name.to_string() 
        });
    }
    
    if name.is_empty() {
        return Err(SikilError::InvalidNameFormat { 
            name: name.to_string(),
            reason: "name cannot be empty".to_string(),
        });
    }
    
    if name.len() > 64 {
        return Err(SikilError::NameTooLong { len: name.len() });
    }
    
    if !SKILL_NAME_REGEX.is_match(name) {
        return Err(SikilError::InvalidNameFormat { 
            name: name.to_string(),
            reason: "must start with lowercase letter/digit, contain only a-z, 0-9, -, _".to_string(),
        });
    }
    
    Ok(())
}
```

---

**Document Version**: 1.1  
**Last Updated**: January 16, 2026  
**Next Review**: February 16, 2026
