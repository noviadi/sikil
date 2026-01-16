# Agent Skills Implementation Comparison

## Agent-by-Agent Directory Structure

### Claude Code
```
Home Directory (~):
└── .claude/
    ├── skills/
    │   ├── git-workflow/
    │   │   ├── SKILL.md
    │   │   ├── scripts/
    │   │   └── references/
    │   ├── code-review/
    │   └── deploy/
    └── CLAUDE.md  (global instructions/memory)

Project Directory:
└── .claude/
    ├── skills/
    │   ├── project-conventions/
    │   └── ...
    └── claude.json  (project config)
```

**Characteristics:**
- Simple structure, consistent naming
- SKILL.md is case-sensitive
- Supports both global and project-level
- No mode-specific skills
- Located in `.claude/` directory consistently

---

### Windsurf (Codeium)
```
Home Directory (~):
└── .codeium/
    └── windsurf/
        └── skills/
            ├── code-review/
            │   ├── SKILL.md
            │   └── ...
            ├── testing/
            └── deploy/

Project Directory:
└── .windsurf/
    └── skills/
        ├── project-standards/
        └── ci-config/
```

**Characteristics:**
- Two separate paths (`.codeium/windsurf/` vs `.windsurf/`)
- No mode-specific skills (unlike Kilo)
- Has UI for creating skills
- Supports @mention invocation
- Clear global/workspace separation

---

### OpenCode
```
Project Directory (Priority):
├── .opencode/
│   └── skill/
│       ├── git-release/
│       │   └── SKILL.md
│       └── api-design/

Home Directory (~) - Multiple fallback paths:
├── ~/.config/opencode/skill/
│   ├── git-release/
│   └── ...
├── .claude/skills/          (Claude-compatible fallback)
└── ~/.claude/skills/        (Claude-compatible fallback)

Search priority:
1. .opencode/skill/<name>/SKILL.md
2. ~/.config/opencode/skill/<name>/SKILL.md  
3. .claude/skills/<name>/SKILL.md
4. ~/.claude/skills/<name>/SKILL.md
```

**Characteristics:**
- Most complex path system (4 locations)
- Fallback to Claude-compatible paths
- Project-level takes priority
- Can override per-agent in opencode.json
- Flexible but confusing

---

### Kilo Code
```
Home Directory (~):
└── .kilocode/
    ├── skills/                  (Generic, all modes)
    │   ├── git-workflow/
    │   └── database-design/
    ├── skills-code/             (Code mode only)
    │   ├── refactoring/
    │   └── typescript-patterns/
    └── skills-architect/        (Architect mode only)
        ├── system-design/
        └── scaling-patterns/

Project Directory:
└── .kilocode/
    ├── skills/
    │   └── project-conventions/
    ├── skills-code/
    │   └── linting-rules/
    └── skills-architect/
        └── architecture-patterns/
```

**Characteristics:**
- Mode-specific skills (unique feature)
- Three skill scopes: generic, code-mode, architect-mode
- Project skills override global
- Most complex override system
- VSCode reload required for new skills

---

### Amp Code
```
Project Directory (workspace):
└── .agents/
    └── skills/
        ├── database-config/
        │   └── SKILL.md
        └── security-setup/

Home Directory (~):
└── ~/.config/agents/
    └── skills/
        ├── git-workflow/
        └── deployment/
```

**Characteristics:**
- Simplest structure
- Only two locations (workspace + user)
- Clear separation
- Less documentation available
- Minimal community adoption yet

---

## Side-by-Side Comparison

| Factor | Claude Code | Windsurf | OpenCode | Kilo Code | Amp |
|--------|------------|----------|----------|-----------|-----|
| **Global path** | `~/.claude/skills` | `~/.codeium/windsurf/skills` | `~/.config/opencode/skill` | `~/.kilocode/skills` | `~/.config/agents/skills` |
| **Project path** | `.claude/skills` | `.windsurf/skills` | `.opencode/skill` | `.kilocode/skills` | `.agents/skills` |
| **Paths per agent** | 2 (global, project) | 2 (global, workspace) | 4 (with fallbacks) | 6+ (generic + modes) | 2 (user, workspace) |
| **Mode-specific** | ❌ No | ❌ No | ❌ No | ✅ Yes | ❌ No |
| **Override priority** | Project > Global | Workspace > Global | Explicit priority order | Project > Global, Mode-specific > Generic | Workspace > User |
| **Reload required** | VSCode reload | VSCode reload | Unknown | VSCode reload | Unknown |
| **Community adoption** | ⭐⭐⭐⭐⭐ (highest) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **Marketplace** | Planned | Via Codeium | Via OpenCode | Via Kilo | Via Amp |
| **Configuration file** | claude.json | windsurf settings | opencode.json | kilocode config | agents config |
| **UI support** | Dashboard planned | UI + CLI | CLI | CLI | CLI |
| **Skill invocation** | Auto + ask Claude | Auto + @mention | Auto | Auto | Auto |
| **Documentation** | ⭐⭐⭐⭐⭐ (best) | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |

