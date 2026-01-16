# Agent Skills Manager - Brainstorm & Design

## Executive Summary

The problem is clear: **fragmented skill installation across 5+ agents creates a management nightmare**. Each agent (Claude Code, Windsurf, OpenCode, Kilo Code, Amp) stores skills in different directories with overlapping but incompatible path structures. A developer working with multiple agents quickly loses visibility into:

- What skills are installed where
- Whether duplicates exist with version conflicts
- Which agents have access to which skills
- Overall disk usage and inventory

This document outlines a **CLI-first, optionally TUI-enhanced tool** to bring visibility and control to this chaos.

---

## Problem Analysis

### Current State
```
Typical developer's machine:
~/.claude/skills/
  ├── git-workflow/
  ├── code-review/
  └── deploy/

~/.config/opencode/skill/
  ├── git-workflow/          ← Different location, same skill?
  ├── api-design/
  └── ...

~/.codeium/windsurf/skills/
  ├── code-review/           ← Different location, same skill again?
  ├── refactor/
  └── ...

~/.kilocode/skills/
  └── git-workflow/          ← Third copy! Which version?

.claude/skills/             ← Project-level Claude skills
Project-1/
  ├── git-workflow/
  └── project-rules/

Project-2/
  .opencode/skill/
  ├── git-workflow/          ← Fourth copy! Outdated?
  └── ...

Project-3/
  .windsurf/skills/
  └── ...

Result: Chaos, redundancy, maintenance headaches
```

### Why This Matters

1. **Redundancy**: Same skill stored 3-4 times consumes disk space and mental effort
2. **Version mismatch**: git-workflow might be v1.0 in Claude, v0.9 in OpenCode, v2.0 in Windsurf
3. **Inconsistency**: Agent improvements in one skill aren't reflected in copies elsewhere
4. **Discovery**: How do you know what skills you have available across all agents?
5. **Maintenance**: Updating a skill means touching 4+ directories, hoping you don't miss one
6. **Learning curve**: New developers don't know where to look or how to organize

---

## Solution Architecture

### Core Concept: Multi-Agent Skill Inventory & Management

A unified tool that:
- **Discovers** all skills across all agent installations
- **Catalogs** them in a queryable database
- **Synchronizes** them across agents
- **Validates** their structure
- **Reports** on status and conflicts

### Design Principle: CLI-First with Optional TUI

**Primary Interface**: Command-line tool (scripts, automation, pipelines)  
**Secondary Interface**: Interactive terminal UI (discovery, exploration, troubleshooting)  
**Tertiary Interface**: JSON output (integration with other tools)

---

## Feature Set

### Tier 1: Discovery & Inventory

#### Command: `skills list`
```bash
$ skills list

Found 47 skills across 5 agents:

PERSONAL SKILLS (Global)
  git-workflow          (2 versions) └─ ~/.claude/skills, ~/.codeium/windsurf/skills
  code-review           (1 version)  └─ ~/.claude/skills
  deploy                (3 versions) └─ ~./claude/skills, ~/.config/opencode/skill, ~/.kilocode/skills
  testing               (1 version)  └─ ~/.codeium/windsurf/skills
  api-design            (1 version)  └─ ~/.config/opencode/skill
  auth-flows            (1 version)  └─ ~/.kilocode/skills
  refactor              (1 version)  └─ ~/.codeium/windsurf/skills

⚠ DUPLICATES DETECTED (7 skills in multiple locations)

PROJECT SKILLS
  Project-1/.claude/skills/ (3 skills)
    ├─ git-workflow [CONFLICTS with global versions]
    ├─ project-conventions
    └─ pr-checklist
  Project-2/.opencode/skill/ (2 skills)
    ├─ api-design [OUTDATED - global is v2.0, project is v1.5]
    └─ db-migration

Total unique skills: 47
Total disk usage: 124 MB
Agents scanned: 5
  ✓ Claude Code      (13 skills)
  ✓ Windsurf         (18 skills)
  ✓ OpenCode         (9 skills)
  ✓ Kilo Code        (8 skills)
  ✓ Amp Code         (5 skills)
```

