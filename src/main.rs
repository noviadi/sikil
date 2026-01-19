use clap::Parser;
use sikil::cli::Cli;
use sikil::commands::{
    execute_adopt, execute_install_local, execute_list, execute_remove, execute_show,
    execute_unmanage, execute_validate, AdoptArgs, InstallArgs, ListArgs, RemoveArgs, ShowArgs,
    UnmanageArgs, ValidateArgs,
};
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
            let args = ShowArgs {
                json_mode: cli.json,
                no_cache: cli.no_cache,
                name,
            };
            if let Err(e) = execute_show(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        sikil::cli::Commands::Install { source, r#to } => {
            // M3-E01-T04: Wire Install Command to CLI
            // For now, only local paths are supported (Git install will be M3-E02)
            let args = InstallArgs {
                json_mode: cli.json,
                path: source,
                to,
            };
            if let Err(e) = execute_install_local(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        sikil::cli::Commands::Validate { path } => {
            let args = ValidateArgs {
                json_mode: cli.json,
                path_or_name: path,
            };
            if let Err(e) = execute_validate(args, &config) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
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
                std::process::exit(1);
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
                std::process::exit(1);
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
                std::process::exit(1);
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
