use super::{Solution, propagation::propagate};
use crate::types::{Cell, Grid, Puzzle};

pub(super) fn solve(grid: Grid, puzzle: &Puzzle, max_sol: usize) -> Solution {
    let mut solutions = Vec::new();
    search(grid, puzzle, &mut solutions, max_sol);
    into_solution(solutions)
}

fn search(grid: Grid, puzzle: &Puzzle, solutions: &mut Vec<Grid>, max_sol: usize) {
    if max_sol > 0 && solutions.len() == max_sol {
        return;
    }

    let Some((row, col)) = find_unknown(&grid) else {
        solutions.push(grid);
        return;
    };

    let mut filled = grid.clone();
    *filled.cell_mut(row, col) = Cell::Filled;
    if propagate(&mut filled, puzzle) {
        search(filled, puzzle, solutions, max_sol);
    }

    let mut blank = grid;
    *blank.cell_mut(row, col) = Cell::Blank;
    if propagate(&mut blank, puzzle) {
        search(blank, puzzle, solutions, max_sol);
    }
}

fn find_unknown(grid: &Grid) -> Option<(usize, usize)> {
    (0..grid.height())
        .flat_map(|row| (0..grid.width()).map(move |col| (row, col)))
        .find(|&(row, col)| *grid.cell(row, col) == Cell::Unknown)
}

fn into_solution(solutions: Vec<Grid>) -> Solution {
    match solutions.len() {
        0 => Solution::None,
        1 => Solution::Unique(solutions.into_iter().next().unwrap()),
        _ => Solution::Multiple(solutions),
    }
}
