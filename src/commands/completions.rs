//! Shell completions command implementation
//!
//! This module provides shell completion generation using clap_complete.

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::fs;
use std::io::Write;

use crate::cli::Cli;

/// Arguments for the completions command
#[derive(Debug, Clone)]
pub struct CompletionsArgs {
    /// Shell type
    pub shell: String,
    /// Output file path (optional)
    pub output: Option<String>,
}

/// Execute the completions command
pub fn execute_completions(args: CompletionsArgs) -> Result<()> {
    // Parse shell type
    let shell = match args.shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported shell '{}'. Supported shells: bash, zsh, fish",
                args.shell
            ));
        }
    };

    // Generate completions
    let mut buf = Vec::new();
    generate(shell, &mut Cli::command(), "sikil", &mut buf);

    // Write to output
    match args.output {
        Some(path) => {
            // Write to file
            let mut file = fs::File::create(&path)?;
            file.write_all(&buf)?;
            eprintln!("Completions written to: {}", path);
        }
        None => {
            // Write to stdout
            std::io::stdout().write_all(&buf)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_parsing() {
        let args = CompletionsArgs {
            shell: "bash".to_string(),
            output: None,
        };
        assert!(execute_completions(args).is_ok());

        let args = CompletionsArgs {
            shell: "zsh".to_string(),
            output: None,
        };
        assert!(execute_completions(args).is_ok());

        let args = CompletionsArgs {
            shell: "fish".to_string(),
            output: None,
        };
        assert!(execute_completions(args).is_ok());

        let args = CompletionsArgs {
            shell: "invalid".to_string(),
            output: None,
        };
        assert!(execute_completions(args).is_err());
    }
}