#### Command: `skills list --agent windsurf`
```bash
$ skills list --agent windsurf

Windsurf Skills (18 total):
  ├─ api-design
  ├─ auth-flows
  ├─ code-review
  ├─ code-style
  ├─ deploy
  ├─ docker-setup
  ├─ error-handling
  ├─ git-workflow
  ├─ logging-config
  ├─ performance-tuning
  ├─ refactor
  ├─ security-audit
  ├─ testing
  ├─ typescript-patterns
  └─ ...

Locations:
  Global: ~/.codeium/windsurf/skills (14 skills)
  Workspace: .windsurf/skills (4 skills)

Conflicts: None
```

#### Command: `skills list --duplicates`
```bash
$ skills list --duplicates

7 Skills with Multiple Installations:

1. git-workflow (4 installations)
   ├─ v2.0 @ ~/.claude/skills/git-workflow
   ├─ v1.9 @ ~/.codeium/windsurf/skills/git-workflow
   ├─ v1.5 @ ~/.config/opencode/skill/git-workflow
   └─ v1.5 @ Project-1/.claude/skills/git-workflow [OUTDATED]

2. code-review (3 installations)
   ├─ v1.2 @ ~/.claude/skills/code-review
   ├─ v1.2 @ ~/.codeium/windsurf/skills/code-review
   └─ v1.1 @ ~/.config/opencode/skill/code-review [OUTDATED]

3. deploy (3 installations)
   ├─ v2.0 @ ~/.claude/skills/deploy
   ├─ v1.8 @ ~/.kilocode/skills/deploy
   └─ v1.6 @ ~/.config/opencode/skill/deploy [OUTDATED]

Recommendation: Run 'skills cleanup' to standardize versions
```

#### Command: `skills show git-workflow`
```bash
$ skills show git-workflow

Skill: git-workflow
Description: Workflow instructions for managing git repositories with proper commit conventions

Installations:
  1. ~/.claude/skills/git-workflow
     Version: 2.0
     Size: 124 KB
     Modified: 2024-01-15
     Agent: Claude Code
     Status: CURRENT

  2. ~/.codeium/windsurf/skills/git-workflow
     Version: 1.9
     Size: 118 KB
     Modified: 2024-01-12
     Agent: Windsurf
     Status: OUTDATED (update available)

  3. ~/.config/opencode/skill/git-workflow
     Version: 1.5
     Size: 95 KB
     Modified: 2024-01-05
     Agent: OpenCode
     Status: OUTDATED (update available)

  4. Project-1/.claude/skills/git-workflow
     Version: 1.5
     Size: 88 KB
     Modified: 2024-01-01
     Agent: Claude Code (project-level)
     Status: OUTDATED & CONFLICTS with global

Files in skill:
  ├─ SKILL.md (3.2 KB)
  ├─ scripts/
  │  ├─ commit-template.sh
  │  └─ branch-naming.py
  └─ references/
     ├─ WORKFLOW.md
     └─ EXAMPLES.md
```

### Tier 2: Installation & Synchronization

#### Command: `skills install`
```bash
# From skills registry
$ skills install anthropic/git-workflow
Installing git-workflow v2.0...
  ✓ Downloaded from skills.io
  ✓ Validated SKILL.md format
  ✓ Installed to ~/.claude/skills/git-workflow
  ✓ Installed to ~/.codeium/windsurf/skills/git-workflow
  ✓ Installed to ~/.config/opencode/skill/git-workflow
  ✓ Installed to ~/.kilocode/skills/git-workflow
  ✓ Installed to ~/.config/agents/skills/git-workflow
Done! 5 agents updated.

# To specific agents
$ skills install deploy --to claude-code,windsurf
Installing deploy...
  ✓ Installed to ~/.claude/skills/deploy
  ✓ Installed to ~/.codeium/windsurf/skills/deploy
Done! 2 agents updated.

# From GitHub
$ skills install github.com/org/my-custom-skill --branch main
$ skills install ./local-skill --copy

# From local directory
$ skills install ./my-new-skill
  Validating ./my-new-skill...
  ✓ SKILL.md found and valid
  ✓ Name: my-new-skill
  ✓ Required fields present
  Where to install?
  1) All agents (default)
  2) Claude Code only
  3) Windsurf only
  4) Skip some agents
  Choice: 1
  ✓ Installed across all agents
```

