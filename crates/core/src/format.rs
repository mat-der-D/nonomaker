use serde::{Deserialize, Serialize};

use crate::{
    solver::Solution,
    types::{Cell, Clue, Grid, Puzzle},
};

#[derive(Debug)]
pub enum FormatError {
    UnknownCell,
    InvalidGrid(String),
    Json(serde_json::Error),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::UnknownCell => write!(f, "grid contains Unknown cells"),
            FormatError::InvalidGrid(msg) => write!(f, "invalid grid: {msg}"),
            FormatError::Json(e) => write!(f, "json error: {e}"),
        }
    }
}

impl std::error::Error for FormatError {}

impl From<serde_json::Error> for FormatError {
    fn from(e: serde_json::Error) -> Self {
        FormatError::Json(e)
    }
}

// ---- internal serde types ----

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct GridJson(Vec<Vec<bool>>);

#[derive(Serialize, Deserialize)]
struct PuzzleJson {
    row_clues: Vec<Vec<usize>>,
    col_clues: Vec<Vec<usize>>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum SolutionJson {
    None { grids: Vec<GridJson> },
    Unique { grids: Vec<GridJson> },
    Multiple { grids: Vec<GridJson> },
}

// ---- Grid ----

fn grid_to_json_type(grid: &Grid) -> Result<GridJson, FormatError> {
    let mut rows = Vec::with_capacity(grid.height());
    for r in 0..grid.height() {
        let mut row = Vec::with_capacity(grid.width());
        for c in 0..grid.width() {
            match grid.cell(r, c) {
                Cell::Filled => row.push(true),
                Cell::Blank => row.push(false),
                Cell::Unknown => return Err(FormatError::UnknownCell),
            }
        }
        rows.push(row);
    }
    Ok(GridJson(rows))
}

fn grid_from_json_type(json: GridJson) -> Result<Grid, FormatError> {
    let height = json.0.len();
    let width = json.0.first().map(|r| r.len()).unwrap_or(0);
    for (i, row) in json.0.iter().enumerate() {
        if row.len() != width {
            return Err(FormatError::InvalidGrid(format!(
                "row {i} has width {} but expected {width}",
                row.len()
            )));
        }
    }
    let mut grid = Grid::new(width, height);
    for (r, row) in json.0.iter().enumerate() {
        for (c, &filled) in row.iter().enumerate() {
            *grid.cell_mut(r, c) = if filled { Cell::Filled } else { Cell::Blank };
        }
    }
    Ok(grid)
}

pub fn grid_to_json(grid: &Grid) -> Result<String, FormatError> {
    Ok(serde_json::to_string(&grid_to_json_type(grid)?)?)
}

pub fn grid_from_json(s: &str) -> Result<Grid, FormatError> {
    let json: GridJson = serde_json::from_str(s)?;
    grid_from_json_type(json)
}

// ---- Puzzle ----

pub fn puzzle_to_json(puzzle: &Puzzle) -> String {
    let json = PuzzleJson {
        row_clues: puzzle
            .row_clues()
            .iter()
            .map(|c| c.blocks().to_vec())
            .collect(),
        col_clues: puzzle
            .col_clues()
            .iter()
            .map(|c| c.blocks().to_vec())
            .collect(),
    };
    serde_json::to_string(&json).expect("PuzzleJson serialization is infallible")
}

pub fn puzzle_from_json(s: &str) -> Result<Puzzle, FormatError> {
    let json: PuzzleJson = serde_json::from_str(s)?;
    Ok(Puzzle::new(
        json.row_clues.into_iter().map(Clue::new).collect(),
        json.col_clues.into_iter().map(Clue::new).collect(),
    ))
}

// ---- Solution ----

pub fn solution_to_json(solution: &Solution) -> Result<String, FormatError> {
    let json = match solution {
        Solution::None => SolutionJson::None { grids: vec![] },
        Solution::Unique(grid) => SolutionJson::Unique {
            grids: vec![grid_to_json_type(grid)?],
        },
        Solution::Multiple(grids) => SolutionJson::Multiple {
            grids: grids
                .iter()
                .map(grid_to_json_type)
                .collect::<Result<_, _>>()?,
        },
    };
    Ok(serde_json::to_string(&json)?)
}

pub fn solution_from_json(s: &str) -> Result<Solution, FormatError> {
    let json: SolutionJson = serde_json::from_str(s)?;
    Ok(match json {
        SolutionJson::None { .. } => Solution::None,
        SolutionJson::Unique { grids } => {
            let grid = grids
                .into_iter()
                .next()
                .ok_or_else(|| FormatError::InvalidGrid("unique solution has no grid".into()))?;
            Solution::Unique(grid_from_json_type(grid)?)
        }
        SolutionJson::Multiple { grids } => Solution::Multiple(
            grids
                .into_iter()
                .map(grid_from_json_type)
                .collect::<Result<_, _>>()?,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Cell, Clue, Grid, Puzzle};

    fn filled_grid_2x2() -> Grid {
        let mut g = Grid::new(2, 2);
        *g.cell_mut(0, 0) = Cell::Filled;
        *g.cell_mut(0, 1) = Cell::Blank;
        *g.cell_mut(1, 0) = Cell::Blank;
        *g.cell_mut(1, 1) = Cell::Filled;
        g
    }

    #[test]
    fn grid_roundtrip() {
        let grid = filled_grid_2x2();
        let s = grid_to_json(&grid).unwrap();
        assert_eq!(s, "[[true,false],[false,true]]");
        let grid2 = grid_from_json(&s).unwrap();
        assert_eq!(grid, grid2);
    }

    #[test]
    fn grid_unknown_cell_errors() {
        let grid = Grid::new(2, 2); // all Unknown
        assert!(matches!(grid_to_json(&grid), Err(FormatError::UnknownCell)));
    }

    #[test]
    fn puzzle_roundtrip() {
        let puzzle = Puzzle::new(
            vec![Clue::new(vec![1, 2]), Clue::new(vec![])],
            vec![Clue::new(vec![3]), Clue::new(vec![1, 1])],
        );
        let s = puzzle_to_json(&puzzle);
        let puzzle2 = puzzle_from_json(&s).unwrap();
        assert_eq!(puzzle.width(), puzzle2.width());
        assert_eq!(puzzle.height(), puzzle2.height());
        assert_eq!(
            puzzle.row_clues()[0].blocks(),
            puzzle2.row_clues()[0].blocks()
        );
        assert_eq!(
            puzzle.col_clues()[1].blocks(),
            puzzle2.col_clues()[1].blocks()
        );
    }

    #[test]
    fn solution_none_roundtrip() {
        let s = solution_to_json(&Solution::None).unwrap();
        assert_eq!(s, r#"{"status":"none","grids":[]}"#);
        assert!(matches!(solution_from_json(&s).unwrap(), Solution::None));
    }

    #[test]
    fn solution_unique_roundtrip() {
        let sol = Solution::Unique(filled_grid_2x2());
        let s = solution_to_json(&sol).unwrap();
        let sol2 = solution_from_json(&s).unwrap();
        assert!(matches!(sol2, Solution::Unique(_)));
    }

    #[test]
    fn solution_multiple_roundtrip() {
        let sol = Solution::Multiple(vec![filled_grid_2x2(), filled_grid_2x2()]);
        let s = solution_to_json(&sol).unwrap();
        let sol2 = solution_from_json(&s).unwrap();
        assert!(matches!(sol2, Solution::Multiple(ref v) if v.len() == 2));
    }
}
