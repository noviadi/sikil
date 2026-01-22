# Agent Skills Management Research

## What is Agent Skills?

Agent Skills is an **open format and documentation set** created by Anthropic for describing, sharing, and discovering agent capabilities. Unlike tools (which are executable functions), skills are **procedural knowledge packages** that contain:

- Instructions (SKILL.md with YAML frontmatter)
- Scripts (optional, executable code)
- References (optional, documentation files)
- Assets (optional, templates and resources)

### Key Design Principle: Progressive Disclosure
- **Metadata** (~100 tokens) loaded at startup: `name`, `description`
- **Instructions** (<5000 tokens) loaded when skill is activated
- **Resources** loaded only when needed

This keeps context windows lean while providing detailed info on demand.

---

## Agent Support Status

### Primary Agents Supporting Skills

| Agent | Skill Location | Status | Notes |
|-------|----------------|--------|-------|
| **Claude Code** | `~/.claude/skills/` (personal), `.claude/skills/` (project) | ✅ Full Support | Anthropic's reference implementation |
| **Windsurf** | `~/.codeium/windsurf/skills/` (global), `.windsurf/skills/` (workspace) | ✅ Full Support | UI + manual creation, auto-invoke + @-mention |
| **OpenCode** | `~/.config/opencode/skill/` (global), `.opencode/skill/` (project), `.claude/skills/` (Claude-compat) | ✅ Full Support | Multiple search paths, Claude compatibility |
| **Kilo Code** | `~/.kilocode/skills/` (global), `.kilocode/skills/` (project), mode-specific folders | ✅ Full Support | Mode-specific skills (`skills-code/`, `skills-architect/`) |
| **Amp Code** | `.agents/skills/` (workspace), `~/.config/agents/skills/` (user-level) | ✅ Full Support | Lazy-loading for tool use performance |

### Other Notable Agents
- **GitHub Copilot** - Exploring adoption
- **Cursor** - Community adoption reported
- **Letta, goose, VS Code** - Limited/community support
- **Notably absent**: OpenAI's tools (not adopting Agent Skills standard)

---

## The Core Problem: Directory Chaos

### Issue: Fragmented Installation Paths

Each agent stores skills in different locations:

```
User's Machine:
├── ~/.claude/skills/                    # Claude Code (personal)
├── ~/.config/opencode/skill/            # OpenCode (global)
├── ~/.codeium/windsurf/skills/          # Windsurf (global)
├── ~/.kilocode/skills/                  # Kilo Code (global)
├── ~/.config/agents/skills/             # Amp (user-level)
├── Project-1/.claude/skills/            # Claude Code (project)
├── Project-1/.opencode/skill/           # OpenCode (project)
├── Project-1/.windsurf/skills/          # Windsurf (workspace)
├── Project-1/.kilocode/skills/          # Kilo Code (project)
├── Project-1/.agents/skills/            # Amp (workspace)
├── Project-2/.claude/skills/
├── Project-2/.opencode/skill/
└── ...multiple projects with scattered skills
```

**Result:**
- 🔴 No unified visibility of what skills are installed where
- 🔴 Duplicate skills across multiple agents
- 🔴 Inconsistent versions (same skill, different versions in different agents)
- 🔴 No easy way to sync skills across agents
- 🔴 Manual management becomes error-prone
- 🔴 Hard to identify unused or deprecated skills

### Additional Complexity Layers

1. **Multiple Search Paths Per Agent**
   - Claude Code: `~/.claude/`, `.claude/`, managed settings, plugin-provided
   - OpenCode: 4 different locations with fallback priority
   - Kilo Code: Global generic + mode-specific + project-level combinations

2. **Configuration Priority Systems**
   - Project skills override global skills
   - Mode-specific override generic
   - Some agents have "managed" level (enterprise deployment)
   - Conflicts are silent and hard to debug

3. **Skills Require Hot-Reload**
   - Most agents require VSCode reload to pick up new skills
   - Changes to SKILL.md files sometimes don't propagate
   - Hard to verify if a skill actually loaded

4. **CLI Inconsistency**
   - No standardized way to list installed skills across agents
   - No CLI tool to search for skills by name/description
   - No bulk install/uninstall operations
   - No skill validation before deployment

---

## Agent Skills Specification Details

### SKILL.md Structure
```yaml
---
name: skill-name              # Required: lowercase, alphanumeric + hyphens
description: "What it does"   # Required: 1-1024 characters
version: 1.0                  # Optional
author: name                  # Optional
license: MIT                  # Optional
compatibility: "requirements" # Optional
---

# Instructions
Your detailed instructions here...

## Using This Skill
Examples and usage patterns...
```

