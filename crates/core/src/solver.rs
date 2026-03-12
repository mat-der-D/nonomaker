use crate::{
    types::{Grid, Puzzle},
};

mod backtracking;
mod fully_probing;
mod line_solver;
mod propagation;
mod sat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Contradiction;

pub trait PartialSolver {
    fn reduce(&self, grid: &mut Grid, puzzle: &Puzzle) -> Result<(), Contradiction>;

    fn solve_partial(&self, puzzle: &Puzzle) -> Option<Grid> {
        let mut grid = Grid::new(puzzle.width(), puzzle.height());
        self.reduce(&mut grid, puzzle).ok()?;
        Some(grid)
    }
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
    fn reduce(&self, grid: &mut Grid, puzzle: &Puzzle) -> Result<(), Contradiction> {
        propagation::reduce(grid, puzzle)
    }
}

#[derive(Debug, Clone)]
pub struct Fp1Solver;

impl PartialSolver for Fp1Solver {
    fn reduce(&self, grid: &mut Grid, puzzle: &Puzzle) -> Result<(), Contradiction> {
        fully_probing::reduce(grid, puzzle, fully_probing::FullyProbingMode::Fp1)
    }
}

#[derive(Debug, Clone)]
pub struct Fp2Solver;

impl PartialSolver for Fp2Solver {
    fn reduce(&self, grid: &mut Grid, puzzle: &Puzzle) -> Result<(), Contradiction> {
        fully_probing::reduce(grid, puzzle, fully_probing::FullyProbingMode::Fp2)
    }
}

#[derive(Debug, Clone)]
pub struct BacktrackingSolver<P = PropagationSolver> {
    partial: P,
    max_sol: usize,
}

impl BacktrackingSolver {
    pub fn new(max_sol: usize) -> Self {
        Self {
            partial: PropagationSolver,
            max_sol,
        }
    }
}

impl<P> BacktrackingSolver<P> {
    pub fn with_partial(max_sol: usize, partial: P) -> Self {
        Self { partial, max_sol }
    }
}

impl<P: PartialSolver> CompleteSolver for BacktrackingSolver<P> {
    fn solve_complete(&self, puzzle: &Puzzle) -> Solution {
        let Some(grid) = self.partial.solve_partial(puzzle) else {
            return Solution::None;
        };
        backtracking::solve(grid, puzzle, &self.partial, self.max_sol)
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
