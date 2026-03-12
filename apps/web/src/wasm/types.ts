export type Grid = boolean[][];
export type PartialGrid = Array<Array<boolean | null>>;

export type PartialSolverType = "linear" | "fp1" | "fp2";
export type CompleteSolverType =
  | "backtracking"
  | "fp1-backtracking"
  | "fp2-backtracking"
  | "sat";

export interface Puzzle {
  row_clues: number[][];
  col_clues: number[][];
}

export type SolutionStatus = "none" | "unique" | "multiple";

export interface Solution {
  status: SolutionStatus;
  grids: Grid[];
}

export interface ImageToGridParams {
  smooth_strength: number;
  threshold: number;
  edge_strength: number;
  noise_removal: number;
  grid_width: number;
  grid_height: number;
}
