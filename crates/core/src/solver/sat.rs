use splr::Certificate;

use super::{Solution, into_solution};
use crate::types::{Cell, Grid, Puzzle};

pub(super) fn solve(puzzle: &Puzzle, max_sol: usize) -> Solution {
    let mut encoder = Encoder::new(puzzle);
    encoder.encode();

    let mut solutions = Vec::new();
    loop {
        if max_sol > 0 && solutions.len() == max_sol {
            break;
        }

        match Certificate::try_from(encoder.clauses.clone()) {
            Ok(Certificate::UNSAT) => break,
            Ok(Certificate::SAT(model)) => {
                let grid = encoder.grid_from_model(&model);
                encoder.add_blocking_clause(&grid);
                solutions.push(grid);
            }
            Err(err) => panic!("SAT solver failed: {err}"),
        }
    }

    into_solution(solutions)
}

#[derive(Debug)]
struct Encoder<'a> {
    puzzle: &'a Puzzle,
    clauses: Vec<Vec<i32>>,
    next_var: i32,
    cells: Vec<i32>,
}

impl<'a> Encoder<'a> {
    fn new(puzzle: &'a Puzzle) -> Self {
        let n_cells = puzzle.width() * puzzle.height();
        let mut next_var = 1_i32;
        let cells = (0..n_cells)
            .map(|_| {
                let var = next_var;
                next_var += 1;
                var
            })
            .collect();
        Self {
            puzzle,
            clauses: Vec::new(),
            next_var,
            cells,
        }
    }

    fn encode(&mut self) {
        for row in 0..self.puzzle.height() {
            let vars = (0..self.puzzle.width())
                .map(|col| self.cell_var(row, col))
                .collect::<Vec<_>>();
            self.encode_line(&vars, self.puzzle.row_clues()[row].blocks());
        }

        for col in 0..self.puzzle.width() {
            let vars = (0..self.puzzle.height())
                .map(|row| self.cell_var(row, col))
                .collect::<Vec<_>>();
            self.encode_line(&vars, self.puzzle.col_clues()[col].blocks());
        }
    }

    fn encode_line(&mut self, cells: &[i32], blocks: &[usize]) {
        let automaton = Automaton::from_blocks(blocks);
        let state_vars = self.alloc_state_vars(cells.len(), automaton.states.len());

        self.encode_start_state(&state_vars, automaton.start);
        self.encode_state_uniqueness(&state_vars);
        self.encode_transitions(&automaton, cells, &state_vars);
        self.encode_accepting_states(&automaton, &state_vars, cells.len());
    }

    fn add_blocking_clause(&mut self, grid: &Grid) {
        let mut clause = Vec::with_capacity(grid.width() * grid.height());
        for row in 0..grid.height() {
            for col in 0..grid.width() {
                let var = self.cell_var(row, col);
                clause.push(match grid.cell(row, col) {
                    Cell::Filled => -var,
                    Cell::Blank => var,
                    Cell::Unknown => unreachable!("SAT solver only produces complete grids"),
                });
            }
        }
        self.clauses.push(clause);
    }

    fn grid_from_model(&self, model: &[i32]) -> Grid {
        let mut values = vec![false; self.next_var as usize];
        for &lit in model {
            let index = lit.unsigned_abs() as usize;
            if index < values.len() {
                values[index] = lit > 0;
            }
        }

        let mut grid = Grid::new(self.puzzle.width(), self.puzzle.height());
        for row in 0..self.puzzle.height() {
            for col in 0..self.puzzle.width() {
                let var = self.cell_var(row, col) as usize;
                *grid.cell_mut(row, col) = if values[var] {
                    Cell::Filled
                } else {
                    Cell::Blank
                };
            }
        }
        grid
    }

    fn exactly_one(&mut self, vars: &[i32]) {
        self.clauses.push(vars.to_vec());
        for (i, &lhs) in vars.iter().enumerate() {
            for &rhs in vars.iter().skip(i + 1) {
                self.clauses.push(vec![-lhs, -rhs]);
            }
        }
    }

    fn alloc_var(&mut self) -> i32 {
        let var = self.next_var;
        self.next_var += 1;
        var
    }

    fn cell_var(&self, row: usize, col: usize) -> i32 {
        self.cells[row * self.puzzle.width() + col]
    }

    fn alloc_state_vars(&mut self, line_len: usize, n_states: usize) -> Vec<Vec<i32>> {
        (0..=line_len)
            .map(|_| (0..n_states).map(|_| self.alloc_var()).collect())
            .collect()
    }

    fn encode_start_state(&mut self, state_vars: &[Vec<i32>], start_state: usize) {
        for (state, &var) in state_vars[0].iter().enumerate() {
            self.clauses
                .push(vec![if state == start_state { var } else { -var }]);
        }
    }

    fn encode_state_uniqueness(&mut self, state_vars: &[Vec<i32>]) {
        for vars in state_vars {
            self.exactly_one(vars);
        }
    }