### Directory Structure
```
my-skill/
├── SKILL.md              # Required: metadata + instructions
├── scripts/              # Optional: executable code (Python, Bash)
├── references/           # Optional: detailed docs (REFERENCE.md, FORMS.md)
└── assets/               # Optional: templates, images, data files
```

### Name Constraints
- Max 64 characters
- Lowercase letters, numbers, hyphens only
- Cannot start/end with hyphen
- No consecutive hyphens

---

## Existing Solutions & Gaps

### Reddit/Community Mentions
- One Reddit user created script to install skills across multiple agents
- GitHub repo mentioned: `skillai/A-Agent-Skills` (20 popular skills)
- Main complaint: "Each agent uses different directories"

### What Doesn't Exist Yet
- ❌ Unified CLI to list all skills across all agents
- ❌ Skill sync/duplication detection
- ❌ Cross-agent skill installer
- ❌ Skill validation tool (check SKILL.md format, naming)
- ❌ Skill version management
- ❌ Skill dependency resolution
- ❌ Skills marketplace integration
- ❌ Conflict detection when same skill exists in multiple locations
- ❌ Skill usage analytics (which skills are actually being used?)
- ❌ Automated backup/recovery of skills
- ❌ Skills configuration UI/TUI

---

## Agent-Specific Implementation Details

### Claude Code
- **Personal skills**: `~/.claude/skills/<skill-name>/SKILL.md`
- **Project skills**: `.claude/skills/<skill-name>/SKILL.md`
- **Priority**: Project > Personal > Plugin > Managed
- **Tools provided**: FileSystemTools, ShellTools (for scripts)
- **Reload**: VSCode reload or `/init` command
- **Verification**: Ask Claude "What skills are available?"

### Windsurf (Codeium)
- **Global**: `~/.codeium/windsurf/skills/<skill-name>/`
- **Workspace**: `.windsurf/skills/<skill-name>/`
- **Special features**: UI creator, @-mention syntax
- **Invocation**: Automatic (progressive disclosure) or manual (`@skill-name`)
- **Best practice**: Clear descriptions help auto-invocation
- **Skills vs Rules**: Skills are for procedures; Rules are for behavior

### OpenCode
- **Search paths** (in order):
  1. `.opencode/skill/<name>/SKILL.md` (project)
  2. `~/.config/opencode/skill/<name>/SKILL.md` (global)
  3. `.claude/skills/<name>/SKILL.md` (project Claude-compat)
  4. `~/.claude/skills/<name>/SKILL.md` (global Claude-compat)
- **Tool**: Native `skill` tool for discovery
- **Can override**: Per-agent permissions in agent frontmatter or opencode.json

### Kilo Code
- **Global generic**: `~/.kilocode/skills/`
- **Global code-mode**: `~/.kilocode/skills-code/`
- **Global architect-mode**: `~/.kilocode/skills-architect/`
- **Project-level**: `.kilocode/skills/`, `.kilocode/skills-code/`, etc.
- **Priority**: Project > Global, Mode-specific > Generic
- **Reload**: VSCode reload required
- **Verification**: Ask agent directly about available skills

### Amp Code
- **Workspace**: `.agents/skills/`
- **User-level**: `~/.config/agents/skills/`
- **Philosophy**: Lazy-load for context efficiency with local tools
- **Default location**: `.agents/skills/`
- Minimal documentation available

---

## Key Opportunities for CLI Tool

1. **Discovery & Inventory**
   - List all skills across all agents
   - Show where each skill is located
   - Detect duplicates with version info

2. **Installation & Management**
   - Unified install command for all agents
   - Sync skills to specific agents
   - Validate SKILL.md format before installation

3. **Monitoring & Analytics**
   - Which agents have which skills?
   - Skill version tracking
   - Disk usage analysis
   - Last-modified dates

4. **Conflict Resolution**
   - Detect skill name collisions
   - Highlight version mismatches
   - Suggest cleanup actions

5. **Skills Marketplace Integration**
   - Search skills.io directory
   - Pull skills from GitHub
   - Install from skill repositories

6. **Developer Experience**
   - Hot-reload without VSCode restart
   - Generate skill templates
   - Run skill validation tests
   - Create skill documentation

---

## Tool Architecture Options

