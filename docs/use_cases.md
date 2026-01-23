# Sikil - Use Cases & Acceptance Criteria

## Overview

This document defines all use cases and acceptance criteria for **Sikil**, a CLI tool for managing Agent Skills across multiple coding agents (Claude Code, Windsurf, OpenCode, Kilo Code, Amp).

### Key Concepts

| Term | Definition |
|------|------------|
| **Managed skill** | Skill stored in `~/.sikil/repo/` and symlinked to agent directories |
| **Unmanaged skill** | Skill existing as physical directory in agent directories (not managed by Sikil) |
| **Skill identity** | YAML `name` field in SKILL.md (primary identifier) |
| **Global paths** | Agent skill directories in home directory (e.g., `~/.claude/skills/`) |
| **Workspace paths** | Agent skill directories in current project (e.g., `.claude/skills/`) |

### Supported Agents

| Agent | Global Path | Workspace Path |
|-------|-------------|----------------|
| Claude Code | `~/.claude/skills/` | `.claude/skills/` |
| Windsurf | `~/.codeium/windsurf/skills/` | `.windsurf/skills/` |
| OpenCode | `~/.config/opencode/skill/` | `.opencode/skill/` |
| Kilo Code | `~/.kilocode/skills/` | `.kilocode/skills/` |
| Amp | `~/.config/agents/skills/` | `.agents/skills/` |

---

## UC-01: List All Skills

### Description
As a developer, I want to list all installed skills across all agents so that I can see what skills are available and their status.

### Command
```bash
sikil list
sikil list --agent claude-code
sikil list --managed
sikil list --unmanaged
sikil list --json
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-01-01 | System scans all global paths for the 5 supported agents |
| UC-01-02 | System scans workspace paths relative to current working directory |
| UC-01-03 | Each skill displays: name (from YAML), directory name (if different), agent, scope (global/workspace), managed/unmanaged status |
| UC-01-04 | Managed skills show symlink indicator and target path |
| UC-01-05 | Unmanaged skills show physical path |
| UC-01-06 | `--agent <name>` filters results to specific agent |
| UC-01-07 | `--managed` shows only managed skills |
| UC-01-08 | `--unmanaged` shows only unmanaged skills |
| UC-01-09 | `--json` outputs results in JSON format |
| UC-01-10 | Skills with missing/invalid SKILL.md are listed with warning label |
| UC-01-11 | Display format: `skill-name (dir:actual-dir-name)` when directory name differs from YAML name |
| UC-01-12 | Summary shows total counts: managed, unmanaged, by agent |

### Example Output
```
Managed Skills (12):
  git-workflow         claude-code, windsurf, opencode    [global]
  code-review          claude-code, windsurf              [global]
  deploy               claude-code, amp                   [global]

Unmanaged Skills (5):
  project-rules        claude-code                        [workspace] ./claude/skills/
  legacy-workflow      windsurf                           [global]    ~/.codeium/windsurf/skills/
  api-design (dir:my-api-design)  opencode               [global]    ~/.config/opencode/skill/

Summary: 17 skills (12 managed, 5 unmanaged) across 5 agents
```

---

## UC-02: Show Skill Details

### Description
As a developer, I want to view detailed information about a specific skill so that I can understand its purpose and installation status.

### Command
```bash
sikil show <skill-name>
sikil show <skill-name> --json
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-02-01 | Display skill metadata: name, description, version (if defined), author (if defined) |
| UC-02-02 | Show all installation locations with agent name and path |
| UC-02-03 | Indicate managed/unmanaged status for each installation |
| UC-02-04 | For managed skills, show canonical path in `~/.sikil/repo/` |
| UC-02-05 | For managed skills, list all symlinked agent locations |
| UC-02-06 | Show file listing within skill directory (SKILL.md, scripts/, references/) |
| UC-02-07 | Display total size on disk |
| UC-02-08 | `--json` outputs full details in JSON format |
| UC-02-09 | If skill not found, display helpful error message |
| UC-02-10 | If multiple skills match (conflict), show all with duplicate warning |

