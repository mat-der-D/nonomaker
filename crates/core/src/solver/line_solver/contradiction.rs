use crate::types::Cell;

use super::super::Contradiction;
use super::bits::LineBits;

pub(crate) fn check_min_space(line: &LineBits, blocks: &[usize]) -> Result<(), Contradiction> {
    let k = blocks.len();
    if k == 0 {
        return Ok(());
    }

    let min_space = blocks.iter().sum::<usize>() + (k - 1);
    if min_space <= line.len() {
        Ok(())
    } else {
        Err(Contradiction)
    }
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
    if line.has_contradiction() {
        Err(Contradiction)
    } else {
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Grid;

    fn make_line(pattern: &str) -> LineBits {
        let n = pattern.len();
        let mut grid = Grid::new(n, 1);
        for (i, c) in pattern.chars().enumerate() {
            *grid.cell_mut(0, i) = match c {
                'F' => Cell::Filled,
                'B' => Cell::Blank,
                _ => Cell::Unknown,
            };
        }
        LineBits::from_grid_row(&grid, 0)
    }

    // --- check_min_space ---

    #[test]
    fn min_space_ok_exact_fit() {
        // ブロック [2, 2] は最小 5 マス必要、長さ 5 → OK
        assert!(check_min_space(&make_line("?????"), &[2, 2]).is_ok());
    }

    #[test]
    fn min_space_ok_empty_blocks() {
        assert!(check_min_space(&make_line("???"), &[]).is_ok());
    }

    #[test]
    fn min_space_err_line_too_short() {
        // ブロック [3] は最小 3 マス、長さ 2 → 矛盾
        assert!(check_min_space(&make_line("??"), &[3]).is_err());
    }

    #[test]
    fn min_space_err_two_blocks_no_gap() {
        // ブロック [2, 2] は最小 5 マス、長さ 4 → 矛盾
        assert!(check_min_space(&make_line("????"), &[2, 2]).is_err());
    }

    // --- check_filled_count ---

    #[test]
    fn filled_count_ok_when_exact() {
        // Filled=2, 必要=2 → OK
        let line = make_line("FF?");
        assert!(check_filled_count(&line, &[2]).is_ok());
    }

    #[test]
    fn filled_count_ok_when_more_unknown_available() {
        // Filled=1, Unknown=2, 必要=2 → OK (Unknown でカバー可能)
        let line = make_line("F??");
        assert!(check_filled_count(&line, &[2]).is_ok());
    }

    #[test]
    fn filled_count_err_when_too_many_filled() {
        // Filled=3, 必要=2 → 矛盾 (既に多すぎる)
        let line = make_line("FFF");
        assert!(check_filled_count(&line, &[2]).is_err());
    }

    #[test]
    fn filled_count_err_when_too_few_available() {
        // Filled=0, Unknown=1, 必要=2 → 矛盾 (Filled になれるセルが足りない)
        let line = make_line("?B");
        assert!(check_filled_count(&line, &[2]).is_err());
    }

    // --- check_consecutive_overflow ---

    #[test]
    fn consecutive_overflow_ok_run_equals_max_block() {
        // 連続 2 = max_block 2 → OK
        let line = make_line("FF?");
        assert!(check_consecutive_overflow(&line, &[2]).is_ok());
    }

    #[test]
    fn consecutive_overflow_err_run_exceeds_max_block() {
        // 連続 3 > max_block 2 → 矛盾
        let line = make_line("FFF");
        assert!(check_consecutive_overflow(&line, &[2]).is_err());
    }

    #[test]
    fn consecutive_overflow_ok_with_blank_separator() {
        // 連続 2, Blank, 連続 2 → max_block=2 → OK
        let line = make_line("FFBFF");
        assert!(check_consecutive_overflow(&line, &[2, 2]).is_ok());
    }

    #[test]
    fn consecutive_overflow_ok_empty_blocks() {
        // blocks が空 → チェックしない
        assert!(check_consecutive_overflow(&make_line("FFF"), &[]).is_ok());
    }

    #[test]
    fn consecutive_overflow_ok_unknown_does_not_extend_run() {
        // Unknown は Filled でないのでランをリセットする
        // 連続 Filled は 1 のみ → max_block=1 → OK
        let line = make_line("F?F");
        assert!(check_consecutive_overflow(&line, &[1, 1]).is_ok());
    }
}
