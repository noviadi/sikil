# Skill Model Spec

## One-Sentence Description

The skill model defines core data structures for representing skills.

## Overview

The skill model (`src/core/skill.rs`) provides the foundational types for representing Agent Skills in sikil. It defines what a skill is (metadata from SKILL.md), where it's installed (installations), which agents it supports, and whether it's managed by sikil. The parser module (`src/core/parser.rs`) handles deserializing SKILL.md files into these structures.

## Data Structures

### SkillMetadata

Metadata parsed from a SKILL.md file's YAML frontmatter.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String` | Yes | Primary identifier |
| `description` | `String` | Yes | Human-readable description |
| `version` | `Option<String>` | No | Version string |
| `author` | `Option<String>` | No | Author name |
| `license` | `Option<String>` | No | License identifier |

Builder methods: `new()`, `with_version()`, `with_author()`, `with_license()`.

### Skill

Represents a skill discovered on the filesystem.

| Field | Type | Description |
|-------|------|-------------|
| `metadata` | `SkillMetadata` | Metadata from SKILL.md |
| `directory_name` | `String` | Directory name (may differ from `metadata.name`) |
| `installations` | `Vec<Installation>` | All installations across agents |
| `is_managed` | `bool` | Whether skill exists in `~/.sikil/repo/` |
| `repo_path` | `Option<PathBuf>` | Path in managed repo (if managed) |

Methods:
- `new()` - Creates skill with metadata and directory name
- `with_installation()` - Adds an installation
- `with_repo()` - Sets managed flag and repo path
- `is_orphan()` - Returns true if no installations exist

### Installation

Represents a skill installation at an agent location.

| Field | Type | Description |
|-------|------|-------------|
| `agent` | `Agent` | Which agent this installation belongs to |
| `path` | `PathBuf` | Absolute path to the installation |
| `scope` | `Scope` | Global or workspace scope |
| `is_symlink` | `Option<bool>` | Whether installation is a symlink |
| `symlink_target` | `Option<PathBuf>` | Target path if symlink |

### Agent (enum)

Supported AI coding agents.

| Variant | CLI Name | Description |
|---------|----------|-------------|
| `ClaudeCode` | `claude-code` | Claude Code (Anthropic) |
| `Windsurf` | `windsurf` | Windsurf (Codeium) |
| `OpenCode` | `opencode` | OpenCode |
| `KiloCode` | `kilocode` | KiloCode |
| `Amp` | `amp` | Amp |

Methods:
- `all()` - Returns slice of all agents
- `cli_name()` - Returns kebab-case CLI name
- `from_cli_name()` - Parses from CLI name string
- `Display` trait - Returns CLI name

### Scope (enum)

Installation scope.

| Variant | Serde Value | Description |
|---------|-------------|-------------|
| `Global` | `global` | Global installation (e.g., `~/.claude/skills/`) |
| `Workspace` | `workspace` | Workspace installation (e.g., `.claude/skills/`) |

## Validation Rules

Skill name validation (`validate_skill_name()` in parser.rs):

| Rule | Description |
|------|-------------|
| Non-empty | Name cannot be empty |
| Pattern | Must match `^[a-z0-9][a-z0-9_-]{0,63}$` |
| Start character | Must start with lowercase letter or digit |
| Allowed characters | Lowercase letters, digits, hyphens, underscores only |
| Length | 1-64 characters |
| No path separators | Cannot contain `/` or `\` |
| No traversal | Cannot be `.` or `..` |

Frontmatter validation:
- Must have opening and closing `---` markers
- Opening marker must be at file start (only whitespace allowed before)
- Required fields: `name`, `description`

## Serialization

All structs derive `Serialize` and `Deserialize` from serde.

| Type | Serde Configuration |
|------|---------------------|
| `Agent` | `#[serde(rename_all = "kebab-case")]` |
| `Scope` | `#[serde(rename_all = "lowercase")]` |
| Optional fields | `#[serde(skip_serializing_if = "Option::is_none")]` |

Parsing flow:
1. `extract_frontmatter()` - Extracts YAML between `---` markers
2. `RawSkillMetadata` - Intermediate struct with all fields optional
3. Supports nested `metadata:` block for author/version/license
4. Validation of required fields after parsing
5. Name validation via `validate_skill_name()`

## Acceptance Criteria

- `SkillMetadata::new("my-skill", "desc")` creates metadata with name and description
- `SkillMetadata` with `None` version/author/license serializes without those fields
- `Skill::is_orphan()` returns `true` when `installations` is empty
- `Skill::is_orphan()` returns `false` when at least one installation exists
- `Skill::with_repo(path)` sets `is_managed` to `true` and `repo_path` to `Some(path)`
- `Agent::all()` returns slice containing all 5 agent variants
- `Agent::cli_name()` returns `"claude-code"` for `Agent::ClaudeCode`
- `Agent::from_cli_name("amp")` returns `Some(Agent::Amp)`
- `Agent::from_cli_name("invalid")` returns `None`
- `Scope::Global` serializes to `"global"`
- `Scope::Workspace` serializes to `"workspace"`
- Skill name `"my-skill"` passes validation
- Skill name `"my_skill_123"` passes validation
- Empty skill name returns validation error
- Skill name starting with `-` returns validation error
- Skill name with uppercase letters returns validation error
- Skill name longer than 64 characters returns validation error
- Skill name `"."` or `".."` returns validation error
- Skill name containing `/` or `\` returns validation error
- SKILL.md without `---` frontmatter markers returns `InvalidSkillMd` error
- SKILL.md missing required `name` field returns `InvalidSkillMd` error
- SKILL.md missing required `description` field returns `InvalidSkillMd` error

## Dependencies

| Dependency | Usage |
|------------|-------|
| `serde` | Serialization/deserialization |
| `serde_yaml` | YAML frontmatter parsing (in parser) |
| `regex` | Skill name validation pattern |
| `fs-err` | File reading with better errors (in parser) |
| `std::path::PathBuf` | File paths |

Internal dependencies:
- `core::errors::SikilError` - Error types for validation failures

## Used By

| Module | Usage |
|--------|-------|
| `core::scanner` | Creates `Skill` and `Installation` from filesystem |
| `core::parser` | Produces `SkillMetadata` from SKILL.md files |
| `core::conflicts` | Uses `Installation` for conflict detection |
| `commands::list` | Displays `Skill`, `Agent`, `Scope` |
| `commands::show` | Displays skill details |
| `commands::install` | Uses parser to validate skills |
| `commands::validate` | Uses parser functions for validation |
| `commands::adopt` | Uses `Agent` for agent selection |
| `commands::sync` | Uses `Agent` for sync operations |
| `commands::remove` | Uses `Agent` for removal |
| `commands::unmanage` | Uses `Agent` for unmanage operations |
| `commands::agent_selection` | Uses `Agent` enum |
| `main.rs` | Uses `Agent` for CLI argument parsing |
