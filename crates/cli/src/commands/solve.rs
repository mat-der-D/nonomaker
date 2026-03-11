use std::path::PathBuf;

use clap::{Args, ValueEnum};
use nonomaker_core::{
    format::{partial_grid_to_json, puzzle_from_json, solution_to_json},
    solver::{BacktrackingSolver, CompleteSolver, PartialSolver, PropagationSolver},
};

use crate::{
    error::CliError,
    io::{read_input, write_output},
};

#[derive(ValueEnum, Clone)]
pub enum Solver {
    Linear,
    Backtracking,
}

#[derive(Args)]
pub struct SolveArgs {
    #[arg(long, default_value = "backtracking")]
    pub solver: Solver,
    #[arg(short, long)]
    pub input: Option<PathBuf>,
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

pub fn run(args: SolveArgs) -> Result<(), CliError> {
    let input = read_input(args.input.as_deref())?;
    let puzzle = puzzle_from_json(&input)?;
    let json = match args.solver {
        Solver::Backtracking => {
            let solution = BacktrackingSolver.solve_complete(&puzzle);
            solution_to_json(&solution)?
        }
        Solver::Linear => match PropagationSolver.solve_partial(&puzzle) {
            None => r#"{"status":"contradiction"}"#.to_string(),
            Some(grid) => {
                format!(
                    r#"{{"status":"ok","grid":{}}}"#,
                    partial_grid_to_json(&grid)
                )
            }
        },
    };
    write_output(args.output.as_deref(), &json)
}