#### Command: `skills sync`
```bash
# Make all agents match Claude Code
$ skills sync --from claude-code --to all

Synchronizing from Claude Code to all agents...
  Claude Code has 13 skills, other agents vary

Changes:
  Windsurf:    add 3 skills (git-workflow, deploy, auth-flows)
  OpenCode:    add 5 skills, remove 2 outdated
  Kilo Code:   add 2 skills, update 1 version
  Amp:         add 4 skills

  Confirm? (y/n): y
  ✓ Synced 13 skills to 4 agents
  ✓ Total updates: 14 installations

# Sync only specific skill
$ skills sync git-workflow --to all --force
✓ Updated git-workflow (v2.0) across 5 agents

# Sync project skills to global
$ skills sync --from-project ./Project-1 --to global
  Found 3 skills in Project-1/.claude/skills/
  Copy to global ~/.claude/skills/?
  ✓ Copied 3 skills
```

#### Command: `skills remove`
```bash
$ skills remove old-skill

Found old-skill in:
  ~/.claude/skills/old-skill
  ~/.codeium/windsurf/skills/old-skill
  ~/.config/opencode/skill/old-skill

Remove from all? (y/n): y
  ✓ Removed from Claude Code
  ✓ Removed from Windsurf
  ✓ Removed from OpenCode
Done! Freed 84 KB

# Remove from specific agent
$ skills remove old-skill --from windsurf
  ✓ Removed from Windsurf only
```

### Tier 3: Validation & Quality

#### Command: `skills validate`
```bash
$ skills validate ./my-skill

Validating my-skill...
  ✓ Directory structure correct
  ✓ SKILL.md exists and readable
  ✓ YAML frontmatter valid
  ✓ Required fields:
    ✓ name: my-skill (64 chars, valid format)
    ✓ description: "..." (256 chars, descriptive)
  ✓ Optional fields:
    ✓ version: 1.0
    ✓ author: John Doe
    ✓ license: MIT
    ✓ compatibility: "Python 3.8+"
  ✓ Files referenced in content exist
  ✓ No broken links in references/
  ✓ Scripts are executable
  ⚠ Warning: scripts/setup.sh uses hardcoded paths (should use {baseDir})

Validation: PASS
Ready to publish or install.
```

#### Command: `skills lint`
```bash
$ skills lint ~/.claude/skills

Linting 13 skills...

ERRORS:
  api-design: Missing description field (required)
  old-deploy: Name contains uppercase (should be: old-deploy) [FIXABLE]

WARNINGS:
  code-review: Version field missing (recommended)
  refactor: 2.5 MB is large, consider splitting
  testing: Scripts use hardcoded absolute paths (should use {baseDir})

Found 2 errors, 3 warnings in 13 skills
Run 'skills lint --fix' to auto-correct fixable issues
```

### Tier 4: Project Management

#### Command: `skills config`
```bash
$ skills config

Agent Paths Configuration:

Claude Code:
  Global: ~/.claude/skills ✓
  Project: .claude/skills ✓
  Managed: /etc/claude/skills (not found)

Windsurf:
  Global: ~/.codeium/windsurf/skills ✓
  Workspace: .windsurf/skills ✓

OpenCode:
  Priority:
    1. .opencode/skill (found)
    2. ~/.config/opencode/skill (found)
    3. .claude/skills (found)
    4. ~/.claude/skills (found)

Kilo Code:
  Global: ~/.kilocode/skills ✓
  Project: .kilocode/skills ✓
  Modes: skills-code/, skills-architect/ (found)

Amp Code:
  Workspace: .agents/skills ✓
  User: ~/.config/agents/skills ✓

Cache:
  Location: ~/.sikil/cache.db
  Size: 2.1 MB
  Skills indexed: 47
  Last update: 2 minutes ago

# Edit config
$ skills config --edit

# Reset to defaults
$ skills config --reset
```