### Example Output
```
Skill: git-workflow
Description: Git workflow automation with conventional commits
Version: 2.0
Author: DevTeam

Status: MANAGED
Canonical: ~/.sikil/repo/git-workflow/

Installations:
  ✓ claude-code   ~/.claude/skills/git-workflow → ~/.sikil/repo/git-workflow
  ✓ windsurf      ~/.codeium/windsurf/skills/git-workflow → ~/.sikil/repo/git-workflow
  ✓ opencode      ~/.config/opencode/skill/git-workflow → ~/.sikil/repo/git-workflow
  ✗ kilo-code     (not installed)
  ✗ amp           (not installed)

Files:
  ├── SKILL.md (3.2 KB)
  ├── scripts/
  │   ├── commit-helper.sh
  │   └── branch-naming.py
  └── references/
      └── CONVENTIONS.md

Total size: 15.4 KB
```

---

## UC-03: Install Skill from Local Path

### Description
As a developer, I want to install a skill from a local directory so that I can add new skills to my managed collection.

### Command
```bash
sikil install ./path/to/skill
sikil install ./path/to/skill --to claude-code,windsurf
sikil install ./path/to/skill --to all
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-03-01 | Validate source directory contains valid SKILL.md with required `name` field |
| UC-03-02 | Copy skill directory to `~/.sikil/repo/<skill-name>/` |
| UC-03-03 | If `--to` not specified, prompt user to select target agents |
| UC-03-04 | If `--to all` specified, install to all 5 agents' global paths |
| UC-03-05 | Create symlinks in each specified agent's global skill directory |
| UC-03-06 | If skill name already exists in repo, fail with error (no overwrite) |
| UC-03-07 | If symlink target already exists as physical directory, fail with error suggesting `adopt` |
| UC-03-08 | If symlink target already exists as symlink, fail with error (already installed) |
| UC-03-09 | Display success message with list of created symlinks |
| UC-03-10 | Validate SKILL.md before installation (use validate logic) |
| UC-03-11 | Create agent skill directories if they don't exist |

### Example Output
```
$ sikil install ./my-new-skill --to claude-code,windsurf

Validating skill...
  ✓ SKILL.md found
  ✓ Name: my-new-skill
  ✓ Description: present

Installing...
  ✓ Copied to ~/.sikil/repo/my-new-skill/
  ✓ Symlinked to ~/.claude/skills/my-new-skill
  ✓ Symlinked to ~/.codeium/windsurf/skills/my-new-skill

Done! Skill 'my-new-skill' installed to 2 agents.
```

---

## UC-04: Install Skill from Git Repository

### Description
As a developer, I want to install a skill from a Git repository so that I can use community or team-shared skills.

### Command
```bash
sikil install user/skill-repo
sikil install user/multi-skills/path/to/skill
sikil install https://github.com/user/skill-repo.git
sikil install https://github.com/user/skill-repo
```

### Accepted URL Formats (GitHub-only in v1)

| Format | Example |
|--------|---------|
| Short form | `user/repo` |
| Short with subdir | `user/repo/skills/my-skill` |
| HTTPS with .git | `https://github.com/user/repo.git` |
| HTTPS without .git | `https://github.com/user/repo` |

> **Note**: SSH URLs (`git@github.com:...`) deferred to v1.1 for security review.

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-04-01 | Support GitHub shorthand: `user/repo` → `https://github.com/user/repo.git` |
| UC-04-02 | Support full HTTPS URL: `https://github.com/user/repo.git` |
| UC-04-03 | ~~Support SSH URL~~ (Deferred to v1.1) |
| UC-04-04 | Support subdirectory path: `user/repo/path/to/skill` |
| UC-04-05 | Clone repository to temporary directory |
| UC-04-06 | If subdirectory specified, extract only that path |
| UC-04-07 | Validate extracted directory contains valid SKILL.md |
| UC-04-08 | Copy validated skill to `~/.sikil/repo/<skill-name>/` (reject if contains symlinks) |
| UC-04-09 | Create symlinks to specified agents (same as local install) |
| UC-04-10 | Clean up temporary clone after installation |
| UC-04-11 | If git clone fails, display clear error message |
| UC-04-12 | If subdirectory not found in repo, display clear error |
| UC-04-13 | Do not store git metadata (.git/) in repo copy |
| UC-04-14 | Reject `file://` protocol with clear error |
| UC-04-15 | Reject URLs containing whitespace or starting with `-` |
| UC-04-16 | Reject non-GitHub URLs with clear error (v1 limitation) |
| UC-04-17 | Reject skills containing symlinks with security error |

