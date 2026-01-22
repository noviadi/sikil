# Shell Completions Spec

## One-Sentence Description

Shell completions generate auto-completion scripts for various shell environments.

## Overview

The `sikil completions` command generates shell-specific completion scripts that enable tab-completion for sikil commands, subcommands, options, and arguments. The implementation uses the `clap_complete` crate to generate completions from the CLI definition.

## Supported Shells

| Shell | Identifier | Case-Insensitive |
|-------|------------|------------------|
| Bash  | `bash`     | Yes              |
| Zsh   | `zsh`      | Yes              |
| Fish  | `fish`     | Yes              |

Any other shell identifier returns an error with the message: `Unsupported shell '<name>'. Supported shells: bash, zsh, fish`

## Generation Process

1. Parse the shell argument (case-insensitive string matching)
2. Map to `clap_complete::Shell` enum variant
3. Call `clap_complete::generate()` with:
   - The shell variant
   - The `Cli::command()` structure
   - Binary name: `"sikil"`
   - Output buffer

## Output

| Option | Behavior |
|--------|----------|
| No `--output` flag | Completions written to stdout |
| `--output <PATH>` | Completions written to file at PATH, confirmation printed to stderr |

## Installation Instructions

**Bash:**
```bash
sikil completions bash >> ~/.bashrc
# or
sikil completions bash --output /etc/bash_completion.d/sikil
```

**Zsh:**
```bash
sikil completions zsh --output ~/.zsh/completions/_sikil
```

**Fish:**
```bash
sikil completions fish --output ~/.config/fish/completions/sikil.fish
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `clap_complete` | Shell completion generation |
| `clap::CommandFactory` | Access CLI command structure via `Cli::command()` |
| `crate::cli::Cli` | CLI definition to generate completions from |

## Used By

- End users installing sikil for shell integration
- Package maintainers creating distribution packages
