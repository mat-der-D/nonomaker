use std::collections::VecDeque;

use crate::types::{Cell, Grid, Puzzle};

use super::propagation::propagate;

#[derive(Debug, Clone, Copy)]
pub(crate) enum FullyProbingMode {
    Fp1,
    Fp2,
}

pub(crate) fn solve(puzzle: &Puzzle, mode: FullyProbingMode) -> Option<Grid> {
    let mut grid = Grid::new(puzzle.width(), puzzle.height());

    loop {
        if !propagate(&mut grid, puzzle) {
            return None;
        }

        let unknown_cells = collect_unknown_cells(&grid);
        if unknown_cells.is_empty() {
            return Some(grid);
        }

        let codec = LiteralCodec::new(&grid);
        let results = match mode {
            FullyProbingMode::Fp1 => run_fp1_round(&grid, puzzle, &unknown_cells, &codec),
            FullyProbingMode::Fp2 => run_fp2_round(&grid, puzzle, &unknown_cells, &codec),
        };
        let inferred = infer_literals(&results, &unknown_cells, &codec)?;
        if inferred.is_empty() {
            return Some(grid);
        }
        apply_literals(&mut grid, &inferred, &codec)?;
    }
}

fn run_fp1_round(
    grid: &Grid,
    puzzle: &Puzzle,
    unknown_cells: &[usize],
    codec: &LiteralCodec,
) -> Vec<ProbeResult> {
    let mut results = vec![ProbeResult::Conflict; codec.literal_count()];
    for &cell in unknown_cells {
        for value in [Cell::Blank, Cell::Filled] {
            let literal = codec.encode(cell, value);
            results[literal] = probe(grid, puzzle, &[literal], codec);
        }
    }
    results
}

fn run_fp2_round(
    grid: &Grid,
    puzzle: &Puzzle,
    unknown_cells: &[usize],
    codec: &LiteralCodec,
) -> Vec<ProbeResult> {
    let mut assumptions = vec![Vec::new(); codec.literal_count()];
    let mut results = vec![ProbeResult::Conflict; codec.literal_count()];
    let mut queue = VecDeque::new();

    for &cell in unknown_cells {
        for value in [Cell::Blank, Cell::Filled] {
            let literal = codec.encode(cell, value);
            assumptions[literal].push(literal);
            queue.push_back(literal);
        }
    }

    while let Some(literal) = queue.pop_front() {
        let result = probe(grid, puzzle, &assumptions[literal], codec);
        results[literal] = result.clone();

        let ProbeResult::Consistent(forced_literals) = result else {
            continue;
        };

        let contrapositive = codec.complement(literal);
        for forced in forced_literals {
            let target = codec.complement(forced);
            if push_unique(&mut assumptions[target], contrapositive) {
                queue.push_back(target);
            }
        }
    }

    results
}

fn infer_literals(
    results: &[ProbeResult],
    unknown_cells: &[usize],
    codec: &LiteralCodec,
) -> Option<Vec<usize>> {
    let mut inferred = vec![false; codec.literal_count()];
    let mut scratch = vec![false; codec.literal_count()];

    for &cell in unknown_cells {
        let blank = codec.encode(cell, Cell::Blank);
        let filled = codec.encode(cell, Cell::Filled);
        let blank_result = &results[blank];
        let filled_result = &results[filled];

        match (blank_result, filled_result) {
            (ProbeResult::Conflict, ProbeResult::Conflict) => return None,
            (ProbeResult::Conflict, ProbeResult::Consistent(forced))
            | (ProbeResult::Consistent(forced), ProbeResult::Conflict) => {
                for &literal in forced {
                    inferred[literal] = true;
                }
            }
            (ProbeResult::Consistent(blank_forced), ProbeResult::Consistent(filled_forced)) => {
                scratch.fill(false);
                for &literal in blank_forced {
                    scratch[literal] = true;
                }
                for &literal in filled_forced {
                    if scratch[literal] {
                        inferred[literal] = true;
                    }
                }
            }
        }
    }

    Some(
        inferred
            .into_iter()
            .enumerate()
            .filter_map(|(literal, is_inferred)| is_inferred.then_some(literal))
            .collect(),
    )
}

