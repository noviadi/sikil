# Archive: Milestone 1 Completed Tasks

Evidence for tasks completed before migration to commit-based verification (2026-01-18).

---

## M1-E01-T01: Initialize Rust Project

**Status**: done  
**Started**: 2026-01-17T09:10:00Z  
**Completed**: 2026-01-17T09:30:00Z  
**By**: amp-agent

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: Cargo.toml exists at project root
- S02: Package metadata complete (name, version, authors, license, edition)
- S03: All 17 production deps added (clap, serde, rusqlite, sha2, etc.)
- S04: All 3 dev deps added (assert_cmd, predicates, insta)
- S05: .gitignore exists with /target

---

## M1-E01-T02: Project Structure

**Status**: done  
**Started**: 2026-01-17T10:00:00Z  
**Completed**: 2026-01-17T10:00:00Z  
**By**: claude-code

### Verification Commands
- `cargo build`
- `cargo run -- --help`

### Subtasks Verified
- S01: src/main.rs exists with clap setup, cargo run -- --help shows usage
- S02: src/lib.rs exists for library exports, cargo build succeeds
- S03: Module directories created (cli/, core/, commands/, utils/) with mod.rs files

---

## M1-E01-T03: Setup Test Infrastructure

**Status**: done  
**Started**: 2026-01-17T11:00:00Z  
**Completed**: 2026-01-17T11:15:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: tests/ directory created for integration tests
- S02: tests/common/mod.rs created with test helpers, compiles successfully
- S03: Helper functions setup_temp_skill_dir() and create_skill_dir() created, unit tests pass
- S04: Helper functions create_skill_md(), create_minimal_skill_md(), create_complete_skill_md() created, tests pass
- S05: cargo test runs successfully, 9 tests passed (5 unit tests in common/mod.rs + 4 integration tests)

---

## M1-E02-T01: Define Skill Model

**Status**: done  
**Started**: 2026-01-17T12:00:00Z  
**Completed**: 2026-01-17T12:15:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: src/core/skill.rs created with skill model types
- S02: SkillMetadata struct defined with name, description, version, author, license fields
- S03: Skill struct defined with metadata, directory_name, installations, is_managed, repo_path fields
- S04: Serialize/Deserialize traits implemented for all structs (SkillMetadata, Skill, Installation, Agent, Scope)
- S05: 12 unit tests written and passing (test_skill_metadata_new, test_skill_metadata_builder, test_skill_new, test_skill_with_installation, test_skill_with_repo, test_skill_is_orphan, test_agent_cli_name, test_agent_from_cli_name, test_agent_all, test_installation_new, test_scope_equality, test_serialization)

---

## M1-E02-T02: Define Installation Model

**Status**: done  
**Started**: 2026-01-17T13:00:00Z  
**Completed**: 2026-01-17T13:05:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: Agent enum defined in src/core/skill.rs (from M1-E02-T01)
- S02: Scope enum defined in src/core/skill.rs (from M1-E02-T01)
- S03: Installation struct completed with is_symlink and symlink_target fields (base from M1-E02-T01, fields added in M1-E02-T02)
- S04: Display trait implemented for Agent (agent.to_string() returns cli_name)
- S05: 14 unit tests passing (12 from M1-E02-T01 + test_agent_display + test_installation_with_symlink, test_installation_new updated)

---

## M1-E02-T03: Define Error Types

**Status**: done  
**Started**: 2026-01-18T08:00:00Z  
**Completed**: 2026-01-18T08:15:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: src/core/errors.rs created with SikilError enum
- S02: SikilError enum defined with #[derive(Error, Debug)] using thiserror
- S03: Error variants added: InvalidSkillMd, SkillNotFound, DirectoryNotFound
- S04: Error variants added: SymlinkError, GitError, ConfigError
- S05: Error variants added: AlreadyExists, PermissionDenied, ValidationError
- S06: Security error variants added: PathTraversal, SymlinkNotAllowed, InvalidGitUrl, ConfigTooLarge
- S07: 14 unit tests written and passing

### Notes
Initially created as error.rs, renamed to errors.rs for roadmap compliance (S01).

---

## M1-E03-T01: Define Config Model

**Status**: done  
**Started**: 2026-01-18T09:00:00Z  
**Completed**: 2026-01-18T09:10:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: src/core/config.rs created
- S02: AgentConfig struct defined with enabled, global_path, workspace_path fields, compiles
- S03: Config struct defined with HashMap<String, AgentConfig>, compiles
- S04: Default trait implemented with hardcoded paths for all 5 agents (claude-code, windsurf, opencode, kilocode, amp), unit test passes
- S05: 7 unit tests written and passing (test_agent_config_new, test_config_new, test_config_insert_and_get_agent, test_config_default_has_all_agents, test_config_default_all_agents_enabled, test_config_default_agent_paths, test_config_serialization)

