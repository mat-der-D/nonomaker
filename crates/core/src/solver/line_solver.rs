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

    fn line_to_str(line: &LineBits) -> String {
        line.cells()
            .map(|c| match c {
                Cell::Unknown => '?',
                Cell::Filled => 'F',
                Cell::Blank => 'B',
            })
            .collect()
    }

    fn solve(pattern: &str, blocks: &[usize]) -> Result<String, ()> {
        let mut line = make_line(pattern);
        solve_line(&mut line, blocks).map_err(|_| ())?;
        Ok(line_to_str(&line))
    }

    // --- 矛盾検出 ---

    #[test]
    fn contradiction_empty_blocks_with_filled() {
        assert!(solve("F??", &[]).is_err());
    }

    #[test]
    fn contradiction_line_too_short() {
        assert!(solve("??", &[3]).is_err());
    }

    #[test]
    fn contradiction_no_valid_placement() {
        // 'BFB' にブロック [2] は置けない
        assert!(solve("BFB", &[2]).is_err());
    }

    #[test]
    fn contradiction_filled_run_exceeds_max_block() {
        assert!(solve("FFF", &[2]).is_err());
    }

    // --- 空ブロック ---

    #[test]
    fn empty_blocks_all_become_blank() {
        assert_eq!(solve("?????", &[]).unwrap(), "BBBBB");
    }

    // --- 完全確定（配置が一意）---

    #[test]
    fn single_block_fills_entire_line() {
        assert_eq!(solve("?????", &[5]).unwrap(), "FFFFF");
    }

    #[test]
    fn two_blocks_exact_fit() {
        // [2,2] を長さ5に収める唯一の配置: FF_FF
        assert_eq!(solve("?????", &[2, 2]).unwrap(), "FFBFF");
    }

    #[test]
    fn already_determined_consistent() {
        assert_eq!(solve("FFBFF", &[2, 2]).unwrap(), "FFBFF");
    }

    // --- オーバーラップ推論のみで確定するセル ---

    #[test]
    fn overlap_inference_single_block() {
        // ブロック [4], 長さ 5: セル 1,2,3 が確定 Filled
        let result = solve("?????", &[4]).unwrap();
        assert_eq!(&result[1..4], "FFF", "セル1-3はオーバーラップで Filled");
        // セル0, セル4 はどちらの配置でも F か B に変わる → 確定不可
        assert_eq!(&result[0..1], "?", "セル0は確定不可");
        assert_eq!(&result[4..5], "?", "セル4は確定不可");
    }

    /// ★ white_inference のギャップ計算バグの統合テスト ★
    ///
    /// ブロック [2, 2], 長さ 6 の有効な配置:
    ///   (block0@0, block1@3): FF_FF_  → FF B FF B
    ///   (block0@0, block1@4): FF__FF  → FF B B FF
    ///   (block0@1, block1@4): _FF_FF  → B FF B FF
    ///
    /// よって確定できるのはセル1(=F in all)とセル4(=F in all)だけ。
    /// セル2, セル3 は配置によって F にも B にもなる → Unknown のまま。
    ///
    /// バグがあると白推論が [2,4) を誤って Blank にし "?FBBF?" になる。
    #[test]
    fn white_inference_does_not_over_blank_gap_cells() {
        let result = solve("??????", &[2, 2]).unwrap();
        assert_eq!(
            result, "?F??F?",
            "セル1とセル4のみ Filled に確定。セル2,3はどちらにもなりうるので Unknown。\
             バグがあると \"?FBBF?\" などになる。"
        );
    }

    /// ブロック [2, 2], 長さ 7: 多くの配置がある
    /// → オーバーラップも白推論も何も確定できない
    #[test]
    fn no_inference_when_too_many_placements() {
        let result = solve("???????", &[2, 2]).unwrap();
        assert_eq!(result, "???????", "配置が多いのでどのセルも確定不可");
    }

    /// ブロック [3, 3], 長さ 10: earliest=[0,4], latest=[3,7]
    /// 正しいギャップ: latest[0]+3=6 to earliest[1]=4 → 6>4 → 空
    /// バグあり:       earliest[0]+3=3 to latest[1]=7 → セル3-6を誤って Blank
    #[test]
    fn white_inference_gap_blocks_3_3_length_10() {
        let result = solve("??????????", &[3, 3]).unwrap();
        assert_eq!(
            result, "??????????",
            "長さ10, ブロック[3,3]ではどのセルも確定できないはず"
        );
    }

    /// ★ DP の gap セル Blank 判定バグの統合テスト ★
    ///
    /// ブロック [4], 長さ 5: 配置 pos=0 ではセル4が gap=Blank、
    /// 配置 pos=1 ではセル4が Filled → セル4は Unknown であるべき。
    /// DP の can_be_blank がギャップセルを検出できないと Filled に誤判定する。
    #[test]
    fn dp_gap_cell_remains_unknown_in_solve_line() {
        let result = solve("?????", &[4]).unwrap();
        assert_eq!(
            &result[0..1],
            "?",
            "セル0: pos=1 の配置で Blank になりうる → Unknown"
        );
        assert_eq!(
            &result[4..5],
            "?",
            "セル4: pos=0 の配置で gap(Blank) になりうる → Unknown"
        );
    }

    // --- 既知セルを活用する推論 ---

    #[test]
    fn known_filled_narrows_placement() {
        // 'F?????' ブロック [3, 2], min_space=3+2+1=6=len → 唯一配置: FFFBFF
        assert_eq!(solve("F?????", &[3, 2]).unwrap(), "FFFBFF");
    }

    #[test]
    fn known_blank_separates_segments() {
        // '??B??' ブロック [2]: Blank が区切りになり左右どちらのセグメントにも入る
        // → 確定はされないが矛盾もしない
        assert!(solve("??B??", &[2]).is_ok());
    }

    // --- 境界値 ---

    #[test]
    fn single_cell_single_block() {
        assert_eq!(solve("?", &[1]).unwrap(), "F");
    }

    #[test]
    fn single_cell_no_block() {
        assert_eq!(solve("?", &[]).unwrap(), "B");
    }

    #[test]
    fn single_cell_block_too_large() {
        assert!(solve("?", &[2]).is_err());
    }
}
