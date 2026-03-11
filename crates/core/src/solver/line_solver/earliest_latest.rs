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
            if line.cell(pos) == Cell::Filled {
                return Err(Contradiction);
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
            if line.cell(pos) == Cell::Filled {
                return Err(Contradiction);
            }
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
        let gap_start = earliest[j] + blocks[j];
        let gap_end = latest[j + 1];

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