### Option 1: Pure CLI (Recommended for users)
```bash
# Discovery
skills list
skills list --agent claude-code
skills list --duplicates
skills list --unused

# Installation
skills install github-username/my-skill
skills install my-skill --to claude-code,windsurf,amp

# Management
skills sync --from claude-code --to windsurf
skills validate ./my-skill
skills remove old-skill --from all

# Info
skills show my-skill
skills show my-skill --agent windsurf
skills config
```

**Pros**: Fast, scriptable, fits CLI-native users  
**Cons**: No real-time visualization, less intuitive for discovery

### Option 2: TUI (Terminal UI) (Recommended for exploration)
```
╔════════════════════════════════════════════════════════════╗
║              Agent Skills Manager - Dashboard              ║
╠════════════════════════════════════════════════════════════╣
║                                                            ║
║ Agents (5)         Skills by Agent      Installed (47)    ║
║ ─────────          ────────────────     ──────────────   ║
║ ✓ Claude Code      git-workflow (2)     git-workflow      ║
║ ✓ Windsurf         code-review (1)      code-review       ║
║ ✓ OpenCode         deploy (3)           deploy            ║
║ ✓ Kilo Code        refactor (2)         testing [DUPE]    ║
║ ✓ Amp Code         testing (4)          auth-flows [OLD]  ║
║                    api-design (1)       ...               ║
║                                                            ║
║ [S]earch [I]nstall [V]iew [D]elete [C]lean [R]eload [Q]uit║
╚════════════════════════════════════════════════════════════╝
```

**Pros**: Interactive discovery, visual conflict detection, great UX  
**Cons**: Not scriptable, requires terminal support

### Option 3: Hybrid (CLI + Optional TUI) (Best of both worlds)
- Core as CLI library (`skills` command)
- Optional interactive mode with `skills tui` or `skills interactive`
- Shell completions for fast CLI users
- JSON output for scripting

**Recommended approach** - Start with CLI, add TUI later if needed.

---

## Implementation Stack Suggestions

### Language Choices
1. **Rust** (best)
   - Performance-critical for scanning multiple directories
   - Cross-platform (Windows, Mac, Linux equally good)
   - Single binary distribution
   - Growing ecosystem for TUI (crossterm, ratatui)
   - Mature CLI library (clap)

2. **Python** (good)
   - Faster development
   - Better for scripting/integration
   - Standard tool in dev environments
   - TUI options: rich, textual, curses

3. **Go** (acceptable)
   - Fast compilation
   - Cross-platform
   - Mature CLI libraries

### Key Libraries Needed

**CLI Framework**:
- Rust: `clap`, `structopt`
- Python: `click`, `typer`, `argparse`
- Go: `cobra`, `urfave/cli`

**TUI Framework** (if pursuing Hybrid approach):
- Rust: `ratatui`, `crossterm`
- Python: `textual`, `rich`
- Go: `bubbletea`

**File Operations**:
- Parse YAML frontmatter from SKILL.md
- Validate directory structure
- Walk file system efficiently

**Data Management**:
- SQLite for caching skill metadata
- JSON for export/import
- TOML for config

---

## Brainstorm: Feature Prioritization

### Phase 1 (MVP - Week 1-2)
- [x] Scan all agent skill directories
- [x] List installed skills with agent breakdown
- [x] Detect duplicates
- [x] Show skill metadata (name, description, version)
- [x] JSON output for scripting

### Phase 2 (Core Features - Week 3-4)
- [ ] Install skills to specific agents
- [ ] Sync skills across agents
- [ ] Validate SKILL.md format
- [ ] Remove/delete skills
- [ ] Configuration file (define agent locations)

### Phase 3 (UX - Week 5-6)
- [ ] TUI/Interactive mode
- [ ] Shell completions (bash, zsh, fish)
- [ ] Search skills in skills.io registry
- [ ] Pull from GitHub repositories
- [ ] Skill templates/scaffolding

### Phase 4 (Advanced - Week 7+)
- [ ] Skill versioning/upgrade management
- [ ] Dependency resolution between skills
- [ ] Usage tracking
- [ ] Conflict auto-resolution
- [ ] CI/CD integration
- [ ] Skill marketplace publishing

---

## Questions for Refinement

1. Should the tool detect CLI-editable agent configs (opencode.json, etc.) or just scan directories?
2. Do you want to support enterprise/managed settings level?
3. Should we cache metadata for performance, or always scan fresh?
4. How should we handle agent paths that don't exist yet?
5. Should skill validation be strict or permissive?
6. Do you want automatic cleanup (detecting orphaned skills)?
7. Should the tool handle skill dependencies (one skill requires another)?
8. Integration with git for version control of skills?
