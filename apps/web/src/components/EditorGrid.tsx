import { useMemo, useState } from "react";
import type { Grid } from "../wasm/types";

interface EditorGridProps {
  grid: Grid;
  onChange: (next: Grid) => void;
  playable?: boolean;
  readOnly?: boolean;
  marks?: boolean[][];
  onMarksChange?: (next: boolean[][]) => void;
}

export function EditorGrid({
  grid,
  onChange,
  playable = false,
  readOnly = false,
  marks,
  onMarksChange,
}: EditorGridProps) {
  const [dragPaintValue, setDragPaintValue] = useState<boolean | null>(null);
  const cellSize = useMemo(() => Math.max(14, Math.floor(560 / Math.max(grid.length, grid[0]?.length ?? 1))), [grid]);

  function paint(row: number, col: number, nextValue: boolean) {
    const next = grid.map((cells) => [...cells]);
    next[row][col] = nextValue;
    onChange(next);
  }

  function togglePlayable(row: number, col: number, secondary: boolean) {
    if (!playable) {
      return;
    }
    if (secondary) {
      const nextMarks = marks?.map((line) => [...line]) ?? grid.map((line) => line.map(() => false));
      nextMarks[row][col] = !nextMarks[row][col];
      onMarksChange?.(nextMarks);
      return;
    }

    const next = grid.map((cells) => [...cells]);
    next[row][col] = !next[row][col];
    onChange(next);
  }

  return (
    <div className="grid-shell">
      <div
        className={`editor-grid ${playable ? "play-mode" : ""}`}
        style={{ gridTemplateColumns: `repeat(${grid[0]?.length ?? 0}, ${cellSize}px)` }}
        onPointerLeave={() => setDragPaintValue(null)}
      >
        {grid.map((row, rowIndex) =>
          row.map((cell, colIndex) => (
            <button
              key={`${rowIndex}-${colIndex}`}
              type="button"
              className={[
                "cell",
                cell ? "filled" : "",
                marks?.[rowIndex]?.[colIndex] ? "marked" : "",
                rowIndex > 0 && rowIndex % 5 === 0 ? "major-top" : "",
                colIndex > 0 && colIndex % 5 === 0 ? "major-left" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              onContextMenu={(event) => {
                event.preventDefault();
                if (readOnly) {
                  return;
                }
                if (playable) {
                  togglePlayable(rowIndex, colIndex, true);
                } else {
                  paint(rowIndex, colIndex, false);
                }
              }}
              onPointerDown={(event) => {
                if (readOnly) {
                  return;
                }
                if (playable) {
                  togglePlayable(rowIndex, colIndex, event.button === 2);
                } else {
                  const nextValue = event.button === 2 ? false : true;
                  setDragPaintValue(nextValue);
                  paint(rowIndex, colIndex, nextValue);
                }
              }}
              onPointerEnter={() => {
                if (readOnly || dragPaintValue === null || playable) {
                  return;
                }
                paint(rowIndex, colIndex, dragPaintValue);
              }}
              onPointerUp={() => setDragPaintValue(null)}
              aria-label={`cell ${rowIndex + 1}-${colIndex + 1}`}
            >
              {marks?.[rowIndex]?.[colIndex] ? "×" : ""}
            </button>
          )),
        )}
      </div>
    </div>
  );
}
