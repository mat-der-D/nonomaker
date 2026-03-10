use crate::types::Cell;

use super::bits::LineBits;

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

pub(crate) fn split_at_blanks(line: &LineBits) -> Vec<Segment> {
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
        seg_start = i + 1; // 確定空白の次のセル
    }

    // 末尾まで残っているセグメントを処理
    if seg_start < line.len() {
        segments.push(Segment {
            start: seg_start,
            end: line.len(),
        })
    }

    segments
}

fn fits(segment: &Segment, blocks: &[usize]) -> bool {
    let num_blocks = blocks.len();
    let min_space = blocks.iter().sum::<usize>() + (num_blocks - 1);
    min_space <= segment.len()
}

fn compute_earliest_segement(segments: &[Segment], blocks: &[usize]) -> Option<Vec<usize>> {
    let mut earliest_seg_indices = Vec::with_capacity(blocks.len());
    let mut slice_start = 0;
    let mut seg_idx = 0;

    for j in 0..blocks.len() {
        while !fits(&segments[seg_idx], &blocks[slice_start..=j]) {
            seg_idx += 1;
            slice_start = j;

            if seg_idx >= segments.len() {
                return None;
            }
        }
        earliest_seg_indices.push(seg_idx);
    }

    Some(earliest_seg_indices)
}

fn compute_latest_segment(segments: &[Segment], blocks: &[usize]) -> Option<Vec<usize>> {
    let mut latest_seg_indices = Vec::with_capacity(blocks.len());
    let mut slice_end = segments.len() - 1;
    let mut seg_idx = segments.len() - 1;

    for j in (0..blocks.len()).rev() {
        while !fits(&segments[seg_idx], &blocks[j..=slice_end]) {
            if seg_idx == 0 {
                return None;
            }

            seg_idx -= 1;
            slice_end = j;
        }
        latest_seg_indices.push(seg_idx);
    }

    Some(latest_seg_indices)
}

fn confirm_empty_segments(
    line: &mut LineBits,
    segments: &[Segment],
    earliest_segment: &[usize],
    latest_segment: &[usize],
) -> Option<Vec<usize>> {
    let mut changed = Vec::new();

    for (seg_idx, segment) in segments.iter().enumerate() {
        let has_block = earliest_segment
            .iter()
            .zip(latest_segment.iter())
            .any(|(&e_seg, &l_seg)| e_seg <= seg_idx && seg_idx <= l_seg);

        if has_block {
            continue;
        }

        for (i, cell) in line
            .cells()
            .enumerate()
            .skip(segment.start)
            .take(segment.len())
        {
            if cell == Cell::Filled {
                return None;
            }
            changed.push(i);
        }
    }

    line.set_cells(&changed, Cell::Blank);
    Some(changed)
}

fn solve_confirmed_segments(
    segments: &[Segment],
    blocks: &[usize],
    earliest_segment: &[usize],
    latest_segment: &[usize],
) -> Option<Vec<usize>> {
    let mut changed = Vec::new();

    // for seg_idx in 0..segments.len() {
    //     let mut j_start = None;
    //     let mut j_end = None;

    //     for j in 0..blocks.len() {
    //         if earliest_segment[j] != latest_segment[j] {
    //             continue;
    //         }
    //     }
    // }
    //
    //

    Some(changed)
}

pub(crate) fn segment_phase(line: &LineBits, blocks: &[usize]) {
    let segments = split_at_blanks(line);

    let earliest_segment = compute_earliest_segement(&segments, blocks);
    let latest_segment = compute_latest_segment(&segments, blocks);

    // confirm_empty_segments(line, segments, blocks, earliest_segment, latest_segment)?
    // solve_confirmed_segments(segments, blocks, earliest_segment, latest_segment)?

    todo!()
}
