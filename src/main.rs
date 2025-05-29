mod commands;
mod models;
mod services;

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

#[cfg(feature = "development")]
fn dev_init() {
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
