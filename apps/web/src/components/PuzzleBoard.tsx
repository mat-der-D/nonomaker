import { useEffect, useMemo, useState } from "react";
import type { Puzzle } from "../wasm/types";
import { maxClueDepth } from "../utils/grid";

export type PlayCell = "unknown" | "filled" | "crossed";

interface PuzzleBoardProps {
  puzzle: Puzzle;
  cells: PlayCell[][];
  onCellsChange: (cells: PlayCell[][]) => void;
}

export function PuzzleBoard({
  puzzle,
  cells,
  onCellsChange,
}: PuzzleBoardProps) {
  const { maxRowClueSlots, maxColClueSlots } = maxClueDepth(puzzle);
  const boardCellSize = 32;
  const rowClueAreaWidth = maxRowClueSlots * boardCellSize + 4;
  const colClueAreaHeight = maxColClueSlots * boardCellSize + 4;
  const boardAreaWidth = puzzle.col_clues.length * boardCellSize + 4;
  const [activeDrag, setActiveDrag] = useState<{
    active: boolean;
    value: PlayCell;
  }>({
    active: false,
    value: "filled",
  });

  useEffect(() => {
    function stopActiveDrag() {
      setActiveDrag((current) => ({ ...current, active: false }));
    }

    window.addEventListener("pointerup", stopActiveDrag);
    return () => window.removeEventListener("pointerup", stopActiveDrag);
  }, []);

  const solvedRows = useMemo(
    () => cells.map((cellRow, rowIndex) => equalClues(computeClues(cellRow), puzzle.row_clues[rowIndex])),
    [cells, puzzle.row_clues],
  );
  const solvedColumns = useMemo(
    () =>
      puzzle.col_clues.map((columnClue, columnIndex) =>
        equalClues(
          computeClues(cells.map((cellRow) => cellRow[columnIndex])),
          columnClue,
        ),
      ),
    [cells, puzzle.col_clues],
  );

  function overwriteCell(cellRowIndex: number, cellColumnIndex: number, nextCellValue: PlayCell) {
    const nextCells = cells.map((cellRow) => [...cellRow]);
    nextCells[cellRowIndex][cellColumnIndex] = nextCellValue;
    onCellsChange(nextCells);
  }

  function resolveNextPlayCell(currentCell: PlayCell, isSecondaryAction: boolean): PlayCell {
    if (isSecondaryAction) {
      return currentCell === "filled" ? "unknown" : "crossed";
    }
    return currentCell === "filled" ? "unknown" : "filled";
  }

  return (
    <div
      className="puzzle-board"
      style={{
        gridTemplateColumns: `${rowClueAreaWidth}px auto`,
        gridTemplateRows: `${colClueAreaHeight}px auto`,
      }}
    >
      <div
        className="puzzle-corner"
        style={{ width: rowClueAreaWidth, height: colClueAreaHeight }}
      />
      <div
        className="column-clues"
        style={{
          width: boardAreaWidth,
          height: colClueAreaHeight,
          gridTemplateColumns: `repeat(${puzzle.col_clues.length}, ${boardCellSize}px)`,
          gridTemplateRows: `repeat(${maxColClueSlots}, ${boardCellSize}px)`,
        }}
      >
        {Array.from({ length: maxColClueSlots }, (_, clueRowIndex) =>
          puzzle.col_clues.map((columnClue, columnIndex) => {
            const clueValue = columnClue[columnClue.length - maxColClueSlots + clueRowIndex] ?? "";
            return (
              <span
                key={`col-${columnIndex}-${clueRowIndex}`}
                className={`clue-cell ${solvedColumns[columnIndex] ? "solved" : ""}`}
              >
                {clueValue}
              </span>
            );
          }),
        )}
      </div>
      <div
        className="row-clues"
        style={{
          width: rowClueAreaWidth,
          gridTemplateColumns: `repeat(${maxRowClueSlots}, ${boardCellSize}px)`,
          gridTemplateRows: `repeat(${puzzle.row_clues.length}, ${boardCellSize}px)`,
        }}
      >
        {puzzle.row_clues.flatMap((rowClue, puzzleRowIndex) =>
          Array.from({ length: maxRowClueSlots }, (_, clueColumnIndex) => {
            const clueValue = rowClue[rowClue.length - maxRowClueSlots + clueColumnIndex] ?? "";
            return (
              <span
                key={`row-${puzzleRowIndex}-${clueColumnIndex}`}
                className={`clue-cell ${solvedRows[puzzleRowIndex] ? "solved" : ""}`}
              >
                {clueValue}
              </span>
            );
          }),
        )}
      </div>
      <div
        className="play-grid"
        style={{ gridTemplateColumns: `repeat(${cells[0]?.length ?? 0}, ${boardCellSize}px)` }}
        onPointerLeave={() => setActiveDrag((current) => ({ ...current, active: false }))}
      >
        {cells.map((cellRow, cellRowIndex) =>
          cellRow.map((cellValue, cellColumnIndex) => (
            <button
              key={`${cellRowIndex}-${cellColumnIndex}`}
              type="button"
              className={["cell", `play-cell-${cellValue}`].join(" ")}
              onPointerDown={(event) => {
                const nextCellValue = resolveNextPlayCell(cellValue, event.button === 2);
                setActiveDrag({ active: true, value: nextCellValue });
                overwriteCell(cellRowIndex, cellColumnIndex, nextCellValue);
              }}
              onPointerEnter={() => {
                if (!activeDrag.active) {
                  return;
                }
                overwriteCell(cellRowIndex, cellColumnIndex, activeDrag.value);
              }}
              onContextMenu={(event) => {
                event.preventDefault();
              }}
              aria-label={`cell ${cellRowIndex + 1}-${cellColumnIndex + 1}`}
            >
              {cellValue === "crossed" ? "×" : ""}
            </button>
          )),
        )}
      </div>
    </div>
  );
}

function computeClues(line: PlayCell[]) {
  const groups: number[] = [];
  let run = 0;

  line.forEach((cell) => {
    if (cell === "filled") {
      run += 1;
      return;
    }
    if (run > 0) {
      groups.push(run);
      run = 0;
    }
  });

  if (run > 0) {
    groups.push(run);
  }

  return groups;
}

function equalClues(a: number[], b: number[]) {
  return a.length === b.length && a.every((value, index) => value === b[index]);
}
