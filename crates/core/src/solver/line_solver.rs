mod bits;
mod contradiction;
mod dp;
mod segment;

use bits::LineBits;
use contradiction::{check_consecutive_overflow, check_filled_count, check_min_space};
use segment::split_at_blanks;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Contradiction;

fn to_result(value: bool) -> Result<(), Contradiction> {
    if value { Ok(()) } else { Err(Contradiction) }
}

pub(crate) fn solve_line(
    line: &mut LineBits,
    blocks: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    // Phase 1: 軽量矛盾チェック
    to_result(check_min_space(line, blocks))?;
    to_result(check_filled_count(line, blocks))?;
    to_result(check_consecutive_overflow(line, blocks))?;

    // Phase 2: Segment 分割とブロック割り当て
    let segments = split_at_blanks(line);
    // assign_blocks_to_segments(segments, blocks)?

    // Phase 3: Earliest / Latest 推論
    // let changed = earliest_latest_inference(line, blocks);

    // Phase 4: DP line solver (未解決セルが残る場合のみ実行)
    // if has_unknowns(line) {
    //     changed |= dp_solve(line, blocks)?
    // }

    // Phase 5: DP 後の矛盾チェック
    // check_no_dead_cells(line)?

    todo!()
}
