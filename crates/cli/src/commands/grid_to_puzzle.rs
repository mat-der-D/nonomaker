use std::path::PathBuf;

use clap::Args;

use crate::error::CliError;
use crate::io::{read_input, write_output};

#[derive(Args, Debug)]
pub struct GridToPuzzleArgs {
    /// Input dot-grid JSON path (stdin when omitted)
    #[arg(long, value_name = "PATH")]
    pub input: Option<PathBuf>,
    /// Output file path (stdout when omitted)
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,
}

fn compute_clue(line: &[bool]) -> Vec<usize> {
    let mut clues = Vec::new();
    let mut count = 0usize;

    for &cell in line {
        if cell {
            count += 1;
        } else if count > 0 {
            clues.push(count);
            count = 0;
        }
    }

    if count > 0 {
        clues.push(count);
    }

    clues
}

pub fn run(args: GridToPuzzleArgs) -> Result<(), CliError> {
    let input = read_input(args.input.as_deref())?;
    let grid: Vec<Vec<bool>> =
        serde_json::from_str(&input).map_err(|e| CliError::Parse(e.to_string()))?;

    let cols = validate_grid(&grid)?;

    let rows = grid.len();
    let row_clues: Vec<Vec<usize>> = grid.iter().map(|row| compute_clue(row)).collect();
    let col_clues: Vec<Vec<usize>> = (0..cols)
        .map(|c| {
            let col: Vec<bool> = (0..rows).map(|r| grid[r][c]).collect();
            compute_clue(&col)
        })
        .collect();

    let output = serde_json::json!({
        "row_clues": row_clues,
        "col_clues": col_clues,
    })
    .to_string();

    write_output(args.output.as_deref(), &output)
}

fn validate_grid(grid: &[Vec<bool>]) -> Result<usize, CliError> {
    if grid.is_empty() {
        return Err(CliError::Parse("grid must not be empty".to_string()));
    }

    let cols = grid[0].len();
    if cols == 0 {
        return Err(CliError::Parse("grid rows must not be empty".to_string()));
    }

    let is_rectangular = grid.iter().all(|row| row.len() == cols);
    if !is_rectangular {
        return Err(CliError::Parse(
            "all rows must have the same length".to_string(),
        ));
    }

    Ok(cols)
}

#[cfg(test)]
mod tests {
    use super::compute_clue;

    #[test]
    fn compute_clue_empty_line() {
        assert_eq!(compute_clue(&[]), Vec::<usize>::new());
    }

    #[test]
    fn compute_clue_all_blank() {
        assert_eq!(compute_clue(&[false, false, false]), Vec::<usize>::new());
    }

    #[test]
    fn compute_clue_single_block() {
        assert_eq!(compute_clue(&[true, true, true]), vec![3]);
    }

    #[test]
    fn compute_clue_multiple_blocks() {
        assert_eq!(
            compute_clue(&[true, false, true, true, false, true]),
            vec![1, 2, 1]
        );
    }
}
