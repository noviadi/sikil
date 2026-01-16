# Agent Skills Manager - Technical Specification

## Data Models (Rust)

```rust
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub installations: Vec<Installation>,
    pub has_conflicts: bool,
}

#[derive(Debug, Clone)]
pub struct Installation {
    pub agent: Agent,
    pub path: PathBuf,
    pub scope: SkillScope,  // Global/Project/Workspace/Mode
}
```

## Scanner Algorithm

1. **Global paths** (always): `~/.claude/skills/`, `~/.codeium/windsurf/skills/`, etc.
2. **Project walk** (CWD → git root): `.claude/skills/`, `.windsurf/skills/`, etc.
3. **Parse SKILL.md** YAML frontmatter
4. **Dedupe** by name, detect conflicts
5. **Cache** in SQLite (~10ms subsequent scans)

## Agent Configurations

```rust
pub const CLAUDE_CODE: AgentConfig = AgentConfig {
    global: "~/.claude/skills",
    project: ".claude/skills",
    priority: vec![SkillScope::Project, SkillScope::Global],
};
```

## Key Algorithms

**Conflict Detection**:
- Same name + different versions = VersionMismatch
- Global + project same name = ScopeConflict
- Missing dependencies = DependencyError

**Sync Engine**:
1. Scan source agent
2. Diff with target agents  
3. Install missing, update outdated
4. Report changes

## Performance

- **Initial scan**: ~150ms (5 agents + 10 projects)
- **Cached scan**: ~10ms
- **Sync 30 skills**: <1 second

## Error Handling

```rust
enum SkillsError {
    InvalidYAML(serde_yaml::Error),
    DirectoryNotFound(PathBuf),
    VersionMismatch { skill: String, versions: Vec<String> },
    PermissionDenied(PathBuf),
}
```

## Config Format (TOML)

```toml
[agents.claude-code]
global_path = "~/.claude/skills"
project_path = ".claude/skills"

[cache]
enabled = true
max_age = "1h"
```

**Extensible**: Add new agents via config only.

---
**Ready for Rust implementation. 5-week MVP timeline.**