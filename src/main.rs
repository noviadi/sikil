use clap::Parser;
use sikil::cli::Cli;
use sikil::commands::{
    execute_adopt, execute_completions, execute_config, execute_install_git, execute_install_local,
    execute_list, execute_remove, execute_show, execute_sync, execute_unmanage, execute_validate,
    AdoptArgs, CompletionsArgs, ConfigArgs, InstallArgs, ListArgs, RemoveArgs, ShowArgs, SyncArgs,
    UnmanageArgs, ValidateArgs,
};
use sikil::core::config::Config;
use sikil::core::errors::SikilError;
use sikil::core::skill::Agent;
use sikil::utils::paths::get_config_path;

/// Gets the appropriate exit code for an error.
/// If the error is a `SikilError`, returns the exit code defined by that error type.
/// Otherwise, returns the default exit code of 1.
fn get_exit_code(err: &anyhow::Error) -> i32 {
    if let Some(sikil_err) = err.downcast_ref::<SikilError>() {
        sikil_err.exit_code()
    } else {
        1 // Default exit code for non-SikilError errors
    }
}

/// Detects if the given source string is a Git URL or a local path.
///
/// Returns `true` if the source appears to be a Git URL, `false` if it's a local path.
///
/// Git URL formats:
/// - HTTPS: `https://github.com/owner/repo.git` or `https://github.com/owner/repo`
/// - Short form: `owner/repo` or `owner/repo/path/to/skill`
///
/// Local path indicators:
/// - Absolute paths starting with `/`
/// - Relative paths starting with `.` or `..`
/// - Paths starting with `-` (argument injection protection)
/// - Paths that exist on the filesystem
/// - Single-segment paths (e.g., `my-skill`)
fn is_git_url(source: &str) -> bool {
    // HTTPS URLs starting with https://github.com/
    if source.to_lowercase().starts_with("https://github.com/") {
        return true;
    }

    // Short form: owner/repo or owner/repo/path/to/skill
    // Check if it looks like "owner/repo" format (contains / but doesn't start with / or . or -)
    if source.contains('/') {
        // Not an absolute path or relative path with dots
        if source.starts_with('/') || source.starts_with('.') || source.starts_with('-') {
            return false;
        }

        // Check if it's a valid short-form Git URL (has at least 2 / separated parts)
        let parts: Vec<&str> = source.split('/').collect();
        if parts.len() >= 2 {
            // Check if any part is empty or contains filesystem-specific patterns
            for part in &parts {
                if part.is_empty() || part.contains('.') || part.contains('\\') {
                    return false;
                }
            }
            // Check if the path exists on filesystem - if so, it's a local path
            if std::path::PathBuf::from(source).exists() {
                return false;
            }
            // Otherwise, assume it's a Git URL
            return true;
        }
    }

    false
}