    fn encode_transitions(
        &mut self,
        automaton: &Automaton,
        cells: &[i32],
        state_vars: &[Vec<i32>],
    ) {
        for (pos, &cell_var) in cells.iter().enumerate() {
            let current_states = &state_vars[pos];
            let next_states = &state_vars[pos + 1];
            for (state, trans) in automaton.states.iter().enumerate() {
                let current = current_states[state];
                self.encode_symbol_transition(
                    current,
                    cell_var,
                    next_states,
                    trans.on_blank,
                    false,
                );
                self.encode_symbol_transition(
                    current,
                    cell_var,
                    next_states,
                    trans.on_filled,
                    true,
                );
            }
        }
    }

    fn encode_symbol_transition(
        &mut self,
        current_state_var: i32,
        cell_var: i32,
        next_states: &[i32],
        next_state: Option<usize>,
        filled: bool,
    ) {
        let cell_literal = if filled { -cell_var } else { cell_var };
        match next_state {
            Some(next) => {
                self.clauses
                    .push(vec![-current_state_var, cell_literal, next_states[next]])
            }
            None => self.clauses.push(vec![-current_state_var, cell_literal]),
        }
    }

    fn encode_accepting_states(
        &mut self,
        automaton: &Automaton,
        state_vars: &[Vec<i32>],
        line_len: usize,
    ) {
        let accept_clause = automaton
            .accepting
            .iter()
            .map(|&state| state_vars[line_len][state])
            .collect::<Vec<_>>();
        self.clauses.push(accept_clause);
    }
}

#[derive(Debug)]
struct Automaton {
    states: Vec<StateTransitions>,
    start: usize,
    accepting: Vec<usize>,
}

impl Automaton {
    fn from_blocks(blocks: &[usize]) -> Self {
        let mut states = Vec::new();
        let mut gap_states = Vec::with_capacity(blocks.len() + 1);
        let mut filled_states = Vec::with_capacity(blocks.len());

        for &block_len in blocks {
            gap_states.push(states.len());
            states.push(StateTransitions {
                on_blank: Some(gap_states.last().copied().unwrap()),
                on_filled: None,
            });

            let start = states.len();
            for progress in 1..=block_len {
                states.push(StateTransitions {
                    on_blank: None,
                    on_filled: None,
                });
                filled_states.push((start, block_len));
                if progress == block_len {
                    break;
                }
            }
        }

        gap_states.push(states.len());
        let trailing_gap = gap_states[blocks.len()];
        states.push(StateTransitions {
            on_blank: Some(trailing_gap),
            on_filled: None,
        });

        for (block_index, &gap_state) in gap_states.iter().take(blocks.len()).enumerate() {
            states[gap_state].on_filled = Some(filled_state_index(blocks, block_index, 1));
        }

        for (block_index, &block_len) in blocks.iter().enumerate() {
            for progress in 1..=block_len {
                let state = filled_state_index(blocks, block_index, progress);
                if progress < block_len {
                    states[state].on_filled =
                        Some(filled_state_index(blocks, block_index, progress + 1));
                } else {
                    states[state].on_blank = Some(gap_states[block_index + 1]);
                }
            }
        }

        let mut accepting = vec![trailing_gap];
        if let Some(last_block) = blocks.last() {
            accepting.push(filled_state_index(blocks, blocks.len() - 1, *last_block));
        }

        Self {
            states,
            start: gap_states[0],
            accepting,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct StateTransitions {
    on_blank: Option<usize>,
    on_filled: Option<usize>,
}

fn filled_state_index(blocks: &[usize], block_index: usize, progress: usize) -> usize {
    let filled_before = blocks.iter().take(block_index).sum::<usize>();
    block_index + filled_before + progress
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Clue, Puzzle};

    #[test]
    fn sat_solver_finds_unique_solution() {
        let puzzle = Puzzle::new(
            vec![Clue::new(vec![2]), Clue::new(vec![])],
            vec![Clue::new(vec![1]), Clue::new(vec![1])],
        );

        let solution = solve(&puzzle, 2);
        assert!(matches!(solution, Solution::Unique(_)));
    }

    #[test]
    fn sat_solver_detects_multiple_solutions() {
        let puzzle = Puzzle::new(
            vec![Clue::new(vec![1]), Clue::new(vec![1])],
            vec![Clue::new(vec![1]), Clue::new(vec![1])],
        );

        let solution = solve(&puzzle, 2);
        assert!(matches!(solution, Solution::Multiple(ref grids) if grids.len() == 2));
    }

    #[test]
    fn sat_solver_detects_unsat_puzzle() {
        let puzzle = Puzzle::new(vec![Clue::new(vec![1])], vec![Clue::new(vec![])]);

        assert!(matches!(solve(&puzzle, 2), Solution::None));
    }
}
