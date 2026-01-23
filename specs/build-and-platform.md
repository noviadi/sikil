# Build and Platform Spec

## One-Sentence Description

Build constraints define how Sikil is compiled for supported platforms.

## Overview

Sikil is a Rust CLI that targets Unix-like systems (macOS and Linux). Windows is not supported because the codebase uses Unix-specific symlink APIs. This document specifies platform requirements, build configuration, dependencies, and distribution.

## Supported Platforms

| Platform | Architecture | Target Triple |
|----------|--------------|---------------|
| macOS | Intel | `x86_64-apple-darwin` |
| macOS | Apple Silicon | `aarch64-apple-darwin` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` |

### Windows Not Supported

The `src/utils/symlink.rs` module uses `std::os::unix::fs::symlink()` which is not available on Windows. All symlink operations (create, test, read target) use Unix-specific APIs.

## Rust Configuration

| Setting | Value | Source |
|---------|-------|--------|
| Edition | 2021 | `Cargo.toml` |
| Minimum Version | 1.75+ | README.md |

### Release Profile

From `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

This produces optimized, small binaries (~10MB target).

## Key Dependencies

### Runtime Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4 | CLI argument parsing |
| `serde` + `serde_yaml` + `serde_json` | 1, 0.9, 1 | Serialization (SKILL.md, config, JSON output) |
| `rusqlite` | 0.31 (bundled) | SQLite cache for skill metadata |
| `fs-err` | 2 | Filesystem operations with better errors |
| `tempfile` | 3 | Temporary directories for atomic operations |
| `walkdir` | 2 | Directory traversal |
| `shellexpand` | 3 | Tilde expansion for paths |
| `directories` | 5 | XDG/platform directories |
| `regex` | 1 | Pattern matching |
| `sha2` | 0.10 | Content hashing |

### Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `assert_cmd` | 2 | CLI integration testing |
| `predicates` | 3 | Test assertions |
| `insta` | 1 | Snapshot testing |

### Build Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4 | CLI structure for man page generation |
| `clap_mangen` | 0.2 | Man page generation |

## Runtime Requirements

| Requirement | Used By | Purpose |
|-------------|---------|---------|
| `git` CLI | `src/utils/git.rs` | Cloning repositories for `sikil install` |

Git is invoked via `std::process::Command`:

```
git clone -c protocol.file.allow=never --depth=1 -- <url> <dest>
```

## Build Script

The `build.rs` script generates the `sikil.1` man page automatically during release builds. It imports `src/cli/app.rs` directly (via `#[path]`) and uses `clap_mangen` to render the man page, ensuring the man page always stays in sync with the CLI.

**Triggers:**
- Release builds (`cargo build --release`)
- When `SIKIL_GENERATE_MAN=1` environment variable is set

**Output:** `sikil.1` in the project root

To force man page regeneration during development:
```bash
./scripts/generate-man.sh
# or
SIKIL_GENERATE_MAN=1 cargo build
```

## Build Commands

### Development Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

This also regenerates the man page.

### Cross-Compilation

```bash
./scripts/build.sh <target>
# Example: ./scripts/build.sh x86_64-unknown-linux-gnu
```

The build script:
1. Verifies cargo is available
2. Installs target via `rustup target add` if needed
3. Runs `cargo build --release --target <target>`
4. Performs smoke test (`sikil --version`)
5. Warns if binary exceeds 10MB

## Binary Distribution

### From Crates.io

```bash
cargo install sikil
```

### From Source

```bash
git clone https://github.com/noviadi/sikil.git
cd sikil
cargo install --path .
```

### Pre-built Binaries

```bash
curl -L https://github.com/noviadi/sikil/releases/latest/download/sikil-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv sikil /usr/local/bin/
```

## Acceptance Criteria

- `cargo build` compiles successfully with Rust 1.75+
- `cargo build --release` produces binary under 10MB
- Release build generates `sikil.1` man page in project root
- `SIKIL_GENERATE_MAN=1 cargo build` generates man page during debug builds
- Binary runs on macOS x86_64 (`x86_64-apple-darwin`) target
- Binary runs on macOS aarch64 (`aarch64-apple-darwin`) target
- Binary runs on Linux x86_64 (`x86_64-unknown-linux-gnu`) target
- Binary runs on Linux aarch64 (`aarch64-unknown-linux-gnu`) target
- Build fails on Windows due to `std::os::unix::fs::symlink()` dependency
- `git` CLI is available at runtime for `sikil install` command
- `./scripts/build.sh <target>` cross-compiles and runs smoke test

## Dependencies

- None (this is a foundational spec)

## Used By

- `scripts/build.sh`: Implements release build process
- CI/CD pipelines: Target platform matrix
- README.md: Installation instructions
