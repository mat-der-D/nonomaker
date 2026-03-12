import type { Grid, Puzzle } from "../wasm/types";
import { maxClueDepth } from "../utils/grid";

interface PuzzleBoardProps {
  puzzle: Puzzle;
  progress: Grid;
  marks: boolean[][];
  onProgressChange: (grid: Grid) => void;
  onMarksChange: (marks: boolean[][]) => void;
}

export function PuzzleBoard({
  puzzle,
  progress,
  marks,
  onProgressChange,
  onMarksChange,
}: PuzzleBoardProps) {
  const { rows, cols } = maxClueDepth(puzzle);

  function toggleCell(row: number, col: number) {
    const next = progress.map((line) => [...line]);
    next[row][col] = !next[row][col];
    onProgressChange(next);
  }

  function toggleMark(row: number, col: number) {
    const next = marks.map((line) => [...line]);
    next[row][col] = !next[row][col];
    onMarksChange(next);
  }

  return (
    <div className="puzzle-board">
      <div className="puzzle-corner" style={{ width: cols * 24, height: rows * 24 }} />
      <div className="column-clues">
        {puzzle.col_clues.map((clue, index) => (
          <div key={index} className="clue-stack vertical">
            {Array.from({ length: rows }, (_, slot) => clue[clue.length - rows + slot] ?? "")}
          </div>
        ))}
      </div>
      <div className="row-clues">
        {puzzle.row_clues.map((clue, index) => (
          <div key={index} className="clue-stack horizontal">
            {Array.from({ length: cols }, (_, slot) => clue[clue.length - cols + slot] ?? "")}
          </div>
        ))}
      </div>
      <div className="play-grid" style={{ gridTemplateColumns: `repeat(${progress[0]?.length ?? 0}, 32px)` }}>
        {progress.map((row, rowIndex) =>
          row.map((cell, colIndex) => (
            <button
              key={`${rowIndex}-${colIndex}`}
              type="button"
              className={`cell ${cell ? "filled" : ""} ${marks[rowIndex][colIndex] ? "marked" : ""}`}
              onClick={() => toggleCell(rowIndex, colIndex)}
              onContextMenu={(event) => {
                event.preventDefault();
                toggleMark(rowIndex, colIndex);
              }}
            >
              {marks[rowIndex][colIndex] ? "×" : ""}
            </button>
          )),
        )}
      </div>
    </div>
  );
}