### Notes
FIX (2026-01-18): Corrected default agent paths to match TRD §Domain Model and official docs:
- Claude Code: ~/.cache/claude-code/skills → ~/.claude/skills
- Windsurf: ~/.cache/windsurf/skills → ~/.codeium/windsurf/skills
- OpenCode: ~/.cache/opencode/skills → ~/.config/opencode/skill
- KiloCode: ~/.cache/kilocode/skills → ~/.kilocode/skills
- Amp: ~/.cache/amp/skills → ~/.config/agents/skills

---

## M1-E03-T02: Config File Loading

**Status**: done  
**Started**: 2026-01-18T09:30:00Z  
**Completed**: 2026-01-18T09:50:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: toml crate already present in Cargo.toml dependencies
- S02: Config::load() implemented, loads from ~/.sikil/config.toml, unit test passes (test_config_load_valid_toml)
- S03: Fallback to defaults if file missing, unit test passes (test_config_load_missing_file_returns_defaults)
- S04: Path expansion using shellexpand implemented in expand_path() helper, unit test passes (test_config_expand_paths)
- S05: Config file size limit (1MB max) implemented and enforced, unit test passes (test_config_load_oversized_file_returns_error)
- S06: #[serde(deny_unknown_fields)] added to AgentConfig and Config structs, unit test passes (test_config_deny_unknown_fields)
- S07: Integration test with temp config file passes (all subtask tests pass)

---

## M1-E03-T03: Test Config System

**Status**: done  
**Started**: 2026-01-18T09:55:00Z  
**Completed**: 2026-01-18T10:00:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: Test loading valid TOML config - already covered by test_config_load_valid_toml
- S02: Test invalid TOML returns error - already covered by test_config_load_invalid_toml_returns_error
- S03: Test partial config merges with defaults - already covered by test_config_load_partial_merges_with_defaults
- S04: Test disabled agent is excluded - added test_config_disabled_agent_is_excluded and test_config_iter_enabled_agents_only
- S05: Test config file over size limit returns error - already covered by test_config_load_oversized_file_returns_error
- S06: Test unknown fields in config returns error - already covered by test_config_deny_unknown_fields

---

## M1-E04-T01: Frontmatter Extraction

**Status**: done  
**Started**: 2026-01-18T11:00:00Z  
**Completed**: 2026-01-18T11:30:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: src/core/parser.rs created with extract_frontmatter() function
- S02: extract_frontmatter() correctly extracts YAML content between --- markers, returns &str
- S03: Missing frontmatter (no --- markers) returns InvalidSkillMd error with descriptive reason
- S04: Malformed frontmatter (single ---) returns InvalidSkillMd error with descriptive reason
- S05: 16 unit tests written and passing (test_extract_frontmatter_valid, test_extract_frontmatter_valid_with_leading_newline, test_extract_frontmatter_valid_multiline, test_extract_frontmatter_missing_markers, test_extract_frontmatter_single_marker, test_extract_frontmatter_not_at_start, test_extract_frontmatter_empty_frontmatter, test_extract_frontmatter_with_empty_lines_in_frontmatter, test_extract_frontmatter_preserves_internal_spacing, test_extract_frontmatter_empty_content, test_extract_frontmatter_only_whitespace, test_extract_frontmatter_three_markers, test_extract_frontmatter_marker_with_spaces, test_extract_frontmatter_complex_yaml, plus 1 doc test)

---

## M1-E04-T02: Metadata Parsing

**Status**: done  
**Started**: 2026-01-18T12:00:00Z  
**Completed**: 2026-01-18T12:15:00Z  
**By**: claude-code

### Verification Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### Subtasks Verified
- S01: parse_skill_md() function implemented in src/core/parser.rs, reads file, extracts frontmatter, parses YAML into SkillMetadata
- S02: Required field 'name' parsed from YAML and validated, unit tests pass (test_parse_skill_md_valid_minimal, test_parse_skill_md_missing_name)
- S03: Required field 'description' parsed from YAML and validated, unit tests pass (test_parse_skill_md_valid_minimal, test_parse_skill_md_missing_description)
- S04: Optional fields version, author, license parsed correctly when present, unit test passes (test_parse_skill_md_valid_with_all_fields)
- S05: Returns InvalidSkillMd error when required fields missing, unit tests pass (test_parse_skill_md_missing_name, test_parse_skill_md_missing_description, test_parse_skill_md_missing_both_required_fields)

### Notes
Implemented parse_skill_md() function in src/core/parser.rs that reads SKILL.md file, extracts YAML frontmatter using extract_frontmatter(), parses into SkillMetadata, and validates required fields (name, description). Added RawSkillMetadata helper struct with Option fields for graceful parsing. Created 9 comprehensive unit tests. All 78 tests pass (67 lib tests + 9 integration tests + 2 doc tests).
