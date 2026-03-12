use crate::types::Cell;

use super::super::Contradiction;
use super::bits::LineBits;

pub(crate) fn earliest_latest_inference(
    line: &mut LineBits,
    blocks: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    let earliest = compute_earliest(line, blocks)?;
    let latest = compute_latest(line, blocks)?;

    let mut changed = overlap_inference(line, &earliest, &latest, blocks);
    changed.extend(white_inference(line, &earliest, &latest, blocks));

    Ok(changed)
}

fn compute_earliest(line: &LineBits, blocks: &[usize]) -> Result<Vec<usize>, Contradiction> {
    let n = line.len();
    let mut earliest_start = Vec::with_capacity(blocks.len());
    let mut pos = 0;

    for &len in blocks {
        loop {
            if pos + len > n {
                return Err(Contradiction);
            }
            if line.can_place_block(pos, len) {
                break;
            }
            pos += 1;
        }
        earliest_start.push(pos);
        pos += len + 1;
    }

    Ok(earliest_start)
}

fn compute_latest(line: &LineBits, blocks: &[usize]) -> Result<Vec<usize>, Contradiction> {
    let k = blocks.len();
    let mut latest_start = vec![0; k];
    let mut pos = line.len();

    for j in (0..k).rev() {
        let len = blocks[j];
        loop {
            if pos < len {
                return Err(Contradiction);
            }
            let start = pos - len;
            if line.can_place_block(start, len) {
                latest_start[j] = start;
                break;
            }
            pos -= 1;
        }
        if j > 0 {
            let Some(p) = latest_start[j].checked_sub(1) else {
                return Err(Contradiction);
            };
            pos = p;
        }
    }

    Ok(latest_start)
}

/// 最左配置と最右配置の重なり部分を Filled に確定する
fn overlap_inference(
    line: &mut LineBits,
    earliest: &[usize],
    latest: &[usize],
    blocks: &[usize],
) -> Vec<usize> {
    let mut changed = Vec::new();

    for j in 0..blocks.len() {
        let overlap_start = latest[j];
        let overlap_end = earliest[j] + blocks[j];

        for i in overlap_start..overlap_end {
            if line.cell(i) == Cell::Unknown {
                changed.push(i);
            }
        }
    }

    line.set_cells(&changed, Cell::Filled);
    changed
}

