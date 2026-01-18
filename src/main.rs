use clap::Parser;
use sikil::cli::Cli;
use sikil::commands::{execute_list, ListArgs};
use sikil::core::config::Config;
use sikil::core::skill::Agent;
use sikil::utils::paths::get_config_path;

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
            };
            if let Err(e) = execute_list(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        sikil::cli::Commands::Show { name } => {
            println!("Show command for: {}", name);
        }
        sikil::cli::Commands::Install { source, r#to } => {
            println!("Install command from: {:?}", source);
            if let Some(agents) = r#to {
                println!("  to agents: {:?}", agents);
            }
        }
        sikil::cli::Commands::Validate { path } => {
            println!("Validate command for: {:?}", path);
        }
        sikil::cli::Commands::Adopt { name, from } => {
            println!("Adopt command for: {:?}", name);
            if let Some(agent) = from {
                println!("  from agent: {:?}", agent);
            }
        }
        sikil::cli::Commands::Unmanage { name, agent, yes } => {
            println!("Unmanage command for: {:?}", name);
            if let Some(a) = agent {
                println!("  agent: {:?}", a);
            }
            if yes {
                println!("  (skip confirmation)");
            }
        }
        sikil::cli::Commands::Remove {
            name,
            agent,
            all,
            yes,
        } => {
            println!("Remove command for: {:?}", name);
            if let Some(a) = agent {
                println!("  agent: {:?}", a);
            }
            if all {
                println!("  (remove all)");
            }
            if yes {
                println!("  (skip confirmation)");
            }
        }
        sikil::cli::Commands::Sync { name, all, r#to } => {
            if all {
                println!("Sync all managed skills");
            } else if let Some(skill_name) = name {
                println!("Sync command for: {:?}", skill_name);
            }
            if let Some(agents) = r#to {
                println!("  to agents: {:?}", agents);
            }
        }
        sikil::cli::Commands::Config { edit, set } => {
            if edit {
                println!("Config edit - not yet implemented");
            }
            if set {
                println!("Config set - not yet implemented");
            }
            if !edit && !set {
                println!("Config show - not yet implemented");
            }
        }
        sikil::cli::Commands::Completions { shell, output } => {
            println!("Completions for: {:?}", shell);
            if let Some(out) = output {
                println!("  output to: {:?}", out);
            }
        }
    }
}