---

## Critical Pain Points & Solutions

### Pain Point 1: Directory Discovery

**Problem:**
```bash
# As a user, I have no way to find all my skills
$ find ~ -name "SKILL.md"  # Brute force search
$ # Have to manually check 5+ locations per agent
```

**Solution with CLI tool:**
```bash
$ skills list
# Shows all 47 skills across all agents with locations
```

---

### Pain Point 2: Version Mismatches

**Problem:**
```
git-workflow installed in:
  ~/.claude/skills/git-workflow → v2.0 (Jan 15)
  ~/.codeium/windsurf/skills/git-workflow → v1.9 (Jan 12)
  ~/.config/opencode/skill/git-workflow → v1.5 (Jan 5)
  Project-1/.claude/skills/git-workflow → v1.5 (Jan 1)
  ~/.kilocode/skills/git-workflow → ???

Which one is used? Which should I update?
```

**Solution with CLI tool:**
```bash
$ skills show git-workflow
# Shows all versions, which is current, recommendations
$ skills upgrade git-workflow  # Updates all to latest
```

---

### Pain Point 3: Duplicate Management

**Problem:**
```bash
# Same skill in 4 places = 4x disk usage, 4x maintenance burden
# No tool to consolidate or deduplicate
# Manual deletion risks breaking agent configurations
```

**Solution:**
```bash
$ skills cleanup
# Detects duplicates, suggests consolidation
# Safe removal with rollback capability
```

---

### Pain Point 4: Cross-Agent Installation

**Problem:**
```bash
# Want to install a skill everywhere? Manual process:
mkdir -p ~/.claude/skills/new-skill
cp SKILL.md ~/.claude/skills/new-skill/

mkdir -p ~/.codeium/windsurf/skills/new-skill
cp SKILL.md ~/.codeium/windsurf/skills/new-skill/

mkdir -p ~/.config/opencode/skill/new-skill
cp SKILL.md ~/.config/opencode/skill/new-skill/

mkdir -p ~/.kilocode/skills/new-skill
cp SKILL.md ~/.kilocode/skills/new-skill/

mkdir -p ~/.config/agents/skills/new-skill
cp SKILL.md ~/.config/agents/skills/new-skill/

# And repeat for all projects...
# Error-prone, easy to miss an agent
```

**Solution:**
```bash
$ skills install new-skill
# Installs to all agents in one command
# Or select specific agents:
$ skills install new-skill --to claude-code,windsurf
```

---

### Pain Point 5: Project vs Global Conflicts

**Problem:**
```
Same skill exists in:
  ~/.claude/skills/git-workflow (global, v2.0)
  Project-1/.claude/skills/git-workflow (project, v1.5)

Which does Claude use? When? Priority unclear without docs.
```

**Solution:**
```bash
$ skills show git-workflow --conflicts
# Shows priority, recommends consolidation
$ skills resolve --consolidate
# Removes project version, uses global (or vice versa)
```

---

### Pain Point 6: Configuration Validation

**Problem:**
```bash
# Created new skill, unsure if it's valid
# Will agents recognize it?
# Any missing required fields?
```

**Solution:**
```bash
$ skills validate ./my-new-skill
# Checks YAML, required fields, file structure
# Suggests fixes for common errors
```

---

## Multi-Project Scenario

### Example: Developer with 3 Projects

```
Developer's machine:
├── Project-1 (Claude Code + Windsurf)
│   ├── .claude/skills/
│   │   ├── project-conventions/
│   │   └── pr-checklist/
│   └── .windsurf/skills/
│       └── deployment/
│
├── Project-2 (Windsurf + Kilo Code)
│   ├── .windsurf/skills/
│   │   └── performance-tuning/
│   └── .kilocode/skills/
│       ├── system-design/
│       └── refactoring/
│
└── Project-3 (All 5 agents)
    ├── .claude/skills/
    ├── .windsurf/skills/
    ├── .opencode/skill/
    ├── .kilocode/skills/
    └── .agents/skills/

Plus global skills:
├── ~/.claude/skills/
├── ~/.codeium/windsurf/skills/
├── ~/.config/opencode/skill/
├── ~/.kilocode/skills/
└── ~/.config/agents/skills/

Total skill directories: 5 global + 10 project = 15 locations!
```

**With CLI tool:**
```bash
$ cd Project-1
$ skills list --local
# Shows 4 project skills

$ cd ..
$ skills list
# Shows all 47 skills across all projects

$ skills sync Project-1 --to Project-2
# Copy Project-1 skills to Project-2

$ skills cleanup
# Shows duplicates across all 3 projects
```

