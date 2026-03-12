use super::{PartialSolver, Solution, into_solution};
use crate::types::{Cell, Grid, Puzzle};

pub(super) fn solve<P: PartialSolver>(
    grid: Grid,
    puzzle: &Puzzle,
    partial: &P,
    max_sol: usize,
) -> Solution {
    let mut solutions = Vec::new();
    search(grid, puzzle, partial, &mut solutions, max_sol);
    into_solution(solutions)
}

fn search<P: PartialSolver>(
    grid: Grid,
    puzzle: &Puzzle,
    partial: &P,
    solutions: &mut Vec<Grid>,
    max_sol: usize,
) {
    if max_sol > 0 && solutions.len() == max_sol {
        return;
    }

    let Some((row, col)) = find_unknown(&grid) else {
        solutions.push(grid);
        return;
    };

    let mut filled = grid.clone();
    *filled.cell_mut(row, col) = Cell::Filled;
    if partial.reduce(&mut filled, puzzle).is_ok() {
        search(filled, puzzle, partial, solutions, max_sol);
    }

    let mut blank = grid;
    *blank.cell_mut(row, col) = Cell::Blank;
    if partial.reduce(&mut blank, puzzle).is_ok() {
        search(blank, puzzle, partial, solutions, max_sol);
    }
}

fn find_unknown(grid: &Grid) -> Option<(usize, usize)> {
    (0..grid.height())
        .flat_map(|row| (0..grid.width()).map(move |col| (row, col)))
        .find(|&(row, col)| *grid.cell(row, col) == Cell::Unknown)
}
