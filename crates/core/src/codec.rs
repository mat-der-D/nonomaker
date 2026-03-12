use std::io::{Read, Write};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};

use crate::types::{Cell, Clue, Grid, Puzzle};

#[derive(Debug)]
pub enum CodecError {
    InvalidGrid(String),
    InvalidId(String),
    Io(std::io::Error),
    Base64(base64::DecodeError),
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidGrid(msg) => write!(f, "invalid grid: {msg}"),
            Self::InvalidId(msg) => write!(f, "invalid id: {msg}"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Base64(err) => write!(f, "base64 decode failed: {err}"),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<std::io::Error> for CodecError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<base64::DecodeError> for CodecError {
    fn from(value: base64::DecodeError) -> Self {
        Self::Base64(value)
    }
}

pub fn grid_to_puzzle(grid: &Grid) -> Result<Puzzle, CodecError> {
    validate_grid(grid)?;

    let row_clues = (0..grid.height())
        .map(|row| {
            let cells = (0..grid.width()).map(|col| *grid.cell(row, col));
            Clue::new(compute_clue(cells))
        })
        .collect();

    let col_clues = (0..grid.width())
        .map(|col| {
            let cells = (0..grid.height()).map(|row| *grid.cell(row, col));
            Clue::new(compute_clue(cells))
        })
        .collect();

    Ok(Puzzle::new(row_clues, col_clues))
}

pub fn grid_to_id(grid: &Grid) -> Result<String, CodecError> {
    validate_grid(grid)?;

    if grid.width() > u8::MAX as usize {
        return Err(CodecError::InvalidGrid(
            "grid width must fit in one byte".to_string(),
        ));
    }

    let bit_len = grid.width() * grid.height();
    let mut payload = Vec::with_capacity(1 + bit_len);
    payload.push(grid.width() as u8);

    for row in 0..grid.height() {
        for col in 0..grid.width() {
            payload.push(if matches!(grid.cell(row, col), Cell::Filled) {
                b'1'
            } else {
                b'0'
            });
        }
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&payload)?;
    let compressed = encoder.finish()?;
    Ok(URL_SAFE_NO_PAD.encode(compressed))
}

pub fn id_to_grid(id: &str) -> Result<Grid, CodecError> {
    let compressed = URL_SAFE_NO_PAD.decode(id)?;
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut payload = Vec::new();
    decoder.read_to_end(&mut payload)?;

    let Some((&width, bit_chars)) = payload.split_first() else {
        return Err(CodecError::InvalidId("missing width byte".to_string()));
    };
    if width == 0 {
        return Err(CodecError::InvalidId("width must be positive".to_string()));
    }
    if bit_chars.is_empty() {
        return Err(CodecError::InvalidId("missing cell payload".to_string()));
    }

    let width = width as usize;
    if bit_chars.len() % width != 0 {
        return Err(CodecError::InvalidId(
            "payload bit length is incompatible with width".to_string(),
        ));
    }

    let height = bit_chars.len() / width;
    let mut grid = Grid::new(width, height);
    for (bit_index, bit_char) in bit_chars.iter().enumerate() {
        let filled = match bit_char {
            b'0' => false,
            b'1' => true,
            _ => {
                return Err(CodecError::InvalidId(
                    "payload contains non-binary data".to_string(),
                ));
            }
        };
        let row = bit_index / width;
        let col = bit_index % width;
        *grid.cell_mut(row, col) = if filled { Cell::Filled } else { Cell::Blank };
    }

    Ok(grid)
}

fn validate_grid(grid: &Grid) -> Result<(), CodecError> {
    if grid.width() == 0 || grid.height() == 0 {
        return Err(CodecError::InvalidGrid(
            "grid dimensions must be positive".to_string(),
        ));
    }

    for row in 0..grid.height() {
        for col in 0..grid.width() {
            if matches!(grid.cell(row, col), Cell::Unknown) {
                return Err(CodecError::InvalidGrid(
                    "grid contains unknown cells".to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn compute_clue(cells: impl IntoIterator<Item = Cell>) -> Vec<usize> {
    let mut clues = Vec::new();
    let mut count = 0usize;

    for cell in cells {
        if matches!(cell, Cell::Filled) {
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

#[cfg(test)]
mod tests {
    use super::{grid_to_id, grid_to_puzzle, id_to_grid};
    use crate::types::{Cell, Grid};

    fn sample_grid() -> Grid {
        let mut grid = Grid::new(3, 2);
        *grid.cell_mut(0, 0) = Cell::Filled;
        *grid.cell_mut(0, 1) = Cell::Blank;
        *grid.cell_mut(0, 2) = Cell::Filled;
        *grid.cell_mut(1, 0) = Cell::Blank;
        *grid.cell_mut(1, 1) = Cell::Filled;
        *grid.cell_mut(1, 2) = Cell::Blank;
        grid
    }

    #[test]
    fn grid_to_puzzle_computes_clues() {
        let puzzle = grid_to_puzzle(&sample_grid()).unwrap();
        assert_eq!(puzzle.row_clues()[0].blocks(), &[1, 1]);
        assert_eq!(puzzle.row_clues()[1].blocks(), &[1]);
        assert_eq!(puzzle.col_clues()[0].blocks(), &[1]);
        assert_eq!(puzzle.col_clues()[1].blocks(), &[1]);
        assert_eq!(puzzle.col_clues()[2].blocks(), &[1]);
    }

    #[test]
    fn grid_id_roundtrip() {
        let grid = sample_grid();
        let id = grid_to_id(&grid).unwrap();
        let decoded = id_to_grid(&id).unwrap();
        assert_eq!(decoded.width(), grid.width());
        assert_eq!(decoded.height(), grid.height());
        for row in 0..grid.height() {
            for col in 0..grid.width() {
                assert_eq!(decoded.cell(row, col), grid.cell(row, col));
            }
        }
    }
}
