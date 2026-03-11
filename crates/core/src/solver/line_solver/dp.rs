use crate::types::Cell;

use super::super::Contradiction;
use super::bits::LineBits;

pub(crate) fn dp_solve(line: &mut LineBits, blocks: &[usize]) -> Result<Vec<usize>, Contradiction> {
    let (filled, blank) = DPSolver::solve(line, blocks)?;
    line.set_cells(&filled, Cell::Filled);
    line.set_cells(&blank, Cell::Blank);
    let mut changed = filled;
    changed.extend(blank);
    Ok(changed)
}

#[derive(Debug)]
struct DPSolver<'a> {
    line: &'a LineBits,
    blocks: &'a [usize],
    fwd: DPArray,
    bwd: DPArray,
}

impl<'a> DPSolver<'a> {
    fn n(&self) -> usize {
        self.line.len()
    }

    fn k(&self) -> usize {
        self.blocks.len()
    }

    fn solve(
        line: &'a LineBits,
        blocks: &'a [usize],
    ) -> Result<(Vec<usize>, Vec<usize>), Contradiction> {
        let mut solver = Self {
            line,
            blocks,
            fwd: DPArray::new(line.len() + 1, blocks.len() + 1),
            bwd: DPArray::new(line.len() + 1, blocks.len() + 1),
        };
        solver.build_forward();
        if !solver.fwd.value(solver.n(), solver.k()) {
            return Err(Contradiction);
        }
        solver.build_backward();
        solver.collect_changes()
    }

    fn build_forward(&mut self) {
        self.fwd.set_value(0, 0, true);
        for i in 0..self.n() {
            for j in 0..=self.k() {
                self.forward_transition(i, j);
            }
        }
    }

    fn forward_transition(&mut self, i: usize, j: usize) {
        if !self.fwd.value(i, j) {
            return;
        }
        // Blank 遷移
        if self.line.cell(i) != Cell::Filled {
            self.fwd.set_value(i + 1, j, true);
        }
        // ブロック配置遷移
        if let Some(&len) = self.blocks.get(j)
            && self.line.can_place_block(i, len)
        {
            let next = self.next_after_block(i, len);
            self.fwd.set_value(next, j + 1, true);
        }
    }

    fn build_backward(&mut self) {
        self.bwd.set_value(self.n(), self.k(), true);
        for i in (0..self.n()).rev() {
            for j in 0..=self.k() {
                self.backward_transition(i, j);
            }
        }
    }

    fn backward_transition(&mut self, i: usize, j: usize) {
        // Blank 遷移
        if self.bwd.value(i + 1, j) && self.line.cell(i) != Cell::Filled {
            self.bwd.set_value(i, j, true);
        }
        // ブロック配置遷移
        if let Some(&len) = self.blocks.get(j)
            && self.line.can_place_block(i, len)
        {
            let next = self.next_after_block(i, len);
            if self.bwd.value(next, j + 1) {
                self.bwd.set_value(i, j, true);
            }
        }
    }

    /// Filled 確定と Blank 確定のインデックスを返す。矛盾なら Err。
    fn collect_changes(&self) -> Result<(Vec<usize>, Vec<usize>), Contradiction> {
        let mut filled = Vec::new();
        let mut blank = Vec::new();
        for i in 0..self.n() {
            if self.line.cell(i) != Cell::Unknown {
                continue;
            }
            match self.resolve_cell(i)? {
                Cell::Filled => filled.push(i),
                Cell::Blank => blank.push(i),
                Cell::Unknown => {}
            }
        }
        Ok((filled, blank))
    }

    /// DP の結果からセル i の状態を判定する。矛盾なら Err。
    fn resolve_cell(&self, i: usize) -> Result<Cell, Contradiction> {
        match (self.can_be_filled(i), self.can_be_blank(i)) {
            (true, true) => Ok(Cell::Unknown),
            (true, false) => Ok(Cell::Filled),
            (false, true) => Ok(Cell::Blank),
            (false, false) => Err(Contradiction),
        }
    }

    // --- Helper ---

    /// ブロック配置後の次の遷移先インデックス
    fn next_after_block(&self, s: usize, len: usize) -> usize {
        let end = s + len;
        if end == self.n() { end } else { end + 1 }
    }

    /// セル i が Filled になりうるか（いずれかのブロック配置経路が存在する）
    fn can_be_filled(&self, i: usize) -> bool {
        for (j, &len) in self.blocks.iter().enumerate() {
            let s_min = (i + 1).saturating_sub(len);
            let s_max = i;

            for s in s_min..=s_max {
                if !self.line.can_place_block(s, self.blocks[j]) {
                    continue;
                }
                let next = self.next_after_block(s, len);
                if self.fwd.value(s, j) && self.bwd.value(next, j + 1) {
                    return true;
                }
            }
        }
        false
    }

    /// セル i が Blank になりうるか（Blank として通過する経路が存在する）
    fn can_be_blank(&self, i: usize) -> bool {
        (0..=self.k()).any(|j| self.fwd.value(i, j) && self.bwd.value(i + 1, j))
    }
}

