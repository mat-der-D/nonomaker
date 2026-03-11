use crate::types::{Cell, Grid};

const fn create_repunit(count: usize) -> u64 {
    let mut repunit = 0;
    let mut i = 0;
    while i < count {
        repunit = (repunit << 1) | 1;
        i += 1;
    }
    repunit
}

#[derive(Debug, Clone)]
pub(crate) struct LineBits {
    can_be_filled: u64,
    can_be_blank: u64,
    len: usize,
    mask: u64, // 有効なビット部分に制限するマスク
}

impl LineBits {
    pub(crate) fn from_grid_row(grid: &Grid, row: usize) -> Self {
        let len = grid.width();
        let mask = create_repunit(len);
        let (can_be_filled, can_be_blank) =
            Self::build_bits((0..len).map(|col| *grid.cell(row, col)));
        Self {
            can_be_filled,
            can_be_blank,
            len,
            mask,
        }
    }

    pub(crate) fn from_grid_col(grid: &Grid, col: usize) -> Self {
        let len = grid.height();
        let mask = create_repunit(len);
        let (can_be_filled, can_be_blank) =
            Self::build_bits((0..len).map(|row| *grid.cell(row, col)));
        Self {
            can_be_filled,
            can_be_blank,
            len,
            mask,
        }
    }

    fn build_bits(cells: impl Iterator<Item = Cell>) -> (u64, u64) {
        let mut can_be_filled = 0;
        let mut can_be_blank = 0;
        for (i, cell) in cells.enumerate() {
            let bit = 1 << i;
            if matches!(cell, Cell::Filled | Cell::Unknown) {
                can_be_filled |= bit;
            }
            if matches!(cell, Cell::Blank | Cell::Unknown) {
                can_be_blank |= bit;
            }
        }
        (can_be_filled, can_be_blank)
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn count_cells(&self, cell: Cell) -> usize {
        let bits = match cell {
            Cell::Unknown => self.can_be_filled & self.can_be_blank,
            Cell::Filled => self.can_be_filled & !self.can_be_blank,
            Cell::Blank => !self.can_be_filled & self.can_be_blank,
        };
        bits.count_ones() as usize
    }

    pub(crate) fn cells(&self) -> Cells {
        Cells {
            can_be_filled: self.can_be_filled,
            can_be_blank: self.can_be_blank,
            pos: 0,
            len: self.len,
        }
    }

    pub(crate) fn set_cells(&mut self, indices: &[usize], cell: Cell) {
        let mut mask = 0;
        for idx in indices {
            mask |= 1 << idx;
        }
        // Filled
        match cell {
            Cell::Filled | Cell::Unknown => self.can_be_filled |= mask,
            Cell::Blank => self.can_be_filled &= !mask,
        };

        // Blank
        match cell {
            Cell::Blank | Cell::Unknown => self.can_be_blank |= mask,
            Cell::Filled => self.can_be_blank &= !mask,
        }
    }

    pub(crate) fn cell(&self, i: usize) -> Cell {
        cell_from_lowest_bits(i, self.can_be_filled >> i, self.can_be_blank >> i)
    }

    /// いずれかのセルが矛盾状態 (can_be_filled=0, can_be_blank=0) かを判定する
    pub(crate) fn has_contradiction(&self) -> bool {
        let valid = self.can_be_filled | self.can_be_blank;
        (valid & self.mask) != self.mask
    }

    /// すべてのセルが確定状態かを判定する
    pub(crate) fn is_solved(&self) -> bool {
        let solved = self.can_be_filled ^ self.can_be_blank;
        solved == self.mask
    }

    /// ブロック(長さ len)を位置 pos に配置できるか
    pub(crate) fn can_place_block(&self, pos: usize, len: usize) -> bool {
        if pos + len > self.len {
            return false;
        }
        if (pos..pos + len).any(|i| self.cell(i) == Cell::Blank) {
            return false;
        }
        if pos + len < self.len && self.cell(pos + len) == Cell::Filled {
            return false;
        }
        if pos > 0 && self.cell(pos - 1) == Cell::Filled {
            return false;
        }
        true
    }
}

#[derive(Debug)]
pub(crate) struct Cells {
    can_be_filled: u64,
    can_be_blank: u64,
    pos: usize,
    len: usize,
}

impl Iterator for Cells {
    type Item = Cell;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        let cell = cell_from_lowest_bits(self.pos, self.can_be_filled, self.can_be_blank);
        self.can_be_filled >>= 1;
        self.can_be_blank >>= 1;
        self.pos += 1;
        Some(cell)
    }
}

