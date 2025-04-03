mod commands;
mod services;

use clap::Parser;
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
    args.command.exec();
}