#[derive(Debug, Clone)]
struct DPArray {
    values: Vec<bool>,
    size1: usize,
    size2: usize,
}

impl DPArray {
    fn new(size1: usize, size2: usize) -> Self {
        Self {
            values: vec![false; size1 * size2],
            size1,
            size2,
        }
    }

    fn raw_index(&self, i1: usize, i2: usize) -> usize {
        if i1 >= self.size1 {
            panic!(
                "DPArray index out of bounds: i1={i1} >= size1={}",
                self.size1
            );
        }
        if i2 >= self.size2 {
            panic!(
                "DPArray index out of bounds: i2={i2} >= size2={}",
                self.size2
            );
        }
        i1 * self.size2 + i2
    }

    fn value(&self, i1: usize, i2: usize) -> bool {
        let index = self.raw_index(i1, i2);
        self.values[index]
    }

    fn set_value(&mut self, i1: usize, i2: usize, value: bool) {
        let index = self.raw_index(i1, i2);
        self.values[index] = value;
    }
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

    // --- dp_solve の基本動作 ---

    #[test]
    fn dp_full_determination_unique_placement() {
        // ブロック [3], 長さ 3 → 唯一の配置 → 全て Filled
        let mut line = make_line("???");
        dp_solve(&mut line, &[3]).unwrap();
        assert_eq!(line_to_str(&line), "FFF");
    }

    #[test]
    fn dp_contradiction_returns_err() {
        // ブロック [2], 長さ 3 で cell[1] が Blank → 配置不能
        let mut line = make_line("?B?");
        assert!(dp_solve(&mut line, &[2]).is_err());
    }

    #[test]
    fn dp_two_blocks_exact_fit() {
        // ブロック [2, 2], 長さ 5 → FFBFF
        let mut line = make_line("?????");
        dp_solve(&mut line, &[2, 2]).unwrap();
        assert_eq!(line_to_str(&line), "FFBFF");
    }

    #[test]
    fn dp_preserves_already_filled_cells() {
        // 既に Filled なセルは変更されない
        let mut line = make_line("FFF");
        dp_solve(&mut line, &[3]).unwrap();
        assert_eq!(line_to_str(&line), "FFF");
    }

    // --- ギャップセルの Blank 判定 ---
    //
    // DP モデルでは、ブロックを位置 s に配置すると位置 s+len (< n の場合) は
    // 強制的に Blank になる（mandatory gap）。
    // この gap セルを「Blank になりうる」と正しく判定できるかを確認する。
    //
    // 例: ライン "?FFF?" ブロック [4]
    //   - 配置 A: pos=0 → cells[0,3]=F, cell[4]=gap(Blank)
    //   - 配置 B: pos=1 → cells[1,4]=F, cell[0]=Blank
    //   → セル 0 と セル 4 はどちらも Unknown であるべき

    #[test]
    fn dp_gap_cell_after_block_must_be_unknown_not_forced_filled() {
        // "?FFF?" にブロック [4]: pos=0 (セル4=gap=Blank) or pos=1 (セル4=Filled)
        // → セル4は Unknown であるべき
        let mut line = make_line("?FFF?");
        dp_solve(&mut line, &[4]).unwrap();
        assert_eq!(
            line.cell(4),
            Cell::Unknown,
            "セル4は配置 pos=0 で Blank になりうるので Unknown でなければならない。\
             Filled と判定されるなら gap セルの Blank チェックが欠落している。"
        );
        // 対称的にセル0も Unknown
        assert_eq!(
            line.cell(0),
            Cell::Unknown,
            "セル0は配置 pos=1 で Blank になりうるので Unknown でなければならない"
        );
    }

    #[test]
    fn dp_gap_cell_two_blocks() {
        // ライン "??????" ブロック [2, 2]: 有効配置 (0,3),(0,4),(1,4)
        // セル2は配置 (1,4) で Filled → Unknown
        // セル3は配置 (0,3) で Filled → Unknown
        // (white_inference なし相当の状態でテスト)
        let mut line = make_line("??????");
        dp_solve(&mut line, &[2, 2]).unwrap();
        assert_eq!(
            line.cell(2),
            Cell::Unknown,
            "セル2は配置によって Filled or Blank になるので Unknown"
        );
        assert_eq!(
            line.cell(3),
            Cell::Unknown,
            "セル3は配置によって Filled or Blank になるので Unknown"
        );
    }

    #[test]
    fn dp_gap_cell_tight_single_block() {
        // "????" ブロック [3]: 有効配置 pos=0 (gap@3) または pos=1 (cell0=Blank)
        // → セル0, セル3 が Unknown
        let mut line = make_line("????");
        dp_solve(&mut line, &[3]).unwrap();
        assert_eq!(
            line.cell(0),
            Cell::Unknown,
            "セル0: pos=1 で Blank になりうる"
        );
        assert_eq!(
            line.cell(3),
            Cell::Unknown,
            "セル3: pos=0 の gap で Blank になりうる"
        );
    }
}