/// どのブロック配置範囲にも含まれないセルを Blank に確定する
fn white_inference(
    line: &mut LineBits,
    earliest: &[usize],
    latest: &[usize],
    blocks: &[usize],
) -> Vec<usize> {
    let n = line.len();
    let k = blocks.len();
    let mut changed = Vec::new();

    // 最初のブロックより左
    for i in 0..earliest[0] {
        if line.cell(i) == Cell::Unknown {
            changed.push(i);
        }
    }

    // 隣接ブロック間の隙間
    for j in 0..k - 1 {
        let gap_start = latest[j] + blocks[j];
        let gap_end = earliest[j + 1];

        for i in gap_start..gap_end {
            if line.cell(i) == Cell::Unknown {
                changed.push(i);
            }
        }
    }

    // 最後のブロックより右
    for i in (latest[k - 1] + blocks[k - 1])..n {
        if line.cell(i) == Cell::Unknown {
            changed.push(i);
        }
    }

    line.set_cells(&changed, Cell::Blank);
    changed
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

    // --- compute_earliest ---

    #[test]
    fn earliest_all_unknown() {
        let line = make_line("?????");
        // ブロック [2, 2]: 最左配置は pos=0, pos=3
        let earliest = compute_earliest(&line, &[2, 2]).unwrap();
        assert_eq!(earliest, vec![0, 3]);
    }

    #[test]
    fn earliest_with_leading_blank() {
        let line = make_line("B????");
        // ブロック [2]: pos=0 は Blank なので最左は 1
        let earliest = compute_earliest(&line, &[2]).unwrap();
        assert_eq!(earliest, vec![1]);
    }

    #[test]
    fn earliest_contradiction_filled_blocks_path() {
        // Filled セルを跨げないので矛盾
        let line = make_line("FBF");
        // ブロック [2]: pos=0 → Blank at 1 → can_place fails; cell(0)=Filled → Err
        assert!(compute_earliest(&line, &[2]).is_err());
    }

    // --- compute_latest ---

    #[test]
    fn latest_all_unknown() {
        // "??????" (長さ6) ブロック [2, 2]:
        // 有効な配置: (0,3), (0,4), (1,4)
        // → ブロック0の最右開始位置=1, ブロック1の最右開始位置=4
        let line = make_line("??????");
        let latest = compute_latest(&line, &[2, 2]).unwrap();
        assert_eq!(latest, vec![1, 4]);
    }

    #[test]
    fn latest_with_trailing_blank() {
        let line = make_line("????B");
        // ブロック [2]: pos=3 → end=5=n+1... pos=2が最右 (end=4, cell(4)=Blank)
        // actually: pos=3 → start=3, end=5=len+1 > n=5? len=5, start=3, start+len=3+2=5=n → can_place_block(3,2)?
        // cell(3)=Unk, cell(4)=Blank → Blank in range → false. pos=2: cells[2,3]=Unk,Unk; cell(4)=Blank → no right issue; cell(1)=Unk → ok. YES.
        let latest = compute_latest(&line, &[2]).unwrap();
        assert_eq!(latest, vec![2]);
    }

    // --- overlap_inference ---
    // オーバーラップ推論: earliest[j]..earliest[j]+len と latest[j]..latest[j]+len の共通部分を Filled にする

    #[test]
    fn overlap_single_block_has_overlap() {
        // ブロック [4], 長さ 5: earliest=[0], latest=[1]
        // オーバーラップ: [latest[0], earliest[0]+len) = [1, 4) → セル 1,2,3 が Filled
        let mut line = make_line("?????");
        let earliest = vec![0];
        let latest = vec![1];
        let changed = overlap_inference(&mut line, &earliest, &latest, &[4]);
        assert_eq!(line.cell(1), Cell::Filled);
        assert_eq!(line.cell(2), Cell::Filled);
        assert_eq!(line.cell(3), Cell::Filled);
        assert!(changed.contains(&1) && changed.contains(&2) && changed.contains(&3));
    }

    #[test]
    fn overlap_single_block_no_overlap_when_slack_equals_block_size() {
        // ブロック [2], 長さ 4: earliest=[0], latest=[2]
        // オーバーラップ: [2, 2) → 空 → Filled なし
        let mut line = make_line("????");
        let earliest = vec![0];
        let latest = vec![2];
        let changed = overlap_inference(&mut line, &earliest, &latest, &[2]);
        assert!(changed.is_empty(), "オーバーラップがないので変更なし");
        assert_eq!(line_to_str(&line), "????");
    }

    #[test]
    fn overlap_two_blocks() {
        // ブロック [2, 2], 長さ 5: earliest=[0,3], latest=[0,3]
        // (唯一の配置) → どちらも [3,2) と [3,5): セル0,1 と セル3,4 が全て Filled
        let mut line = make_line("?????");
        let earliest = vec![0, 3];
        let latest = vec![0, 3];
        overlap_inference(&mut line, &earliest, &latest, &[2, 2]);
        assert_eq!(line.cell(0), Cell::Filled);
        assert_eq!(line.cell(1), Cell::Filled);
        assert_eq!(line.cell(3), Cell::Filled);
        assert_eq!(line.cell(4), Cell::Filled);
        // セル 2 は未確定のまま (空白セル)
        assert_eq!(line.cell(2), Cell::Unknown);
    }

    // --- white_inference ---
    //
    // 「どの有効な配置でも Blank になるセル」のみ Blank に確定する。
    // これは次の条件が全て成立するセル:
    //   セル p が ブロック j に含まれる ⟺ latest[j] ≤ p < earliest[j] + blocks[j]
    // この範囲に一度も入らないセルが確定 Blank となる。
    //
    // ブロック j がセル p を覆いうる範囲: [earliest[j], latest[j] + blocks[j])
    // よって「覆われない」のは:
    //   ブロック間の隙間: [latest[j] + blocks[j], earliest[j+1])
    //   先頭: [0, earliest[0])
    //   末尾: [latest[k-1] + blocks[k-1], n)

    #[test]
    fn white_inference_blanks_before_first_block() {
        // earliest[0]=2 → セル 0,1 は確定 Blank
        let mut line = make_line("?????");
        let earliest = vec![2];
        let latest = vec![3];
        white_inference(&mut line, &earliest, &latest, &[2]);
        assert_eq!(line.cell(0), Cell::Blank);
        assert_eq!(line.cell(1), Cell::Blank);
    }

    #[test]
    fn white_inference_blanks_after_last_block() {
        // latest[0]=1, blocks[0]=2 → ブロック最右端=3 → セル 3,4 は確定 Blank
        let mut line = make_line("?????");
        let earliest = vec![0];
        let latest = vec![1];
        white_inference(&mut line, &earliest, &latest, &[2]);
        assert_eq!(line.cell(3), Cell::Blank);
        assert_eq!(line.cell(4), Cell::Blank);
    }

    /// ★ white_inference のギャップ計算バグを検出するテスト ★
    ///
    /// ブロック j とブロック j+1 の間で確定 Blank になるセルは:
    ///   [latest[j] + blocks[j], earliest[j+1])
    ///
    /// 現在の実装は誤って:
    ///   [earliest[j] + blocks[j], latest[j+1])
    /// を使っており、これは広すぎる範囲を Blank にする。
    ///
    /// 具体例: ブロック [2, 2], 長さ 6
    ///   earliest=[0, 3], latest=[1, 4]
    ///
    /// 正しいギャップ: latest[0]+blocks[0]=3 to earliest[1]=3 → [3,3) = 空 → Blank なし
    /// 誤ったギャップ: earliest[0]+blocks[0]=2 to latest[1]=4 → [2,4) → セル2,3 が誤って Blank
    ///
    /// セル2は配置 (1,4) でブロック0に含まれるため Blank にしてはいけない。
    #[test]
    fn white_inference_gap_between_blocks_must_use_latest_not_earliest() {
        let mut line = make_line("??????");
        // earliest=[0,3], latest=[1,4] → ギャップ [latest[0]+2, earliest[1]) = [3, 3) = 空
        let earliest = vec![0, 3];
        let latest = vec![1, 4];
        white_inference(&mut line, &earliest, &latest, &[2, 2]);
        // セル 2 はブロック0が位置1に置かれると Filled になるので Blank にしてはいけない
        assert_ne!(
            line.cell(2),
            Cell::Blank,
            "セル2は配置 (block0@1, block1@4) でFilledになりうるため Blank 確定は誤り"
        );
        // セル 3 はブロック1が位置3に置かれると Filled になるので Blank にしてはいけない
        assert_ne!(
            line.cell(3),
            Cell::Blank,
            "セル3は配置 (block0@0, block1@3) でFilledになりうるため Blank 確定は誤り"
        );
    }

    /// white_inference のギャップが正しく空になるケース
    ///
    /// ブロック [2, 2], 長さ 5: earliest=[0,3], latest=[0,3]
    ///   ギャップ: latest[0]+2=2 to earliest[1]=3 → [2,3) → セル2 が Blank
    #[test]
    fn white_inference_gap_is_blank_when_no_slack() {
        let mut line = make_line("?????");
        let earliest = vec![0, 3];
        let latest = vec![0, 3];
        white_inference(&mut line, &earliest, &latest, &[2, 2]);
        assert_eq!(
            line.cell(2),
            Cell::Blank,
            "配置が唯一なのでセル2は確定 Blank のはず"
        );
    }

    // --- earliest_latest_inference (統合) ---

    #[test]
    fn inference_single_block_no_slack() {
        // ブロック [5], 長さ 5 → 唯一の配置 → 全セル Filled
        let mut line = make_line("?????");
        earliest_latest_inference(&mut line, &[5]).unwrap();
        assert_eq!(line_to_str(&line), "FFFFF");
    }

    #[test]
    fn inference_single_block_with_slack() {
        // ブロック [4], 長さ 5: earliest=[0], latest=[1]
        // オーバーラップ [1,4) → セル1,2,3=Filled
        // ホワイト: before=0..0=empty, after=5..5=empty
        // セル0,4 は不確定
        let mut line = make_line("?????");
        earliest_latest_inference(&mut line, &[4]).unwrap();
        assert_eq!(line.cell(1), Cell::Filled);
        assert_eq!(line.cell(2), Cell::Filled);
        assert_eq!(line.cell(3), Cell::Filled);
        assert_eq!(line.cell(0), Cell::Unknown);
        assert_eq!(line.cell(4), Cell::Unknown);
    }
}
