#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    Unknown,
    Filled,
    Blank,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            cells: vec![Cell::Unknown; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.cells.len() / self.width
    }

    fn cell_index(&self, row: usize, col: usize) -> usize {
        if col >= self.width {
            panic!()
        }

        if row >= self.height() {
            panic!()
        }

        row * self.width + col
    }

    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        let index = self.cell_index(row, col);
        &self.cells[index]
    }
}

#[derive(Debug, Clone)]
pub struct Clue {
    blocks: Vec<usize>,
}

impl Clue {
    pub fn new(blocks: Vec<usize>) -> Self {
        Self { blocks }
    }
}

#[derive(Debug, Clone)]
pub struct Puzzle {
    row_clues: Vec<Clue>,
    col_clues: Vec<Clue>,
}

impl Puzzle {
    pub fn new(row_clues: Vec<Clue>, col_clues: Vec<Clue>) -> Self {
        Self {
            row_clues,
            col_clues,
        }
    }
}
