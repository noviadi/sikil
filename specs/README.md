# Single Source of Truth (SSOT) Specs

This directory contains specification documents for each distinct topic of concern in the Sikil codebase. Each spec documents one component that can be described in a single sentence without "and".

## Topic Index

| Topic | Spec | One-Sentence Description |
|-------|------|--------------------------|
| **Core Domain** | | |
| Skill Model | [skill-model.md](skill-model.md) | Defines data structures for representing skills |
| Agent Model | [agent-model.md](agent-model.md) | Defines supported AI coding agents |
| Error Handling | [error-handling.md](error-handling.md) | Defines structured error types for user feedback |
| **Discovery & Analysis** | | |
| Skill Scanner | [skill-scanner.md](skill-scanner.md) | Discovers installed skills across agent directories |
| Skill Discovery | [skill-discovery.md](skill-discovery.md) | Displays installed skills via list/show commands |
| Conflict Detection | [conflict-detection.md](conflict-detection.md) | Identifies duplicate skills across installations |
| Skill Validation | [skill-validation.md](skill-validation.md) | Verifies skill directories conform to SKILL.md specification |
| **Skill Management** | | |
| Skill Installation | [skill-installation.md](skill-installation.md) | Copies skills to managed repository via symlinks |
| Skill Adoption | [skill-adoption.md](skill-adoption.md) | Moves unmanaged skills into managed repository |
| Skill Removal | [skill-removal.md](skill-removal.md) | Deletes skills from agent directories |
| Skill Unmanagement | [skill-unmanagement.md](skill-unmanagement.md) | Converts managed skills back to standalone copies |
| Skill Synchronization | [skill-synchronization.md](skill-synchronization.md) | Creates missing symlinks for managed skills |
| Agent Targeting | [agent-targeting.md](agent-targeting.md) | Selects enabled agents for multi-agent operations |
| **Infrastructure** | | |
| CLI Schema | [cli-schema.md](cli-schema.md) | Defines command-line arguments for all commands |
| CLI Output | [cli-output.md](cli-output.md) | Controls terminal formatting for user feedback |
| Configuration | [configuration.md](configuration.md) | Manages user preferences in TOML format |
| Cache | [cache.md](cache.md) | Stores parsed skill metadata for faster scans |
| **Utilities** | | |
| Filesystem Paths | [filesystem-paths.md](filesystem-paths.md) | Defines Sikil's on-disk directory layout |
| Symlink Operations | [symlink-operations.md](symlink-operations.md) | Manages symbolic links to agent directories |
| Atomic Operations | [atomic-operations.md](atomic-operations.md) | Ensures safe filesystem mutations via rollback |
| Git Operations | [git-operations.md](git-operations.md) | Retrieves skills from GitHub repositories |
| Shell Completions | [shell-completions.md](shell-completions.md) | Generates auto-completion scripts for shells |
| **Build & Testing** | | |
| Build Constraints | [build-and-platform.md](build-and-platform.md) | Defines compilation targets for supported platforms |
| Testing Strategy | [testing-strategy.md](testing-strategy.md) | Validates behavior through automated tests |

## Architecture Mapping

```
specs/ssot/
├── Core Domain
│   ├── skill-model.md          → src/core/skill.rs, parser.rs
│   ├── agent-model.md          → src/core/agent.rs
│   └── error-handling.md       → src/core/errors.rs
├── Discovery & Analysis
│   ├── skill-scanner.md        → src/core/scanner.rs
│   ├── skill-discovery.md      → src/commands/{list,show}.rs
│   ├── conflict-detection.md   → src/core/conflicts.rs
│   └── skill-validation.md     → src/commands/validate.rs
├── Skill Management
│   ├── skill-installation.md   → src/commands/install.rs
│   ├── skill-adoption.md       → src/commands/adopt.rs
│   ├── skill-removal.md        → src/commands/remove.rs
│   ├── skill-unmanagement.md   → src/commands/unmanage.rs
│   ├── skill-synchronization.md → src/commands/sync.rs
│   └── agent-targeting.md      → src/commands/agent_selection.rs
├── Infrastructure
│   ├── cli-schema.md           → src/cli/app.rs
│   ├── cli-output.md           → src/cli/output.rs
│   ├── configuration.md        → src/core/config.rs, commands/config.rs
│   └── cache.md                → src/core/cache.rs
├── Utilities
│   ├── filesystem-paths.md     → src/utils/paths.rs
│   ├── symlink-operations.md   → src/utils/symlink.rs
│   ├── atomic-operations.md    → src/utils/atomic.rs
│   ├── git-operations.md       → src/utils/git.rs
│   └── shell-completions.md    → src/commands/completions.rs
└── Build & Testing
    ├── build-and-platform.md   → Cargo.toml, scripts/build.sh
    └── testing-strategy.md     → tests/, src/core/snapshots/
```

## Usage

Each spec follows a consistent format:
- **One-Sentence Description** - Topic scope test
- **Overview** - Brief context
- **Technical Details** - Implementation specifics
- **Dependencies/Used By** - Relationship to other components
