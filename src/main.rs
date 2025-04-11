mod commands;
mod models;
mod services;
mod traits;

use clap::Parser;
use colored::Colorize;
use commands::Exec;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    let args = Args::parse();

    println!("Args: {:?}", args);
    args.command.show();

    if let Err(e) = args.command.exec() {
        println!("{}: {e}", "error".red());
    }
}