fn main() {
    let cli = Cli::parse();

    // Handle global flags
    if cli.quiet && cli.verbose {
        eprintln!("Error: --quiet and --verbose are mutually exclusive");
        std::process::exit(1);
    }

    // Load config
    let config_path = get_config_path();
    let mut config = match Config::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };
    config.expand_paths();

    // Dispatch to command handlers
    match cli.command {
        sikil::cli::Commands::List {
            agent,
            managed,
            unmanaged,
            conflicts,
            duplicates,
        } => {
            // Parse agent filter if provided
            let agent_filter = match agent {
                Some(name) => match Agent::from_cli_name(&name) {
                    Some(a) => Some(a),
                    None => {
                        eprintln!("Error: Unknown agent '{}'", name);
                        eprintln!(
                            "Valid agents: {}",
                            Agent::all()
                                .iter()
                                .map(|a| a.cli_name())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        std::process::exit(1);
                    }
                },
                None => None,
            };

            let args = ListArgs {
                json_mode: cli.json,
                no_cache: cli.no_cache,
                agent_filter,
                managed_only: managed,
                unmanaged_only: unmanaged,
                conflicts_only: conflicts,
                duplicates_only: duplicates,
                verbose: cli.verbose,
            };
            if let Err(e) = execute_list(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Show { name } => {
            let args = ShowArgs {
                json_mode: cli.json,
                no_cache: cli.no_cache,
                name,
            };
            if let Err(e) = execute_show(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Install { source, r#to } => {
            // M3-E01-T04: Wire Install Command to CLI
            // M3-E02-T06: Wire Git URL detection to install command
            // Detect if source is a Git URL or local path and dispatch accordingly
            if is_git_url(&source) {
                // Parse agents for Git install
                let agents = if let Some(to_value) = &to {
                    // Convert --to flag value to vector of agent names
                    to_value.split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    // No --to specified, will use all enabled agents in execute_install_git
                    Vec::new()
                };

                if let Err(e) = execute_install_git(&source, agents, &config, cli.json) {
                    eprintln!("Error: {}", e);
                    std::process::exit(get_exit_code(&e));
                }
            } else {
                // Local path install
                let args = InstallArgs {
                    json_mode: cli.json,
                    path: source,
                    to,
                };
                if let Err(e) = execute_install_local(args, &config) {
                    eprintln!("Error: {}", e);
                    std::process::exit(get_exit_code(&e));
                }
            }
        }
        sikil::cli::Commands::Validate { path } => {
            let args = ValidateArgs {
                json_mode: cli.json,
                path_or_name: path,
            };
            if let Err(e) = execute_validate(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Adopt { name, from } => {
            let args = AdoptArgs {
                json_mode: cli.json,
                name,
                from,
            };
            if let Err(e) = execute_adopt(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Unmanage { name, agent, yes } => {
            let args = UnmanageArgs {
                json_mode: cli.json,
                name,
                agent,
                yes,
            };
            if let Err(e) = execute_unmanage(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Remove {
            name,
            agent,
            all,
            yes,
        } => {
            // M3-E05-T01-S02: Execute remove command
            let args = RemoveArgs {
                json_mode: cli.json,
                name,
                agent,
                all,
                yes,
            };
            if let Err(e) = execute_remove(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Sync { name, all, r#to } => {
            // M4-E01-T04: Wire Sync Command to CLI
            let args = SyncArgs {
                json_mode: cli.json,
                name,
                all,
                to,
            };
            if let Err(e) = execute_sync(args, &config, None) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Config { edit, set } => {
            let (set_key, set_value) = if set.is_empty() {
                (None, None)
            } else {
                (Some(set[0].clone()), Some(set[1].clone()))
            };

            let args = ConfigArgs {
                edit,
                set: !set.is_empty(),
                set_key,
                set_value,
                json_mode: cli.json,
            };
            if let Err(e) = execute_config(args) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
        sikil::cli::Commands::Completions { shell, output } => {
            let args = CompletionsArgs { shell, output };
            if let Err(e) = execute_completions(args) {
                eprintln!("Error: {}", e);
                std::process::exit(get_exit_code(&e));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_url_https_github() {
        assert!(is_git_url("https://github.com/owner/repo.git"));
        assert!(is_git_url("https://github.com/owner/repo"));
        assert!(is_git_url(
            "https://github.com/owner/repo.git/skills/my-skill"
        ));
    }

    #[test]
    fn test_is_git_url_short_form() {
        assert!(is_git_url("owner/repo"));
        assert!(is_git_url("owner/repo/skills/my-skill"));
        assert!(is_git_url("owner/repo/path/to/deep/skill"));
    }

    #[test]
    fn test_is_git_url_absolute_path_false() {
        assert!(!is_git_url("/home/user/skills/my-skill"));
        assert!(!is_git_url("/tmp/skill"));
    }

    #[test]
    fn test_is_git_url_relative_with_dots_false() {
        assert!(!is_git_url("./skills/my-skill"));
        assert!(!is_git_url("../other-skill"));
        assert!(!is_git_url("./skill"));
    }

    #[test]
    fn test_is_git_url_starting_with_dash_false() {
        assert!(!is_git_url("-evil-flag"));
        assert!(!is_git_url("-owner/repo"));
    }

    #[test]
    fn test_is_git_url_single_segment_false() {
        assert!(!is_git_url("skill-name"));
        assert!(!is_git_url("my-local-skill"));
    }

    #[test]
    fn test_is_git_url_non_github_https_false() {
        assert!(!is_git_url("https://gitlab.com/owner/repo.git"));
        assert!(!is_git_url("https://example.com/skill"));
    }

    #[test]
    fn test_is_git_url_file_protocol_false() {
        assert!(!is_git_url("file:///etc/passwd"));
        assert!(!is_git_url("FILE:///etc/passwd"));
    }
}
