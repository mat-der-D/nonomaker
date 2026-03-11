mod bits;
mod contradiction;
mod dp;
mod earliest_latest;
mod segment;

pub(super) use bits::LineBits;
use contradiction::{
    check_consecutive_overflow, check_filled_count, check_min_space, check_no_dead_cells,
};
use dp::dp_solve;
use earliest_latest::earliest_latest_inference;
use segment::segment_phase;

use crate::types::Cell;

use super::Contradiction;

pub(crate) fn solve_line(
    line: &mut LineBits,
    blocks: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    // 空ブロック: 全セルを Blank に確定
    if blocks.is_empty() {
        if line.count_cells(Cell::Filled) != 0 {
            return Err(Contradiction);
        }
        let unknowns: Vec<usize> = (0..line.len())
            .filter(|&i| line.cell(i) == Cell::Unknown)
            .collect();
        line.set_cells(&unknowns, Cell::Blank);
        return Ok(unknowns);
    }

    // Phase 1: 軽量矛盾チェック
    check_min_space(line, blocks)?;
    check_filled_count(line, blocks)?;
    check_consecutive_overflow(line, blocks)?;

    // Phase 2: Segment 分割とブロック割り当て
    let mut changed = segment_phase(line, blocks)?;

    // Phase 3: Earliest / Latest 推論
    changed.extend(earliest_latest_inference(line, blocks)?);

    // Phase 4: DP line solver（未解決セルが残る場合のみ実行）
    if line.count_cells(Cell::Unknown) > 0 {
        changed.extend(dp_solve(line, blocks)?);
    }

    // Phase 5: DP 後の矛盾チェック
    check_no_dead_cells(line)?;

    Ok(changed)
}