#### Command: `skills cleanup`
```bash
$ skills cleanup

Analyzing skill installations...

Findings:
  - 7 duplicate skills with version mismatches
  - 3 orphaned skills (not used by any agent)
  - 45 MB disk usage (24 MB could be freed)
  - 1 skill has conflicts (git-workflow in global and project)

Cleanup Options:
  1) Auto-reconcile versions (keeps newest)
  2) Remove orphaned skills
  3) Consolidate duplicates (keep global, remove project copies)
  4) Review each conflict manually
  5) All of the above

Choice: 5

Cleanup plan:
  ├─ Update 3 outdated versions
  ├─ Remove 3 orphaned skills
  ├─ Move 2 project skills to global (consolidate)
  └─ Resolve 1 global/project conflict

  Proceed? (y/n): y
  ✓ Cleanup complete
  ✓ Freed 45 MB
  ✓ All skills now in optimal state
```

### Tier 5: Advanced Features

#### Command: `skills marketplace search`
```bash
$ skills marketplace search git

Searching skills.io marketplace...

Found 12 git-related skills:

1. git-workflow (anthropic/git-workflow)
   Description: Complete workflow for git repository management
   Author: Anthropic
   Downloads: 5.2K
   ⭐ 4.8/5 (124 reviews)
   Tags: git, workflow, conventions
   Status: INSTALLED (v2.0) ✓

2. git-commit-templates (community/git-commit)
   Description: Pre-formatted commit message templates
   Author: Community
   Downloads: 1.2K
   ⭐ 4.2/5 (45 reviews)
   Tags: git, commits, templates
   Status: Not installed

3. git-security-scanner (org/git-security)
   Description: Scan commits for secrets and security issues
   Downloads: 856
   ⭐ 4.6/5 (12 reviews)
   Status: Not installed

# Install from search
$ skills marketplace install git-security-scanner
$ skills marketplace install anthropic/git-workflow --upgrade
```

#### Command: `skills usage` (Analytics)
```bash
$ skills usage

Skills Usage Analytics:

Most Used Skills (by agent invocations):
  1. code-review        (847 uses)  └─ Claude Code (612), Windsurf (235)
  2. git-workflow       (563 uses)  └─ All agents
  3. testing            (421 uses)  └─ Primarily Windsurf
  4. deploy             (312 uses)  └─ Claude Code, Windsurf

Unused Skills (>30 days):
  - old-deploy          (last used: 63 days ago)
  - experimental-ai     (last used: 127 days ago)
  - beta-feature        (never used)

Recommendation: Consider removing unused skills to reduce clutter
  Run 'skills cleanup --unused' to remove

# Export statistics
$ skills usage --export json --output stats.json
```

---

## Interface Examples

### CLI Command Structure

```bash
# Discovery
skills list                           # List all skills
skills list --agent windsurf          # Filter by agent
skills list --duplicates              # Show conflicts
skills list --unused                  # Find unused
skills show skill-name                # Details on one
skills search "keyword"               # Search by name/desc

# Installation
skills install skill-name             # Install everywhere
skills install skill-name --to agent1,agent2
skills install ./local-skill          # From directory
skills install github.com/org/skill   # From GitHub

# Management
skills sync --from agent1 --to agent2 # Synchronize
skills remove skill-name              # Delete
skills remove skill-name --from agent # Delete from specific agent
skills cleanup                        # Auto-fix issues
skills validate ./skill               # Check format

# Configuration
skills config                         # Show current paths
skills config --edit                  # Modify paths
skills config --reset                 # Restore defaults

# Advanced
skills lint                           # Quality check
skills marketplace search "keyword"   # Search registry
skills marketplace install skill      # Install from registry
skills usage                          # Analytics
skills backup                         # Create backup
skills restore backup.tar.gz          # Restore backup

# Options
-v, --verbose                         # More output
-q, --quiet                           # Less output
--json                                # JSON output
--help                                # Help
--version                             # Version
```

