# Contributing to Sikil

Thank you for your interest in contributing to Sikil! This document provides guidelines for contributors.

## Development Setup

### Prerequisites

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - Required for cloning and some features
- **SQLite** - Bundled with the application, but development tools may be helpful

### Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/noviadi/sikil.git
   cd sikil
   ```

2. **Install dependencies and build**
   ```bash
   cargo build
   ```

3. **Run tests to verify setup**
   ```bash
   cargo test
   ```

4. **Install for development**
   ```bash
   cargo install --path .
   ```

### Development Workflow

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** - See [Code Style Guidelines](#code-style-guidelines)

3. **Run the verification suite**
   ```bash
   ./scripts/verify.sh
   ```

4. **Test your changes manually**
   ```bash
   cargo run -- --help
   # Test your specific changes
   ```

5. **Commit your changes** - Use [conventional commits](https://www.conventionalcommits.org/)
   ```bash
   git commit -m "feat: add new feature"
   ```

6. **Push and create a pull request**
   ```bash
   git push origin feature/your-feature-name
   ```

## Testing Instructions

### Running Tests

Sikil uses a comprehensive test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests only
cargo test --test '*_test'

# Run unit tests only
cargo test --lib
```

### Test Structure

- **Unit tests**: Located in `src/*/tests.rs` files
- **Integration tests**: Located in `tests/` directory
- **Snapshot tests**: Use `insta` for output verification

### Writing Tests

1. **Unit Tests** - Test individual functions and modules
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_function() {
           let result = function_under_test();
           assert_eq!(result, expected);
       }
   }
   ```

2. **Integration Tests** - Test CLI commands end-to-end
   ```rust
   use assert_cmd::Command;
   
   #[test]
   fn test_command() {
       let mut cmd = Command::cargo_bin("sikil").unwrap();
       cmd.arg("list")
          .assert()
          .success();
   }
   ```

3. **Snapshot Tests** - Verify structured output
   ```rust
   use insta::assert_debug_snapshot;
   
   #[test]
   fn test_output() {
       let result = generate_output();
       assert_debug_snapshot!(result);
   }
   ```

### Test Fixtures

Test fixtures are located in `tests/fixtures/`:
- `skills/` - Sample skill directories for testing
- Configuration files for various scenarios

### Coverage Guidelines

- All public functions should have tests
- All CLI commands should have integration tests
- Error paths should be tested
- Edge cases and boundary conditions should be covered

## Code Style Guidelines

### Rust Style

Sikil follows standard Rust conventions with additional project-specific rules:

#### Error Handling

- **Core layer** (`src/core/`): Use `thiserror`
  ```rust
  #[derive(thiserror::Error, Debug)]
  pub enum SikilError {
      #[error("Skill not found: {name}")]
      SkillNotFound { name: String },
  }
  ```

- **Commands layer** (`src/commands/`): Use `anyhow`
  ```rust
  use anyhow::{Context, Result};
  
  fn execute() -> Result<()> {
      core_function().with_context(|| "Failed to execute")?;
      Ok(())
  }
  ```

#### Serialization

- Use kebab-case for enums:
  ```rust
  #[derive(Serialize, Deserialize)]
  #[serde(rename_all = "kebab-case")]
  pub enum MyEnum {
      VariantOne,
      VariantTwo,
  }
  ```

- Optional fields with defaults:
  ```rust
  #[derive(Serialize, Deserialize)]
  pub struct MyStruct {
      #[serde(default, skip_serializing_if = "Option::is_none")]
      pub optional_field: Option<String>,
  }
  ```

#### Filesystem Operations

- Use `fs-err` instead of `std::fs`:
  ```rust
  use fs_err as fs;
  
  fs::create_dir_all(path)?;
  ```

- Use `tempfile` for atomic operations:
  ```rust
  use tempfile::tempdir;
  
  let temp_dir = tempdir()?;
  // Use temp_dir.path() for operations
  ```

### Code Organization

#### Module Structure

```
src/
├── cli/           # CLI argument parsing and output formatting
├── commands/      # Command orchestration and error handling
├── core/          # Domain logic and types
└── utils/         # Reusable utilities
```

#### Dependencies Flow

Dependencies flow downward: `cli` → `commands` → `core` → `utils`

- **cli/**: Can use all other modules
- **commands/**: Can use `core` and `utils`
- **core/**: Can only use `utils`
- **utils/**: No internal dependencies

#### Naming Conventions

- **Functions**: `snake_case`
- **Types**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Files**: `snake_case.rs`

### Documentation

#### Public API Documentation

All public items must have documentation:

```rust
/// Parses a skill from the given path.
///
/// # Arguments
///
/// * `path` - Path to the skill directory
///
/// # Returns
///
/// Returns `Skill` on success, or `Error` if:
/// - SKILL.md is missing
/// - Frontmatter is invalid
///
/// # Examples
///
/// ```rust
/// let skill = parse_skill(Path::new("./my-skill"))?;
/// ```
pub fn parse_skill(path: &Path) -> Result<Skill> {
    // ...
}
```

#### Comments

- Use `//` for line comments
- Use `/* */` for block comments (rare)
- Explain *why*, not *what*
- Keep comments up-to-date with code

### Git Conventions

#### Commit Messages

Use [conventional commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(list): add --json output flag

Add structured JSON output for the list command to enable
programmatic consumption of skill inventory.

Closes #123
```

#### Version Management

Sikil follows [Semantic Versioning 2.0.0](https://semver.org/). Version numbers are managed in `Cargo.toml` and reflected in the CLI via `--version`.

**Version Format**: `MAJOR.MINOR.PATCH` (e.g., `0.1.0`)

**When to bump versions**:
- **MAJOR**: Breaking changes to public API or command behavior
- **MINOR**: New features, backward-compatible additions
- **PATCH**: Bug fixes, documentation updates

**Version bump process**:
1. Update version in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"  # Increment as appropriate
   ```
2. Update the `--version` flag in `src/cli/app.rs`:
   ```rust
   #[command(version = "0.2.0")]
   ```
3. Add an entry to `CHANGELOG.md` under the new version
4. Commit and tag:
   ```bash
   git add Cargo.toml src/cli/app.rs CHANGELOG.md
   git commit -m "chore: bump version to 0.2.0"
   git tag v0.2.0
   git push && git push --tags
   ```

**Important**: Always update both `Cargo.toml` and `src/cli/app.rs` to ensure version consistency.

#### Pull Request Process

1. **Update documentation** if needed
2. **Add tests** for new functionality
3. **Run verification suite** (`./scripts/verify.sh`)
4. **Update CHANGELOG.md** for user-facing changes
5. **Request review** from maintainers

### Performance Guidelines

- Use `Vec` with known capacity when possible
- Prefer `&str` over `String` for function parameters
- Use `Cow<str>` for conditional ownership
- Profile with `cargo build --release` for performance testing

### Security Considerations

- Validate all external inputs
- Use path canonicalization to prevent traversal
- Reject symlinks in skill directories
- Limit file sizes (1MB for config, 64KB for frontmatter)
- Use command arrays for subprocess calls (no shell injection)

## Getting Help

- **Issues**: Report bugs or request features on [GitHub Issues](https://github.com/noviadi/sikil/issues)
- **Discussions**: Use [GitHub Discussions](https://github.com/noviadi/sikil/discussions) for questions
- **Documentation**: See `docs/` directory for detailed technical documentation

## License

By contributing to Sikil, you agree that your contributions will be licensed under the MIT License.
