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
        Self {
            can_be_filled: 0,
            can_be_blank: 0,
            len,
            mask: create_repunit(len),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn count_cells(&self, cell: Cell) -> usize {
        todo!()
    }

    pub(crate) fn cells(&self) -> Cells {
        todo!()
    }

    pub(crate) fn set_cells(&mut self, indices: &[usize], cell: Cell) {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct Cells {}

impl Iterator for Cells {
    type Item = Cell;
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
