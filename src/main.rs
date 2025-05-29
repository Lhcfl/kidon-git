mod commands;
mod models;
mod services;

use clap::Parser;
use colored::Colorize;
use commands::Exec;
use log::debug;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: commands::Commands,
}

#[cfg(feature = "development")]
fn dev_init() {
    use simple_logger::SimpleLogger;
    
    SimpleLogger::new()
        .with_colors(true)
        .without_timestamps()
        .init()
        .unwrap();
}

fn main() {
    let args = Args::parse();

    #[cfg(feature = "development")]
    dev_init();

    debug!("Args: {args:?}");
    debug!("Command: {:?}", args.command);

    if let Err(e) = args.command.exec() {
        println!("{}: {e}", "error".red());
    }
}
