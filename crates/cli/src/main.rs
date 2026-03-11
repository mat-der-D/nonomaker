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
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Solve(args) => commands::solve::run(args),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
