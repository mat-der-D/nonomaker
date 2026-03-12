import type { Grid, PartialGrid, Puzzle } from "../wasm/types";

export function createGrid(width: number, height: number, value = false): Grid {
  return Array.from({ length: height }, () => Array.from({ length: width }, () => value));
}

export function createPartialGrid(width: number, height: number): PartialGrid {
  return Array.from({ length: height }, () => Array.from({ length: width }, () => null));
}

export function cloneGrid(grid: Grid): Grid {
  return grid.map((row) => [...row]);
}

export function equalGrid(a: Grid, b: Grid) {
  if (a.length !== b.length || a[0]?.length !== b[0]?.length) {
    return false;
  }

  return a.every((row, rowIndex) => row.every((cell, colIndex) => cell === b[rowIndex][colIndex]));
}

export function puzzleDimensions(puzzle: Puzzle) {
  return {
    width: puzzle.col_clues.length,
    height: puzzle.row_clues.length,
  };
}

export function maxClueDepth(puzzle: Puzzle) {
  return {
    maxRowClueSlots: Math.max(...puzzle.row_clues.map((clue) => clue.length), 0),
    maxColClueSlots: Math.max(...puzzle.col_clues.map((clue) => clue.length), 0),
  };
}
