use crate::{
    solver::propagation::propagate,
    types::{Grid, Puzzle},
};

mod line_solver;
mod propagation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Contradiction;

pub trait PartialSolver {
    fn solve_partial(&self, puzzle: &Puzzle) -> Option<Grid>;
}

pub trait CompleteSolver {
    fn solve_complete(&self, puzzle: &Puzzle) -> Solution;
}

pub enum Solution {
    None,
    Unique(Grid),
    Multiple(Vec<Grid>),
}

#[derive(Debug, Clone)]
pub struct PropagationSolver;

impl PartialSolver for PropagationSolver {
    fn solve_partial(&self, puzzle: &Puzzle) -> Option<Grid> {
        let mut grid = Grid::new(puzzle.width(), puzzle.height());
        let valid = propagate(&mut grid, puzzle);
        if valid { Some(grid) } else { None }
    }
}