### TUI Dashboard

```
╔════════════════════════════════════════════════════════════════════╗
║                  AGENT SKILLS MANAGER DASHBOARD                   ║
╠════════════════════════════════════════════════════════════════════╣
║                                                                    ║
║  QUICK STATS                  AGENTS                  SKILLS      ║
║  ────────────                 ──────                  ──────      ║
║  Total Skills: 47             □ Claude Code    (13)   Updated: 2d ║
║  Installed: 5 agents          □ Windsurf       (18)   Unused: 3   ║
║  Duplicates: 7                □ OpenCode        (9)   Disk: 124MB ║
║  Conflicts: 1                 □ Kilo Code       (8)               ║
║  Issues: 4                    □ Amp Code        (5)               ║
║                                                                    ║
║  ┌──────────────────────────────────────────────────────────────┐ ║
║  │ 🔍 Search skills...                                     [Filter]│ ║
║  └──────────────────────────────────────────────────────────────┘ ║
║                                                                    ║
║  INSTALLED SKILLS                          ACTIONS                ║
║  ─────────────────────────────────────┐    ───────                ║
║  > api-design               [stable]   │    [I]nstall             ║
║    auth-flows               [stable]   │    [V]iew details        ║
║  > code-review              [stable]   │    [S]ync across agents  ║
║    code-style               [stable]   │    [U]pgrade version     ║
║  > DEPLOY                [3 versions] │    [D]elete              ║
║    git-workflow             [outdated] │    [C]lean duplicates    ║
║    logging-config           [missing]  │    [M]arkplace search    ║
║    performance-tuning       [stable]   │    [R]efresh             ║
║    refactor                 [stable]   │    [Q]uit                ║
║    security-audit           [stable]   │                          ║
║    testing                  [stable]   │                          ║
║                                        │                          ║
║  ⚠ ISSUES: 4 items need attention      │                          ║
║    • git-workflow (outdated v1.9)      │                          ║
║    • DEPLOY (3 versions, pick one)     │                          ║
║    • old-deploy (unused, delete?)      │                          ║
║    • logging-config (not in Amp)       │                          ║
║                                                                    ║
╠════════════════════════════════════════════════════════════════════╣
║ [S]earch [I]nstall [V]iew [D]elete [C]lean [M]arketplace [?]Help  ║
╚════════════════════════════════════════════════════════════════════╝
```

### Interactive Selection Menu

```
Select skill to manage:
  1) api-design
  2) auth-flows
  3) code-review
  4) code-style
> 5) DEPLOY                           ← 3 conflicting versions!
  6) git-workflow                     ← Update available
  7) logging-config                   ← Not installed in Amp
  8) performance-tuning
  9) refactor
  10) security-audit
  11) testing

Or [S]earch, [C]reate new, [B]ack, [Q]uit
Choice: 5

╔════════════════════════════════════════════════════════════════════╗
║                           DEPLOY SKILL                            ║
╠════════════════════════════════════════════════════════════════════╣
║                                                                    ║
║ Name: deploy                                                       ║
║ Description: Deploy application to staging/production with        ║
║              validation and rollback support                       ║
║ Author: Anthropic Team                                             ║
║ License: MIT                                                       ║
║                                                                    ║
║ INSTALLATIONS (3 versions detected - CONFLICT)                     ║
║ ───────────────────────────────────────────────                   ║
║ □ v2.0  @ ~/.claude/skills/deploy           [LATEST]              ║
║ ☑ v1.8  @ ~/.kilocode/skills/deploy         [OUTDATED]            ║
║ ☑ v1.6  @ ~/.config/opencode/skill/deploy   [OUTDATED]            ║
║                                                                    ║
║ AGENTS                                                             ║
║ ──────                                                             ║
║ ✓ Claude Code       (v2.0)                                         ║
║ ✓ Windsurf          (v2.0) shared via ~/.codeium/...              ║
║ ✓ Kilo Code         (v1.8) [1 version behind]                     ║
║ ✓ OpenCode          (v1.6) [2 versions behind]                    ║
║ ✗ Amp Code          (not installed)                                ║
║                                                                    ║
║ RESOLUTION OPTIONS                                                 ║
║ ──────────────────                                                 ║
║ [U]pgrade all to v2.0                                              ║
║ [D]elete old versions                                              ║
║ [I]nstall to Amp Code                                              ║
║ [R]evert to v1.8 (all agents)                                      ║
║ [V]iew skill details                                               ║
║ [B]ack                                                             ║
║                                                                    ║
║ Recommended: [U]pgrade to ensure consistency                       ║
║                                                                    ║
╚════════════════════════════════════════════════════════════════════╝
```

