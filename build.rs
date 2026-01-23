use std::env;
use std::fs;
use std::path::Path;

use clap::CommandFactory;
use clap_mangen::Man;

// Include the real CLI definition in a module to avoid conflicts
#[path = "src/cli/app.rs"]
mod cli;

fn main() {
    // Rerun when CLI definition or this script changes
    println!("cargo::rerun-if-changed=src/cli/app.rs");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=SIKIL_GENERATE_MAN");

    // Only generate man page in release builds or when explicitly requested
    let profile = env::var("PROFILE").unwrap_or_default();
    let gen_man = matches!(
        env::var("SIKIL_GENERATE_MAN").as_deref(),
        Ok("1") | Ok("true")
    );

    if profile != "release" && !gen_man {
        return;
    }

    generate_man_page();
}

fn generate_man_page() {
    let cmd = cli::Cli::command();
    let man = Man::new(cmd);

    let mut buffer = Vec::new();
    if let Err(e) = man.render(&mut buffer) {
        println!("cargo::warning=Failed to render man page: {}", e);
        return;
    }

    let out_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let man_page_path = Path::new(&out_dir).join("sikil.1");

    // Only write if contents changed to avoid dirty working tree
    if let Ok(existing) = fs::read(&man_page_path) {
        if existing == buffer {
            return;
        }
    }

    if let Err(e) = fs::write(&man_page_path, buffer) {
        println!("cargo::warning=Failed to write man page: {}", e);
        return;
    }

    println!("cargo::warning=Generated man page: sikil.1");
}