### Example Output
```
$ sikil install github.com/anthropic/skills/git-workflow --to all

Cloning repository...
  ✓ Cloned github.com/anthropic/skills
  ✓ Found skill at /git-workflow

Validating skill...
  ✓ SKILL.md found
  ✓ Name: git-workflow

Installing...
  ✓ Copied to ~/.sikil/repo/git-workflow/
  ✓ Symlinked to ~/.claude/skills/git-workflow
  ✓ Symlinked to ~/.codeium/windsurf/skills/git-workflow
  ✓ Symlinked to ~/.config/opencode/skill/git-workflow
  ✓ Symlinked to ~/.kilocode/skills/git-workflow
  ✓ Symlinked to ~/.config/agents/skills/git-workflow

Done! Skill 'git-workflow' installed to 5 agents.
```

---

## UC-05: Adopt Unmanaged Skill

### Description
As a developer, I want to adopt an existing unmanaged skill into management so that I can sync it across agents.

### Command
```bash
sikil adopt <skill-name>
sikil adopt <skill-name> --from claude-code
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-05-01 | Identify unmanaged skill by name across all scanned paths |
| UC-05-02 | If skill exists in multiple locations, require `--from <agent>` to specify source |
| UC-05-03 | If `--from` not specified and multiple exist, display error with available locations |
| UC-05-04 | Move (not copy) skill directory to `~/.sikil/repo/<skill-name>/` |
| UC-05-05 | Replace original location with symlink to new repo location |
| UC-05-06 | If skill name already exists in `~/.sikil/repo/`, fail with error |
| UC-05-07 | Preserve all files, permissions, and directory structure during move |
| UC-05-08 | Display before/after status |
| UC-05-09 | If move fails, restore original state (atomic operation) |

### Example Output
```
$ sikil adopt legacy-workflow --from windsurf

Adopting skill 'legacy-workflow'...
  Source: ~/.codeium/windsurf/skills/legacy-workflow (unmanaged)

  ✓ Moved to ~/.sikil/repo/legacy-workflow/
  ✓ Created symlink at ~/.codeium/windsurf/skills/legacy-workflow

Done! Skill 'legacy-workflow' is now managed.
Tip: Use 'sikil sync legacy-workflow' to install to other agents.
```

---

## UC-06: Unmanage Skill

### Description
As a developer, I want to unmanage a skill so that it becomes a regular physical directory in agent locations.

### Command
```bash
sikil unmanage <skill-name>
sikil unmanage <skill-name> --agent claude-code
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-06-01 | Verify skill exists in `~/.sikil/repo/` |
| UC-06-02 | If `--agent` specified, unmanage only for that agent |
| UC-06-03 | If `--agent` not specified, unmanage for all agents that have symlinks |
| UC-06-04 | For each symlinked location: remove symlink, copy repo content to that location |
| UC-06-05 | After all symlinks converted, delete skill from `~/.sikil/repo/` |
| UC-06-06 | If only some agents unmanaged (via `--agent`), keep repo copy intact |
| UC-06-07 | Display list of converted locations |
| UC-06-08 | Confirm action before proceeding (interactive prompt) |
| UC-06-09 | Support `--yes` flag to skip confirmation |

### Example Output
```
$ sikil unmanage git-workflow

This will convert 'git-workflow' from managed to unmanaged.

Current symlinks:
  ~/.claude/skills/git-workflow
  ~/.codeium/windsurf/skills/git-workflow
  ~/.config/opencode/skill/git-workflow

Each symlink will be replaced with a physical copy.
The managed copy at ~/.sikil/repo/git-workflow/ will be deleted.

Continue? [y/N]: y

  ✓ Copied to ~/.claude/skills/git-workflow
  ✓ Copied to ~/.codeium/windsurf/skills/git-workflow
  ✓ Copied to ~/.config/opencode/skill/git-workflow
  ✓ Removed symlinks
  ✓ Deleted ~/.sikil/repo/git-workflow/

Done! Skill 'git-workflow' is now unmanaged (3 physical copies).
```

---

## UC-07: Remove Skill

### Description
As a developer, I want to remove a skill from specific agents or completely so that I can clean up unused skills.

