//! CLI application structure using clap
//!
//! This module defines the main CLI structure with all subcommands
//! and global flags for the sikil tool.

use clap::{Parser, Subcommand};

/// Sikil - A CLI tool for managing AI coding assistant skills
#[derive(Parser, Debug)]
#[command(name = "sikil")]
#[command(author = "Sikil Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Manage AI coding assistant skills across multiple agents", long_about = None)]
#[command(after_help = r##"
EXAMPLES:
    sikil list
        List all installed skills

    sikil show <skill>
        Show detailed information about a skill

    sikil install <path>
        Install a skill from a local path

    sikil install <git-url>
        Install a skill from a Git repository

    sikil validate <path>
        Validate a skill's SKILL.md file

    sikil --json list
        Output in JSON format

For more help on any command:
    sikil <command> --help
"##)]
pub struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    pub json: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output (except JSON data on stdout)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable cache for this command
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for sikil
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List installed skills
    #[command(after_help = r##"
EXAMPLES:
    sikil list
        List all installed skills

    sikil list --agent claude-code
        List skills installed for Claude Code only

    sikil list --managed
        List only managed skills

    sikil list --json
        Output in JSON format
"##)]
    List,

    /// Show detailed information about a skill
    #[command(after_help = r##"
EXAMPLES:
    sikil show my-skill
        Show details for 'my-skill'

    sikil show --json my-skill
        Output in JSON format
"##)]
    Show {
        /// Name of the skill to show
        name: String,
    },

    /// Install a skill from a local path or Git repository
    #[command(after_help = r##"
EXAMPLES:
    sikil install ./path/to/skill
        Install a skill from a local path

    sikil install user/repo
        Install a skill from GitHub (short form)

    sikil install https://github.com/user/repo.git
        Install a skill from a Git URL

    sikil install ./skill --to claude-code,windsurf
        Install to specific agents

    sikil install ./skill --to all
        Install to all enabled agents
"##)]
    Install {
        /// Path to the skill directory or Git URL
        #[arg(value_name = "PATH_OR_URL")]
        source: String,

        /// Agents to install to (comma-separated or 'all')
        #[arg(short, long, value_name = "AGENTS")]
        r#to: Option<String>,
    },

    /// Validate a skill's SKILL.md file
    #[command(after_help = r##"
EXAMPLES:
    sikil validate ./path/to/skill
        Validate a skill at a given path

    sikil validate my-skill
        Validate an installed skill by name
"##)]
    Validate {
        /// Path to the skill or name of installed skill
        #[arg(value_name = "PATH_OR_NAME")]
        path: String,
    },

    /// Adopt an unmanaged skill into management
    #[command(after_help = r##"
EXAMPLES:
    sikil adopt my-skill
        Adopt 'my-skill' into management

    sikil adopt my-skill --from claude-code
        Adopt from a specific agent if multiple locations exist
"##)]
    Adopt {
        /// Name of the skill to adopt
        name: String,

        /// Agent to adopt from (required if multiple locations)
        #[arg(short, long, value_name = "AGENT")]
        from: Option<String>,
    },

    /// Unmanage a skill (convert to unmanaged)
    #[command(after_help = r##"
EXAMPLES:
    sikil unmanage my-skill
        Unmanage from all agents

    sikil unmanage my-skill --agent claude-code
        Unmanage from a specific agent only

    sikil unmanage my-skill --yes
        Skip confirmation prompt
"##)]
    Unmanage {
        /// Name of the skill to unmanage
        name: String,

        /// Specific agent to unmanage from
        #[arg(short, long, value_name = "AGENT")]
        agent: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },

    /// Remove an installed skill
    #[command(after_help = r##"
EXAMPLES:
    sikil remove my-skill --agent claude-code
        Remove from a specific agent

    sikil remove my-skill --all
        Remove from all agents and delete from repo

    sikil remove my-skill --all --yes
        Skip confirmation prompt
"##)]
    Remove {
        /// Name of the skill to remove
        name: String,

        /// Specific agent(s) to remove from
        #[arg(short, long, value_name = "AGENT")]
        agent: Option<String>,

        /// Remove from all agents and delete from repo
        #[arg(long, conflicts_with = "agent")]
        all: bool,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },

    /// Sync a skill to other agents
    #[command(after_help = r##"
EXAMPLES:
    sikil sync my-skill
        Sync 'my-skill' to all enabled agents

    sikil sync --all
        Sync all managed skills

    sikil sync my-skill --to claude-code,windsurf
        Sync to specific agents only
"##)]
    Sync {
        /// Name of the skill to sync (omit for --all)
        name: Option<String>,

        /// Sync all managed skills
        #[arg(long, conflicts_with = "name")]
        all: bool,

        /// Specific agents to sync to
        #[arg(short, long, value_name = "AGENTS")]
        r#to: Option<String>,
    },

    /// Manage configuration
    Config {
        /// Edit the config file
        #[arg(long)]
        edit: bool,

        /// Set a config value (key.subkey value)
        #[arg(long, value_names = ["KEY", "VALUE"])]
        set: bool,
    },

    /// Generate shell completions
    #[command(after_help = r##"
EXAMPLES:
    sikil completions bash
        Generate bash completions

    sikil completions zsh --output ~/.zsh/completions/_sikil
        Generate zsh completions to a file

    sikil completions fish
        Generate fish completions
"##)]
    Completions {
        /// Shell type (bash, zsh, fish)
        #[arg(value_name = "SHELL")]
        shell: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long, value_name = "PATH")]
        output: Option<String>,
    },
}
