use clap::Parser;

/// Sikil - A CLI tool for managing AI coding assistant skills
#[derive(Parser, Debug)]
#[command(name = "sikil")]
#[command(author = "Your Name <you@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "Manage AI coding assistant skills across multiple agents", long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long)]
    config: Option<String>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path);
    }

    if cli.debug {
        println!("Debug mode is ON");
    }

    // You can check the value of the arguments and handle them accordingly
    println!("Sikil - Skill Management Tool");
}