---

## Enterprise Use Case: Organization Skills Management

### Problem: Enterprise Setup

```
Company: 50 developers, 5 coding agents, 200 skills

Challenges:
- Skill governance (approval process before use)
- Version control (pin certain skill versions)
- Audit trail (who installed what, when)
- Compliance (skills must pass security scan)
- Consistency (all teams use same version)
- Training (new skills roll-out to all developers)

Current solution: Manual, error-prone, no audit trail
```

### Solution: CLI Tool + Enterprise Features

```bash
# IT admin sets up organization skills
$ skills admin setup org=acme domain=acme.com

# Creates central repository
$ skills admin publish org/git-workflow:v2.0
✓ Skill validated
✓ Security scanned
✓ Published to acme.skills.io

# Developer installs from org registry
$ skills install acme/git-workflow
# Automatically gets v2.0 (pinned by IT)
# Audit log created

# Admin can mandate skills across team
$ skills admin mandate git-workflow code-review --team backend
✓ Installed to all 12 backend team members
✓ Audit trail created

# Tracking
$ skills admin report --usage
# Shows which skills used by whom, frequency, adoption rate
```

---

## Skill Dependency Example

### Current Problem

```
Scenario: Some skills depend on others

auth-flows skill needs:
  - base-templates skill (for HTML templates)
  - logging-config skill (for audit trails)

If developer installs only auth-flows:
  ❌ Agent gets reference errors
  ❌ auth-flows fails silently
  ❌ Developer blames the tool

No way to express dependencies today
```

### Solution with Enhanced CLI

```bash
# SKILL.md can specify dependencies
---
name: auth-flows
description: "..."
depends-on:
  - base-templates: ">=1.0"
  - logging-config: ">=2.0"
---

# When installing:
$ skills install auth-flows
Checking dependencies...
  ✓ base-templates (v1.5) available
  ✓ logging-config (v2.1) available
✓ All dependencies satisfied
✓ Installing auth-flows
```

---

## CLI vs TUI Decision Matrix

| Use Case | CLI | TUI | Recommendation |
|----------|-----|-----|-----------------|
| List all skills | ✅ Quick | 📊 Better overview | Both |
| Find specific skill | ✅ Search fast | 📊 Visual filtering | CLI for scripting, TUI for exploration |
| Install skill | ✅ Single command | 📊 Confirm selections | CLI for automation, TUI for safety |
| Detect conflicts | ❌ Text hard to parse | ✅ Visual indicators | TUI (colors, alignment matter) |
| Sync between agents | ✅ Can script | 📊 See preview | CLI for automation, TUI for review |
| Cleanup duplicates | ❌ Risk of mistakes | ✅ Safe selection | TUI (too dangerous via CLI) |
| Check configuration | ✅ JSON output | 📊 Structured view | CLI for CI/CD, TUI for review |
| Regular management | 🤝 Mix | 🤝 Mix | Both! Use CLI for scripts, TUI for interactive |

**Verdict:** Hybrid approach (both CLI + TUI) gives users best of both worlds.

---

## Performance Considerations

### Scanning Time (Initial)
```bash
$ time skills list
Scanning 5 agent directories...
  ~/.claude/skills/          (13 skills, 234 files)
  ~/.codeium/windsurf/skills/ (18 skills, 567 files)
  ~/.config/opencode/skill/   (9 skills, 145 files)
  ~/.kilocode/skills/         (8 skills, 189 files)
  ~/.config/agents/skills/    (5 skills, 89 files)
  + 10 project directories...

Time: ~150ms (with caching: ~10ms)

Result: Sub-second listing, acceptable for daily use
```

### Optimization Strategies
1. **Cache metadata** - SQLite database of SKILL.md frontmatter
2. **Incremental updates** - Only re-scan changed directories
3. **Parallel scanning** - Process directories concurrently
4. **Smart invalidation** - Detect file changes, invalidate cache

---

## Conclusion: Why This Tool is Needed

Today's fragmented skill ecosystem creates friction:

| Aspect | Current | With Tool |
|--------|---------|-----------|
| Discovering skills | 🔴 Manual exploration | 🟢 One command |
| Installing skills | 🔴 5+ manual steps per agent | 🟢 One command |
| Checking versions | 🔴 Remember where each is | 🟢 One command shows all |
| Finding duplicates | 🔴 Brute force searching | 🟢 Automatic detection |
| Updating skills | 🔴 Touch 5+ directories | 🟢 One command |
| Onboarding new dev | 🔴 2-4 hours manual setup | 🟢 <5 minutes automated |
| Team management | 🔴 Impossible to track | 🟢 Full audit trail |
| Cleanup & maintenance | 🔴 Risky, error-prone | 🟢 Safe, automated |

**This tool is a force multiplier for anyone using multiple coding agents.**