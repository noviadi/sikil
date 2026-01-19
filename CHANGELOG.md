# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release infrastructure

## [0.1.0] - 2026-01-19

### Added
- **Core CLI Framework** - Complete command-line interface with argument parsing and output formatting
- **Skill Discovery** - Scan and list skills across all agent installations (Claude Code, Windsurf, OpenCode, Kilo Code, Amp)
- **Skill Installation** - Install skills from local paths or GitHub repositories to multiple agents
- **Git Integration** - Clone and install skills directly from GitHub with security hardening
- **Skill Management** - Adopt existing unmanaged skills, unmanage managed skills, and remove installations
- **Synchronization** - Sync managed skills across agents with conflict detection
- **Validation** - Verify skill structure and metadata before installation
- **Configuration** - Flexible agent configuration with TOML-based settings
- **JSON Output** - Structured JSON output for all commands for programmatic use
- **Shell Completions** - Generate bash, zsh, and fish shell completions
- **Caching System** - SQLite-based caching for fast skill discovery
- **Security Features** - Path traversal prevention, symlink rejection, and GitHub-only Git URLs
- **Cross-Platform Support** - Support for macOS (Intel + ARM) and Linux (x86_64 + aarch64)

### Commands
- `list` - List all skills with filtering options (agent, managed/unmanaged, conflicts, duplicates)
- `show` - Display detailed information about specific skills
- `install` - Install skills from local paths or Git repositories
- `adopt` - Bring existing unmanaged skills under centralized management
- `unmanage` - Convert managed skills back to unmanaged copies
- `remove` - Remove skills from specific agents or completely
- `sync` - Synchronize managed skills across agents
- `validate` - Verify skill structure and metadata
- `config` - Manage agent configuration and settings
- `completions` - Generate shell completions

### Features
- **Multi-Agent Support** - Unified management across 5 AI coding agents
- **Atomic Operations** - Safe filesystem operations with rollback on failure
- **Conflict Detection** - Identify duplicate skills and version mismatches
- **Progress Indicators** - Visual progress for long-running operations
- **Flexible Installation** - Install to specific agents or all enabled agents
- **Skill Metadata** - Parse and validate SKILL.md YAML frontmatter
- **Symlink Management** - Efficient symlink-based skill storage
- **Workspace Support** - Support for both global and workspace-scoped installations

### Technical
- **Rust 2021 Edition** - Modern Rust with latest language features
- **SQLite Caching** - Fast local caching with WAL mode
- **Serde Integration** - Robust serialization for configuration and data
- **Error Handling** - Comprehensive error reporting with context
- **Testing** - Complete test suite with unit and integration tests
- **Documentation** - Comprehensive documentation and contributing guidelines

### Security
- **Input Validation** - Comprehensive validation of all external inputs
- **Path Security** - Protection against path traversal attacks
- **Git Security** - Hardened Git operations with protocol restrictions
- **Symlink Safety** - Rejection of symlinks in skill directories
- **Size Limits** - Configurable limits for configuration and skill files

[Unreleased]: https://github.com/noviadi/sikil/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/noviadi/sikil/releases/tag/v0.1.0