---

## Implementation Roadmap

### Phase 1: MVP (2 weeks)
**Goal**: CLI tool with discovery and basic inventory

- [ ] Scan all 5 agent skill directories
- [ ] Parse SKILL.md metadata (YAML frontmatter)
- [ ] Build in-memory skill database
- [ ] Implement `skills list` command
- [ ] Implement `skills show <name>` command
- [ ] Detect duplicates and version mismatches
- [ ] JSON output support
- [ ] Configuration file for agent paths

**Deliverable**: `skills` binary, works on Linux/Mac/Windows

### Phase 2: Core Management (2 weeks)
**Goal**: CLI commands for installation and sync

- [ ] `skills install` (multiple sources)
- [ ] `skills sync` (cross-agent synchronization)
- [ ] `skills remove` command
- [ ] `skills validate` (SKILL.md validation)
- [ ] Basic conflict detection
- [ ] Progress indicators and confirmations

### Phase 3: UX Polish (1 week)
**Goal**: Make it pleasant to use

- [ ] Shell completions (bash, zsh, fish)
- [ ] Colored output and better formatting
- [ ] `--verbose` and `--quiet` modes
- [ ] Error messages and recovery suggestions
- [ ] Performance optimization (caching)

### Phase 4: Optional TUI (2 weeks)
**Goal**: Interactive terminal interface

- [ ] Dashboard view
- [ ] Keyboard navigation
- [ ] Real-time conflict detection
- [ ] Interactive skill selection
- [ ] Side-by-side agent comparison

### Phase 5: Advanced (3 weeks)
**Goal**: Professional features

- [ ] Marketplace integration
- [ ] Skill versioning and upgrades
- [ ] Usage analytics
- [ ] Automated cleanup
- [ ] Backup/restore
- [ ] CI/CD integration

---

## Technology Choices

### Recommended: Rust + Ratatui (TUI)

**Why Rust:**
- Single binary, no runtime dependencies
- Performance for scanning large directories
- Type safety prevents bugs
- Excellent cross-platform support
- Growing TUI ecosystem

**Core Dependencies:**
```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }  # CLI parsing
tokio = { version = "1.35", features = ["full"] }  # Async I/O
serde = { version = "1.0", features = ["derive"] }  # Serialization
serde_yaml = "0.9"                                   # YAML parsing
serde_json = "1.0"                                   # JSON support
ratatui = "0.26"                                     # TUI framework
crossterm = "0.27"                                   # Terminal control
regex = "1.10"                                       # Pattern matching
walkdir = "2.4"                                      # Directory traversal
rusqlite = "0.31"                                    # SQLite caching
reqwest = "0.11"                                     # HTTP client (marketplace)
```