### Command
```bash
sikil remove <skill-name> --agent claude-code
sikil remove <skill-name> --agent claude-code,windsurf
sikil remove <skill-name> --all
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-07-01 | `--agent` or `--all` is required (no default) |
| UC-07-02 | `--agent <name>` removes symlink from specified agent only |
| UC-07-03 | `--agent` with multiple agents: `--agent claude-code,windsurf` |
| UC-07-04 | `--all` removes all symlinks AND deletes from `~/.sikil/repo/` |
| UC-07-05 | For managed skills: remove symlink(s) |
| UC-07-06 | For unmanaged skills: delete physical directory |
| UC-07-07 | Confirm before deletion (interactive prompt) |
| UC-07-08 | Support `--yes` flag to skip confirmation |
| UC-07-09 | Display summary of removed locations |
| UC-07-10 | If skill not found, display helpful error |
| UC-07-11 | After `--agent` removal, if no symlinks remain, prompt to also remove from repo |

### Example Output
```
$ sikil remove old-skill --all

This will completely remove 'old-skill':
  - ~/.sikil/repo/old-skill/
  - ~/.claude/skills/old-skill (symlink)
  - ~/.codeium/windsurf/skills/old-skill (symlink)

Continue? [y/N]: y

  ✓ Removed symlink ~/.claude/skills/old-skill
  ✓ Removed symlink ~/.codeium/windsurf/skills/old-skill
  ✓ Deleted ~/.sikil/repo/old-skill/

Done! Skill 'old-skill' completely removed.
```

---

## UC-08: Sync Skill to Agents

### Description
As a developer, I want to sync a managed skill to agents that don't have it so that all my agents have consistent skills.

### Command
```bash
sikil sync <skill-name>
sikil sync --all
sikil sync <skill-name> --to claude-code,windsurf
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-08-01 | Verify skill exists in `~/.sikil/repo/` |
| UC-08-02 | Identify which agents are missing the skill (no symlink present) |
| UC-08-03 | Create symlinks to agents that don't have the skill |
| UC-08-04 | Skip agents that already have symlink to same repo location |
| UC-08-05 | If agent has unmanaged skill with same name, fail with error suggesting `adopt` first |
| UC-08-06 | `--all` syncs all managed skills to all agents |
| UC-08-07 | `--to` limits sync to specified agents only |
| UC-08-08 | Display which symlinks were created |
| UC-08-09 | Display which agents were skipped (already synced) |
| UC-08-10 | If skill already synced to all agents, display "already synced" message |

### Example Output
```
$ sikil sync git-workflow

Syncing 'git-workflow' to agents...

Already installed:
  ✓ claude-code
  ✓ windsurf

Installing:
  ✓ Symlinked to ~/.config/opencode/skill/git-workflow
  ✓ Symlinked to ~/.kilocode/skills/git-workflow
  ✓ Symlinked to ~/.config/agents/skills/git-workflow

Done! Skill 'git-workflow' synced to 3 new agents (5 total).
```

---

## UC-09: Validate Skill

### Description
As a developer, I want to validate a skill's structure and metadata before installation so that I can ensure it's properly formatted.

### Command
```bash
sikil validate ./path/to/skill
sikil validate <skill-name>
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-09-01 | Accept path to local directory or installed skill name |
| UC-09-02 | Check SKILL.md exists in directory |
| UC-09-03 | Check SKILL.md has valid YAML frontmatter (between `---` markers) |
| UC-09-04 | Validate required field: `name` (non-empty string) |
| UC-09-05 | Validate required field: `description` (non-empty string, 1-1024 chars) |
| UC-09-06 | Validate `name` format: lowercase, alphanumeric + hyphens, max 64 chars |
| UC-09-07 | Validate `name` constraints: no leading/trailing hyphens, no consecutive hyphens |
| UC-09-08 | Warn if optional fields missing: `version`, `author` |
| UC-09-09 | Check for common subdirectories: `scripts/`, `references/` (informational) |
| UC-09-10 | Display validation summary: passed/failed with details |
| UC-09-11 | Exit code 0 on success, non-zero on failure |

### Example Output
```
$ sikil validate ./my-skill

Validating skill at ./my-skill...

Required:
  ✓ SKILL.md exists
  ✓ YAML frontmatter valid
  ✓ name: "my-skill" (valid format)
  ✓ description: "My custom skill for..." (128 chars)

Optional:
  ✓ version: "1.0"
  ⚠ author: not specified
  ⚠ license: not specified

Structure:
  ✓ scripts/ directory found (2 files)
  ✓ references/ directory found (1 file)

