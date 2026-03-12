mod commands;
mod error;
mod io;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nonomaker", about = "Nonogram puzzle tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve a nonogram puzzle
    Solve(commands::solve::SolveArgs),
    /// Convert an image into a dot-grid JSON
    Convert(commands::convert::ConvertArgs),
    /// Convert a dot-grid JSON into puzzle clues JSON
    GridToPuzzle(commands::grid_to_puzzle::GridToPuzzleArgs),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Solve(args) => commands::solve::run(args),
        Commands::Convert(args) => commands::convert::run(args),
        Commands::GridToPuzzle(args) => commands::grid_to_puzzle::run(args),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
