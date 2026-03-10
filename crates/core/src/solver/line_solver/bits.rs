use crate::types::Cell;

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
    pub(crate) fn new(len: usize) -> Self {
        let mask = create_repunit(len);
        Self {
            can_be_filled: mask,
            can_be_blank: mask,
            len,
            mask,
        }
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
