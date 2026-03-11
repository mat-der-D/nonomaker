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

fn to_result(value: bool) -> Result<(), Contradiction> {
    if value { Ok(()) } else { Err(Contradiction) }
}

pub(crate) fn solve_line(
    line: &mut LineBits,
    blocks: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    // 空ブロック: 全セルを Blank に確定
    if blocks.is_empty() {
        to_result(line.count_cells(Cell::Filled) == 0)?;
        let unknowns: Vec<usize> = (0..line.len())
            .filter(|&i| line.cell(i) == Cell::Unknown)
            .collect();
        line.set_cells(&unknowns, Cell::Blank);
        return Ok(unknowns);
    }

    // Phase 1: 軽量矛盾チェック
    to_result(check_min_space(line, blocks))?;
    to_result(check_filled_count(line, blocks))?;
    to_result(check_consecutive_overflow(line, blocks))?;

    // Phase 2: Segment 分割とブロック割り当て
    let mut changed = segment_phase(line, blocks).ok_or(Contradiction)?;

    // Phase 3: Earliest / Latest 推論
    changed.extend(earliest_latest_inference(line, blocks).ok_or(Contradiction)?);

    // Phase 4: DP line solver（未解決セルが残る場合のみ実行）
    if line.count_cells(Cell::Unknown) > 0 {
        changed.extend(dp_solve(line, blocks).ok_or(Contradiction)?);
    }

    // Phase 5: DP 後の矛盾チェック
    to_result(check_no_dead_cells(line))?;

    Ok(changed)
}