/// 最下位ビットからセル状態を判定する
fn cell_from_lowest_bits(pos: usize, filled: u64, blank: u64) -> Cell {
    match (filled & 1, blank & 1) {
        (1, 1) => Cell::Unknown,
        (1, 0) => Cell::Filled,
        (0, 1) => Cell::Blank,
        (0, 0) => panic!("contradiction at cell {pos}"),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Grid;

    /// パターン文字列 ('F'=Filled, 'B'=Blank, '?'=Unknown) から LineBits を生成する
    fn make_line(pattern: &str) -> LineBits {
        let n = pattern.len();
        let mut grid = Grid::new(n, 1);
        for (i, c) in pattern.chars().enumerate() {
            *grid.cell_mut(0, i) = match c {
                'F' => Cell::Filled,
                'B' => Cell::Blank,
                '?' => Cell::Unknown,
                other => panic!("不正な文字: {other}"),
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

    // --- cell / count_cells ---

    #[test]
    fn cell_reads_correct_state() {
        let line = make_line("F?B");
        assert_eq!(line.cell(0), Cell::Filled);
        assert_eq!(line.cell(1), Cell::Unknown);
        assert_eq!(line.cell(2), Cell::Blank);
    }

    #[test]
    fn count_cells_all_unknown() {
        let line = make_line("???");
        assert_eq!(line.count_cells(Cell::Unknown), 3);
        assert_eq!(line.count_cells(Cell::Filled), 0);
        assert_eq!(line.count_cells(Cell::Blank), 0);
    }

    #[test]
    fn count_cells_mixed() {
        let line = make_line("F?B");
        assert_eq!(line.count_cells(Cell::Unknown), 1);
        assert_eq!(line.count_cells(Cell::Filled), 1);
        assert_eq!(line.count_cells(Cell::Blank), 1);
    }

    // --- cells() iterator ---

    #[test]
    fn cells_iterator_order() {
        assert_eq!(line_to_str(&make_line("FB?")), "FB?");
        assert_eq!(line_to_str(&make_line("?BF")), "?BF");
    }

    // --- is_solved / has_contradiction ---

    #[test]
    fn is_solved_when_all_determined() {
        assert!(make_line("FFB").is_solved());
        assert!(make_line("F").is_solved());
        assert!(make_line("B").is_solved());
    }

    #[test]
    fn is_not_solved_when_unknown_remains() {
        assert!(!make_line("F?B").is_solved());
        assert!(!make_line("???").is_solved());
    }

    #[test]
    fn no_contradiction_in_normal_states() {
        assert!(!make_line("???").has_contradiction());
        assert!(!make_line("FFB").has_contradiction());
    }

    // --- set_cells ---

    #[test]
    fn set_cells_unknown_to_filled() {
        let mut line = make_line("???");
        line.set_cells(&[0, 2], Cell::Filled);
        assert_eq!(line.cell(0), Cell::Filled);
        assert_eq!(line.cell(1), Cell::Unknown);
        assert_eq!(line.cell(2), Cell::Filled);
    }

    #[test]
    fn set_cells_unknown_to_blank() {
        let mut line = make_line("???");
        line.set_cells(&[1], Cell::Blank);
        assert_eq!(line.cell(1), Cell::Blank);
    }

    // --- can_place_block ---

    #[test]
    fn can_place_block_fits_exactly() {
        let line = make_line("???");
        assert!(line.can_place_block(0, 3));
    }

    #[test]
    fn can_place_block_out_of_bounds() {
        let line = make_line("???");
        assert!(!line.can_place_block(2, 2)); // 2+2=4 > 3
    }

    #[test]
    fn can_place_block_blocked_by_blank_in_range() {
        let line = make_line("?B?");
        assert!(!line.can_place_block(0, 2)); // cell(1)=Blank
        assert!(!line.can_place_block(0, 3)); // cell(1)=Blank
        assert!(line.can_place_block(2, 1)); // cell(2)=Unknown, no right neighbor
    }

    #[test]
    fn can_place_block_blocked_by_filled_on_right() {
        // block=[0,1] の直後に Filled があると拡張してしまうので置けない
        let line = make_line("??F");
        assert!(!line.can_place_block(0, 2)); // cell(2)=Filled → 不可
        assert!(line.can_place_block(0, 3)); // cell(3) が存在しない → 可
    }

    #[test]
    fn can_place_block_blocked_by_filled_on_left() {
        // block=[1,2] の直前に Filled があると前のブロックと連結するので置けない
        let line = make_line("F??");
        assert!(!line.can_place_block(1, 2)); // cell(0)=Filled → 不可
        assert!(line.can_place_block(0, 2)); // cell(0-1)が存在しない → 可
    }

    #[test]
    fn can_place_block_at_end_no_right_check() {
        // ライン末尾に置く場合は右側のチェックがない
        let line = make_line("???");
        assert!(line.can_place_block(1, 2)); // [1,2], 右端=3=n → right check skipped
    }

    #[test]
    fn can_place_block_size_one() {
        let line = make_line("?B?");
        assert!(line.can_place_block(0, 1));
        assert!(!line.can_place_block(1, 1)); // cell(1)=Blank
        assert!(line.can_place_block(2, 1));
    }
}