Validation PASSED
```

---

## UC-10: Detect and Report Conflicts

### Description
As a developer, I want to see conflicts and duplicates across my skill installations so that I can identify issues to resolve.

### Command
```bash
sikil list --conflicts
sikil list --duplicates
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-10-01 | **duplicate-unmanaged**: Same skill name exists as physical directories in multiple locations |
| UC-10-02 | **duplicate-managed**: Same skill name, all installations are symlinks to same realpath (expected state, informational) |
| UC-10-03 | `--conflicts` shows only duplicate-unmanaged (problematic) |
| UC-10-04 | `--duplicates` shows both duplicate-unmanaged and duplicate-managed |
| UC-10-05 | For each conflict, display: skill name, all locations, managed/unmanaged status |
| UC-10-06 | Provide actionable recommendations (e.g., "use `sikil adopt` to consolidate") |
| UC-10-07 | Conflict detection uses YAML `name` as identity (not directory name) |
| UC-10-08 | Display total conflict count in summary |

### Example Output
```
$ sikil list --conflicts

Conflicts Detected (2):

1. git-workflow (duplicate-unmanaged)
   Locations:
     ~/.claude/skills/git-workflow (unmanaged, physical)
     ~/.codeium/windsurf/skills/git-workflow (unmanaged, physical)
     ~/.config/opencode/skill/git-workflow (unmanaged, physical)
   
   Recommendation: Use 'sikil adopt git-workflow --from claude-code' to manage,
                   then 'sikil sync git-workflow' to consolidate.

2. code-review (duplicate-unmanaged)
   Locations:
     ~/.claude/skills/code-review (unmanaged, physical)
     ./.claude/skills/code-review (unmanaged, physical, workspace)
   
   Recommendation: Workspace skill may intentionally override global.
                   Use 'sikil adopt' if you want to consolidate.

Summary: 2 conflicts found. Use 'sikil list' to see all skills.
```

---

## UC-11: Configure Agent Paths

### Description
As a developer, I want to override default agent paths so that I can accommodate non-standard installations.

### Command
```bash
sikil config
sikil config --edit
sikil config set agents.claude-code.global_path "~/custom/path"
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-11-01 | Configuration file location: `~/.sikil/config.toml` |
| UC-11-02 | `sikil config` displays current configuration |
| UC-11-03 | `sikil config --edit` opens config file in $EDITOR |
| UC-11-04 | `sikil config set <key> <value>` sets configuration value |
| UC-11-05 | Support overriding global_path per agent |
| UC-11-06 | Support overriding workspace_path per agent |
| UC-11-07 | Support disabling specific agents: `agents.kilo-code.enabled = false` |
| UC-11-08 | If config file doesn't exist, create with defaults on first run |
| UC-11-09 | Validate config file syntax on load |
| UC-11-10 | Display error if config is invalid |

### Config File Format
```toml
# ~/.sikil/config.toml

[agents.claude-code]
enabled = true
global_path = "~/.claude/skills"           # default
workspace_path = ".claude/skills"          # default

[agents.windsurf]
enabled = true
global_path = "~/.codeium/windsurf/skills"
workspace_path = ".windsurf/skills"

[agents.opencode]
enabled = true
global_path = "~/.config/opencode/skill"
workspace_path = ".opencode/skill"

[agents.kilo-code]
enabled = true
global_path = "~/.kilocode/skills"
workspace_path = ".kilocode/skills"

[agents.amp]
enabled = true
global_path = "~/.config/agents/skills"
workspace_path = ".agents/skills"
```

---

## UC-12: JSON Output for Scripting

### Description
As a developer, I want JSON output from commands so that I can integrate Sikil with other tools and scripts.

### Command
```bash
sikil list --json
sikil show <skill-name> --json
sikil list --conflicts --json
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-12-01 | `--json` flag available on: `list`, `show`, `validate` |
| UC-12-02 | JSON output goes to stdout |
| UC-12-03 | Human-readable messages go to stderr (when --json used) |
| UC-12-04 | JSON structure is consistent and documented |
| UC-12-05 | JSON includes all data shown in human-readable output |
| UC-12-06 | Exit codes remain consistent with non-JSON mode |

### Example JSON Output
```json
{
  "skills": [
    {
      "name": "git-workflow",
      "directory_name": "git-workflow",
      "description": "Git workflow automation",
      "version": "2.0",
      "managed": true,
      "repo_path": "~/.sikil/repo/git-workflow",
      "installations": [
        {
          "agent": "claude-code",
          "path": "~/.claude/skills/git-workflow",
          "scope": "global",
          "is_symlink": true,
          "symlink_target": "~/.sikil/repo/git-workflow"
        }
      ]
    }
  ],
  "summary": {
    "total": 17,
    "managed": 12,
    "unmanaged": 5,
    "conflicts": 2
  }
}
```

