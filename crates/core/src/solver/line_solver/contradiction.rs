use crate::types::Cell;

use super::bits::LineBits;

pub(crate) fn check_min_space(line: &LineBits, blocks: &[usize]) -> bool {
    let k = blocks.len();
    if k == 0 {
        return true;
    }

    let min_space = blocks.iter().sum::<usize>() + (k - 1);
    min_space <= line.len()
}

pub(crate) fn check_filled_count(line: &LineBits, blocks: &[usize]) -> bool {
    let req_filled = blocks.iter().sum();
    let n_filled = line.count_cells(Cell::Filled);
    let n_unknown = line.count_cells(Cell::Unknown);
    (n_filled <= req_filled) && (n_filled + n_unknown >= req_filled)
}

pub(crate) fn check_consecutive_overflow(line: &LineBits, blocks: &[usize]) -> bool {
    let Some(&max_block) = blocks.iter().max() else {
        // blocks が空のケースは check_filled_count と重複するので pass
        return true;
    };

    let mut current_run = 0;
    for cell in line.cells() {
        current_run = if cell == Cell::Filled {
            current_run + 1
        } else {
            0
        };

        if current_run > max_block {
            return false;
        }
    }
    true
}
