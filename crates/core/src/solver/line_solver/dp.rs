use super::bits::LineBits;

pub(crate) fn dp_solve(line: &LineBits, blocks: &[usize]) -> Option<Vec<usize>> {
    let n = line.len();
    let k = blocks.len();

    let fwd = dp_solve_forward();
    if !fwd.value(n, k) {
        return None;
    }
    let bwd = dp_solve_backward();

    let mut changed = Vec::new();
    for (i, cell) in line.cells().enumerate() {
        //
    }

    todo!()
}

fn dp_solve_forward() -> DPArray {
    todo!()
}

fn dp_solve_backward() -> DPArray {
    todo!()
}

fn can_be_filled(i: usize, fwd: &DPArray, bwd: &DPArray, blocks: &[usize]) -> bool {
    for (j, &block) in blocks.iter().enumerate() {
        let s_min = (i + 1).saturating_sub(block);
        let s_max = i;

        for s in s_min..=s_max {
            // todo!(); // ここで範囲内チェック

            if !can_place_forward(s, j, block) {
                continue;
            }

            let next = if s + block == n { n } else { s + block + 1 };
            //
        }
    }
    false
}

fn can_be_blank(i: usize, fwd: &DPArray, bwd: &DPArray, blocks: &[usize]) -> bool {
    (0..blocks.len()).any(|j| fwd.value(i, j) && bwd.value(i + 1, j))
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
            panic!() // TODO: 適切なメッセージを設定する
        }
        if i2 >= self.size2 {
            panic!() // TODO: 適切なメッセージを設定する
        }
        i1 * self.size1 + i2
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