**Project Structure:**
```
skills-manager/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── commands/             # CLI command implementations
│   │   ├── list.rs
│   │   ├── install.rs
│   │   ├── sync.rs
│   │   ├── remove.rs
│   │   ├── validate.rs
│   │   └── mod.rs
│   ├── core/                 # Core logic
│   │   ├── skill.rs         # Skill struct and parsing
│   │   ├── agent.rs         # Agent configuration
│   │   ├── scanner.rs       # Directory scanning
│   │   ├── database.rs      # Caching and indexing
│   │   └── mod.rs
│   ├── ui/                   # TUI components
│   │   ├── dashboard.rs
│   │   ├── components/
│   │   └── mod.rs
│   ├── config.rs            # Configuration management
│   ├── utils.rs             # Utilities
│   └── errors.rs            # Error types
├── tests/
├── Cargo.toml
├── README.md
└── docs/
```

---

## Market Fit & Community

### Target Users
1. **Multi-agent developers** using 2+ coding agents
2. **Teams** managing skills across organization
3. **Tool maintainers** creating skills for distribution
4. **Enterprises** deploying skills at scale

### Community Opportunities
- Plugin for each agent (VS Code extension, Windsurf plugin, etc.)
- Skills marketplace integration
- GitHub Actions for CI/CD
- Community skill registry
- Educational use (teaching agent skills)

### Where to Promote
- Agent Skills specification communities
- Reddit r/AI_Agents, r/ClaudeCode
- GitHub + trending
- Dev.to, HackerNews
- Discussions on agent product forums

---

## Success Metrics

### Phase 1-2 (MVP + Core)
- [ ] Can discover all skills across all agents
- [ ] Can list skills with filters
- [ ] Can install/sync between agents
- [ ] Handles all 5 major agents
- [ ] Works on Windows, Mac, Linux

### Phase 3 (Polish)
- [ ] Sub-second list performance
- [ ] Shell completion support
- [ ] 0 manual CLI errors (good help/error messages)
- [ ] First-time user can manage skills in <5 minutes

### Phase 4 (TUI)
- [ ] Dashboard loads in <2 seconds
- [ ] Keyboard navigation intuitive
- [ ] Visual conflict detection clear
- [ ] Users prefer TUI over CLI for exploration

### Phase 5 (Advanced)
- [ ] Marketplace integration working
- [ ] Analytics useful for cleanup decisions
- [ ] Backup/restore tested and reliable

---

## Alternative Approaches Considered

### Option A: Web UI Dashboard
- **Pros**: Works everywhere, pretty
- **Cons**: Requires server/frontend, slower for CLI users, doesn't fit workflow
- **Decision**: CLI-first better for developer ergonomics

### Option B: VSCode Extension
- **Pros**: Native UI, integrated with one agent
- **Cons**: Doesn't solve cross-agent problem, limited visibility
- **Decision**: Would be extension #5 once CLI exists, not primary tool

### Option C: Language-Specific (Python)
- **Pros**: Easier scripting, familiar to some users
- **Cons**: Slower startup time, runtime dependency overhead
- **Decision**: Rust better for CLI tool distribution

---

## Open Questions for Refinement

1. **Scope of skill templates**: Should tool generate new skills?
2. **Enterprise integration**: Support Okta/SAML/LDAP for managed deployments?
3. **Conflict resolution**: Should we auto-pick "newest" or always ask?
4. **Caching strategy**: How long to cache before re-scanning?
5. **Marketplace**: Build proprietary or integrate with existing (skills.io)?
6. **Analytics**: Require opt-in tracking or fully anonymous?
7. **Plugins**: Support custom agent types beyond the 5?
8. **Skill dependencies**: Resolution strategy when skills depend on each other?

---

## Conclusion

**Agent Skills are the future of agent customization**, but today's fragmented installations create friction. This tool bridges that gap, bringing:

✅ **Visibility** - Know what you have and where  
✅ **Control** - Manage across multiple agents effortlessly  
✅ **Quality** - Catch conflicts and outdated versions  
✅ **Simplicity** - One command to rule them all  

The MVP can ship in 2 weeks with core value; full feature set in 8 weeks would be production-ready for teams and enterprises.