---

## UC-13: Help and Usage Information

### Description
As a developer, I want clear help information so that I can learn how to use Sikil effectively.

### Command
```bash
sikil --help
sikil <command> --help
sikil --version
```

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| UC-13-01 | `sikil --help` displays overview of all commands |
| UC-13-02 | `sikil <command> --help` displays detailed help for specific command |
| UC-13-03 | Help includes usage examples |
| UC-13-04 | Help includes option descriptions |
| UC-13-05 | `sikil --version` displays version number |
| UC-13-06 | Unknown command displays helpful error with suggestions |

---

## Non-Functional Requirements

> **Note**: NFR IDs match [PRD.md](PRD.md) for direct traceability.

### Performance

| ID | Requirement |
|----|-------------|
| NFR-01 | `sikil list` completes in < 500ms for typical installation (50 skills) |
| NFR-02 | Subsequent scans use cache for < 100ms response |
| NFR-03 | Cache invalidation based on directory mtime |
| NFR-04 | Binary size < 10MB |

### Reliability

| ID | Requirement |
|----|-------------|
| NFR-05 | All destructive operations (remove, unmanage) require confirmation |
| NFR-06 | Failed operations restore original state (atomic) |
| NFR-07 | Graceful handling of permission errors |
| NFR-08 | Graceful handling of missing directories |
| NFR-09 | Broken symlinks detected and reported as warnings |

### Compatibility

| ID | Requirement |
|----|-------------|
| NFR-10 | Supports macOS Intel |
| NFR-11 | Supports macOS Apple Silicon |
| NFR-12 | Supports Linux x86_64 |
| NFR-13 | Supports Linux aarch64 |
| NFR-14 | Windows explicitly not supported |
| NFR-15 | Minimum Git version 2.0+ |

### Usability

| ID | Requirement |
|----|-------------|
| NFR-16 | Colored output for terminal (respects NO_COLOR) |
| NFR-17 | Clear error messages with actionable guidance |
| NFR-18 | Progress indicators for long operations (git clone, large copies) |
| NFR-19 | First-time user success in < 5 minutes |

### Security

| ID | Requirement |
|----|-------------|
| NFR-20 | No execution of skill scripts (read-only operations) |
| NFR-21 | Path traversal prevention via skill name validation (`^[a-z0-9][a-z0-9_-]{0,63}$`) |
| NFR-22 | Git clone over HTTPS only (GitHub-only in v1) |
| NFR-23 | Reject skills containing symlinks (prevent escape attacks) |
| NFR-24 | Reject dangerous URL formats (`file://`, URLs starting with `-`) |
| NFR-25 | Config file hardening (1MB size limit, reject unknown fields) |
| NFR-26 | SQLite WAL mode for crash safety |

---

## Out of Scope

The following features are explicitly excluded from this version:

- Version comparison and upgrade recommendations
- Marketplace/registry integration (skills.io)
- Skill dependencies resolution
- Windows support
- Hot-reload triggering in agents
- Usage analytics/telemetry
- TUI (Terminal UI) interface
- Kilo Code mode-specific skill directories (`skills-code/`, `skills-architect/`)
- Backup/restore functionality
- Enterprise/team management features

---

## Glossary

| Term | Definition |
|------|------------|
| **Agent** | A coding AI assistant (Claude Code, Windsurf, etc.) |
| **Skill** | A directory containing SKILL.md with instructions for an agent |
| **SKILL.md** | Markdown file with YAML frontmatter defining skill metadata |
| **Global path** | Agent's skill directory in user's home (~/.claude/skills/) |
| **Workspace path** | Agent's skill directory in current project (.claude/skills/) |
| **Managed** | Skill stored in ~/.sikil/repo/ and symlinked to agents |
| **Unmanaged** | Skill existing as physical directory, not managed by Sikil |
| **Symlink** | Symbolic link pointing from agent directory to ~/.sikil/repo/ |
| **Realpath** | Resolved physical path after following symlinks |

---

**Document Version**: 1.2  
**Created**: January 16, 2026  
**Updated**: January 16, 2026  
**Status**: Ready for implementation

> **v1.1 Update**: Added security NFRs, updated UC-04 for GitHub-only URLs and symlink rejection.  
> **v1.2 Update**: Aligned NFR IDs with PRD.md (NF-xx → NFR-xx) for direct traceability.
