mod commands;
mod models;
mod services;
mod traits;

use clap::Parser;
use colored::Colorize;
use commands::Exec;
use log::debug;
use simple_logger::SimpleLogger;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    let args = Args::parse();

    SimpleLogger::new()
        .with_colors(true)
        .without_timestamps()
        .init()
        .unwrap();

    debug!("Args: {:?}", args);
    debug!("Command: {:?}", args.command);

    if let Err(e) = args.command.exec() {
        println!("{}: {e}", "error".red());
    }
}
