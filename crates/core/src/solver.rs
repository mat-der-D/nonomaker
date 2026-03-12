use crate::{
    solver::propagation::propagate,
    types::{Grid, Puzzle},
};

mod backtracking;
mod fully_probing;
mod line_solver;
mod propagation;
mod sat;

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

fn into_solution(solutions: Vec<Grid>) -> Solution {
    match solutions.len() {
        0 => Solution::None,
        1 => Solution::Unique(solutions.into_iter().next().unwrap()),
        _ => Solution::Multiple(solutions),
    }
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

#[derive(Debug, Clone)]
pub struct Fp1Solver;

impl PartialSolver for Fp1Solver {
    fn solve_partial(&self, puzzle: &Puzzle) -> Option<Grid> {
        fully_probing::solve(puzzle, fully_probing::FullyProbingMode::Fp1)
    }
}

#[derive(Debug, Clone)]
pub struct Fp2Solver;

impl PartialSolver for Fp2Solver {
    fn solve_partial(&self, puzzle: &Puzzle) -> Option<Grid> {
        fully_probing::solve(puzzle, fully_probing::FullyProbingMode::Fp2)
    }
}

#[derive(Debug, Clone)]
pub struct BacktrackingSolver {
    max_sol: usize,
}

impl BacktrackingSolver {
    pub fn new(max_sol: usize) -> Self {
        Self { max_sol }
    }
}

impl CompleteSolver for BacktrackingSolver {
    fn solve_complete(&self, puzzle: &Puzzle) -> Solution {
        let Some(grid) = PropagationSolver.solve_partial(puzzle) else {
            return Solution::None;
        };
        backtracking::solve(grid, puzzle, self.max_sol)
    }
}

#[derive(Debug, Clone)]
pub struct SatSolver {
    max_sol: usize,
}

impl SatSolver {
    pub fn new(max_sol: usize) -> Self {
        Self { max_sol }
    }
}

impl CompleteSolver for SatSolver {
    fn solve_complete(&self, puzzle: &Puzzle) -> Solution {
        sat::solve(puzzle, self.max_sol)
    }
}
