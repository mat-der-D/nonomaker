use std::collections::VecDeque;

use super::Contradiction;
use super::line_solver::{LineBits, solve_line};
use crate::types::{Grid, Puzzle};

#[derive(Debug, Clone, Copy)]
struct LineIdEncoder {
    height: usize,
}

impl LineIdEncoder {
    fn new(grid: &Grid) -> Self {
        Self {
            height: grid.height(),
        }
    }

    // encode_col との対称性のため &self を取る
    #[allow(unused_variables)]
    fn encode_row(&self, row: usize) -> usize {
        row
    }

    fn encode_col(&self, col: usize) -> usize {
        self.height + col
    }

    fn is_row(&self, line_id: usize) -> bool {
        line_id < self.height
    }

    fn decode(&self, line_id: usize) -> Decode {
        if self.is_row(line_id) {
            Decode::Row(line_id)
        } else {
            Decode::Col(line_id - self.height)
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Decode {
    Col(usize),
    Row(usize),
}

pub(crate) fn reduce(grid: &mut Grid, puzzle: &Puzzle) -> Result<(), Contradiction> {
    let mut propagator = Propagator::new(grid, puzzle);
    propagator.reduce()
}

#[derive(Debug)]
struct Propagator<'a> {
    grid: &'a mut Grid,
    puzzle: &'a Puzzle,
    id_encoder: LineIdEncoder,
    id_queue: VecDeque<usize>,
    in_id_queue: Vec<bool>,
    solved: Vec<bool>,
}

impl<'a> Propagator<'a> {
    fn new(grid: &'a mut Grid, puzzle: &'a Puzzle) -> Self {
        let id_encoder = LineIdEncoder::new(grid);
        let n_lines = grid.width() + grid.height();
        Self {
            grid,
            puzzle,
            id_encoder,
            id_queue: (0..n_lines).collect(),
            in_id_queue: vec![true; n_lines],
            solved: vec![false; n_lines],
        }
    }

    fn enqueue(&mut self, line_id: usize) {
        if self.solved[line_id] {
            return;
        }
        if !self.in_id_queue[line_id] {
            self.id_queue.push_back(line_id);
            self.in_id_queue[line_id] = true;
        }
    }

    fn dequeue(&mut self) -> Option<usize> {
        let popped = self.id_queue.pop_front();
        if let Some(line_id) = popped {
            self.in_id_queue[line_id] = false;
        }
        popped
    }

    fn enqueue_cross_lines(&mut self, line_id: usize, changed_cells: &[usize]) {
        for &cell_pos in changed_cells {
            let cross_id = Self::get_cross_id(&self.id_encoder, line_id, cell_pos);
            self.enqueue(cross_id);
        }
    }

    fn get_cross_id(id_encoder: &LineIdEncoder, line_id: usize, cell_pos: usize) -> usize {
        if id_encoder.is_row(line_id) {
            let col = cell_pos;
            id_encoder.encode_col(col)
        } else {
            let row = cell_pos;
            id_encoder.encode_row(row)
        }
    }

    fn reduce(&mut self) -> Result<(), Contradiction> {
        while let Some(line_id) = self.dequeue() {
            if self.solved[line_id] {
                continue;
            }

            let (changed_cells, solved) = self.solve_line(line_id)?;

            if !changed_cells.is_empty() {
                self.enqueue_cross_lines(line_id, &changed_cells);
            }

            if solved {
                self.solved[line_id] = true;
            }
        }
        Ok(())
    }

    fn solve_line(&mut self, line_id: usize) -> Result<(Vec<usize>, bool), Contradiction> {
        let decoded = self.id_encoder.decode(line_id);
        let (mut line, blocks) = match decoded {
            Decode::Row(row) => {
                let line = LineBits::from_grid_row(self.grid, row);
                let blocks = self.puzzle.row_clues()[row].blocks();
                (line, blocks)
            }
            Decode::Col(col) => {
                let line = LineBits::from_grid_col(self.grid, col);
                let blocks = self.puzzle.col_clues()[col].blocks();
                (line, blocks)
            }
        };

        let changed_cells = solve_line(&mut line, blocks)?;

        for &cell_pos in changed_cells.iter() {
            let (row, col) = match decoded {
                Decode::Row(row) => (row, cell_pos),
                Decode::Col(col) => (cell_pos, col),
            };
            *self.grid.cell_mut(row, col) = line.cell(cell_pos);
        }
        Ok((changed_cells, line.is_solved()))
    }
}
