use clap::CommandFactory;
use clap_mangen::Man;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = sikil::cli::app::Cli::command();
    let man = Man::new(cmd);

    let mut buffer = Vec::new();
    man.render(&mut buffer)?;

    // Write man page to current directory
    let man_page_path = Path::new("sikil.1");
    fs::write(man_page_path, buffer)?;

    println!("Man page generated: sikil.1");

    Ok(())
}
