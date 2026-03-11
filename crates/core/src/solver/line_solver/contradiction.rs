use crate::types::Cell;

use super::bits::LineBits;
use super::super::Contradiction;

pub(crate) fn check_min_space(line: &LineBits, blocks: &[usize]) -> Result<(), Contradiction> {
    let k = blocks.len();
    if k == 0 {
        return Ok(());
    }

    let min_space = blocks.iter().sum::<usize>() + (k - 1);
    if min_space <= line.len() { Ok(()) } else { Err(Contradiction) }
}

pub(crate) fn check_filled_count(line: &LineBits, blocks: &[usize]) -> Result<(), Contradiction> {
    let req_filled = blocks.iter().sum();
    let n_filled = line.count_cells(Cell::Filled);
    let n_unknown = line.count_cells(Cell::Unknown);
    if (n_filled <= req_filled) && (n_filled + n_unknown >= req_filled) {
        Ok(())
    } else {
        Err(Contradiction)
    }
}

pub(crate) fn check_no_dead_cells(line: &LineBits) -> Result<(), Contradiction> {
    if line.has_contradiction() { Err(Contradiction) } else { Ok(()) }
}

pub(crate) fn check_consecutive_overflow(
    line: &LineBits,
    blocks: &[usize],
) -> Result<(), Contradiction> {
    let Some(&max_block) = blocks.iter().max() else {
        // blocks が空のケースは check_filled_count と重複するので pass
        return Ok(());
    };

    let mut current_run = 0;
    for cell in line.cells() {
        current_run = if cell == Cell::Filled {
            current_run + 1
        } else {
            0
        };

        if current_run > max_block {
            return Err(Contradiction);
        }
    }
    Ok(())
}