fn probe(grid: &Grid, puzzle: &Puzzle, assumptions: &[usize], codec: &LiteralCodec) -> ProbeResult {
    let mut probed = grid.clone();
    for &literal in assumptions {
        let (row, col, value) = codec.decode(literal);
        let cell = probed.cell_mut(row, col);
        match *cell {
            Cell::Unknown => *cell = value,
            current if current == value => {}
            _ => return ProbeResult::Conflict,
        }
    }

    if !propagate(&mut probed, puzzle) {
        return ProbeResult::Conflict;
    }

    ProbeResult::Consistent(collect_forced_literals(grid, &probed, codec))
}

fn collect_forced_literals(base: &Grid, probed: &Grid, codec: &LiteralCodec) -> Vec<usize> {
    let mut forced = Vec::new();
    for row in 0..base.height() {
        for col in 0..base.width() {
            if *base.cell(row, col) != Cell::Unknown {
                continue;
            }
            let value = *probed.cell(row, col);
            if value == Cell::Unknown {
                continue;
            }
            let cell = codec.cell_index(row, col);
            forced.push(codec.encode(cell, value));
        }
    }
    forced
}

fn apply_literals(grid: &mut Grid, literals: &[usize], codec: &LiteralCodec) -> Option<()> {
    for &literal in literals {
        let (row, col, value) = codec.decode(literal);
        let cell = grid.cell_mut(row, col);
        match *cell {
            Cell::Unknown => *cell = value,
            current if current == value => {}
            _ => return None,
        }
    }
    Some(())
}

fn collect_unknown_cells(grid: &Grid) -> Vec<usize> {
    (0..grid.height())
        .flat_map(|row| (0..grid.width()).map(move |col| (row, col)))
        .filter_map(|(row, col)| (*grid.cell(row, col) == Cell::Unknown).then_some(row * grid.width() + col))
        .collect()
}

fn push_unique(values: &mut Vec<usize>, value: usize) -> bool {
    if values.contains(&value) {
        return false;
    }
    values.push(value);
    true
}

#[derive(Debug, Clone)]
enum ProbeResult {
    Conflict,
    Consistent(Vec<usize>),
}

#[derive(Debug, Clone, Copy)]
struct LiteralCodec {
    width: usize,
    height: usize,
}

impl LiteralCodec {
    fn new(grid: &Grid) -> Self {
        Self {
            width: grid.width(),
            height: grid.height(),
        }
    }

    fn literal_count(&self) -> usize {
        self.width * self.height * 2
    }

    fn cell_index(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    fn encode(&self, cell: usize, value: Cell) -> usize {
        let bit = match value {
            Cell::Blank => 0,
            Cell::Filled => 1,
            Cell::Unknown => panic!("unknown cells cannot be encoded as literals"),
        };
        cell * 2 + bit
    }

    fn decode(&self, literal: usize) -> (usize, usize, Cell) {
        let cell = literal / 2;
        let row = cell / self.width;
        let col = cell % self.width;
        let value = if literal % 2 == 0 {
            Cell::Blank
        } else {
            Cell::Filled
        };
        (row, col, value)
    }

    fn complement(&self, literal: usize) -> usize {
        literal ^ 1
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        format::puzzle_from_json,
        solver::{Fp1Solver, Fp2Solver, PartialSolver, PropagationSolver},
    };

    use super::*;

    #[test]
    fn fp_solvers_are_monotonic_on_cli_fixtures() {
        for fixture in input_fixtures() {
            let input = fs::read_to_string(&fixture)
                .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture.display()));
            let puzzle = puzzle_from_json(&input).unwrap();

            let linear = PropagationSolver.solve_partial(&puzzle).unwrap();
            let fp1 = Fp1Solver.solve_partial(&puzzle).unwrap();
            let fp2 = Fp2Solver.solve_partial(&puzzle).unwrap();

            assert!(
                count_known(&linear) <= count_known(&fp1),
                "fp1 regressed on {}",
                fixture.display()
            );
            assert!(
                count_known(&fp1) <= count_known(&fp2),
                "fp2 regressed on {}",
                fixture.display()
            );
        }
    }

    fn count_known(grid: &Grid) -> usize {
        (0..grid.height())
            .flat_map(|row| (0..grid.width()).map(move |col| (row, col)))
            .filter(|&(row, col)| *grid.cell(row, col) != Cell::Unknown)
            .count()
    }

    fn input_fixtures() -> Vec<PathBuf> {
        let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cli/tests/fixtures");
        let mut fixtures: Vec<_> = fs::read_dir(&fixture_dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_dir.display()))
            .map(|entry| entry.expect("failed to read fixture entry").path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".input.json"))
            })
            .collect();
        fixtures.sort();
        fixtures
    }
}
