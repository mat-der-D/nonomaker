use crate::types::Cell;

use super::bits::LineBits;
use super::super::Contradiction;

#[derive(Debug, Clone)]
pub(crate) struct Segment {
    start: usize, // inclusive
    end: usize,   // exclusive
}

impl Segment {
    fn len(&self) -> usize {
        self.end - self.start
    }
}

pub(crate) fn segment_phase(
    line: &mut LineBits,
    blocks: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    let segments = split_at_blanks(line);

    if segments.is_empty() {
        return if blocks.is_empty() { Ok(Vec::new()) } else { Err(Contradiction) };
    }

    let earliest_segment = compute_earliest_segment(&segments, blocks)?;
    let latest_segment = compute_latest_segment(&segments, blocks)?;

    confirm_empty_segments(line, &segments, &earliest_segment, &latest_segment)
}

fn split_at_blanks(line: &LineBits) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut seg_start = 0;

    for (i, cell) in line.cells().enumerate() {
        if cell != Cell::Blank {
            continue;
        }

        if seg_start < i {
            segments.push(Segment {
                start: seg_start,
                end: i,
            })
        }
        seg_start = i + 1;
    }

    if seg_start < line.len() {
        segments.push(Segment {
            start: seg_start,
            end: line.len(),
        })
    }

    segments
}

fn fits(segment: &Segment, blocks: &[usize]) -> bool {
    if blocks.is_empty() {
        return true;
    }
    let min_space = blocks.iter().sum::<usize>() + (blocks.len() - 1);
    min_space <= segment.len()
}

fn compute_earliest_segment(
    segments: &[Segment],
    blocks: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    let mut earliest = Vec::with_capacity(blocks.len());
    let mut slice_start = 0;
    let mut seg_idx = 0;

    for j in 0..blocks.len() {
        while !fits(&segments[seg_idx], &blocks[slice_start..=j]) {
            seg_idx += 1;
            slice_start = j;

            if seg_idx >= segments.len() {
                return Err(Contradiction);
            }
        }
        earliest.push(seg_idx);
    }

    Ok(earliest)
}

fn compute_latest_segment(
    segments: &[Segment],
    blocks: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    let k = blocks.len();
    let mut latest = vec![0; k];
    let mut slice_end = k - 1;
    let mut seg_idx = segments.len() - 1;

    for j in (0..k).rev() {
        while !fits(&segments[seg_idx], &blocks[j..=slice_end]) {
            if seg_idx == 0 {
                return Err(Contradiction);
            }
            seg_idx -= 1;
            slice_end = j;
        }
        latest[j] = seg_idx;
    }

    Ok(latest)
}

fn confirm_empty_segments(
    line: &mut LineBits,
    segments: &[Segment],
    earliest_segment: &[usize],
    latest_segment: &[usize],
) -> Result<Vec<usize>, Contradiction> {
    let mut changed = Vec::new();

    for (seg_idx, segment) in segments.iter().enumerate() {
        let has_block = earliest_segment
            .iter()
            .zip(latest_segment.iter())
            .any(|(&e, &l)| e <= seg_idx && seg_idx <= l);

        if has_block {
            continue;
        }

        for (i, cell) in line
            .cells()
            .enumerate()
            .skip(segment.start)
            .take(segment.len())
        {
            match cell {
                Cell::Filled => return Err(Contradiction),
                Cell::Unknown => changed.push(i),
                Cell::Blank => {}
            }
        }
    }

    line.set_cells(&changed, Cell::Blank);
    Ok(changed)
}
