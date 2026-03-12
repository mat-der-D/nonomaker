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

type PaintMode = "fill" | "blank";

export function EditorGrid({
  grid,
  onChange,
  playable = false,
  readOnly = false,
  marks,
  onMarksChange,
}: EditorGridProps) {
  const [paintMode, setPaintMode] = useState<PaintMode>("fill");
  const [isPointerDown, setPointerDown] = useState(false);
  const cellSize = useMemo(() => Math.max(14, Math.floor(560 / Math.max(grid.length, grid[0]?.length ?? 1))), [grid]);

  function paint(row: number, col: number, mode: PaintMode) {
    const next = grid.map((cells) => [...cells]);
    next[row][col] = mode === "fill";
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
      {!playable && !readOnly && (
        <div className="paint-switch" role="group" aria-label="paint mode">
          <button
            className={paintMode === "fill" ? "active" : ""}
            onClick={() => setPaintMode("fill")}
            type="button"
          >
            Fill
          </button>
          <button
            className={paintMode === "blank" ? "active" : ""}
            onClick={() => setPaintMode("blank")}
            type="button"
          >
            Erase
          </button>
        </div>
      )}
      <div
        className={`editor-grid ${playable ? "play-mode" : ""}`}
        style={{ gridTemplateColumns: `repeat(${grid[0]?.length ?? 0}, ${cellSize}px)` }}
        onPointerLeave={() => setPointerDown(false)}
      >
        {grid.map((row, rowIndex) =>
          row.map((cell, colIndex) => (
            <button
              key={`${rowIndex}-${colIndex}`}
              type="button"
              className={`cell ${cell ? "filled" : ""} ${marks?.[rowIndex]?.[colIndex] ? "marked" : ""}`}
              onContextMenu={(event) => {
                event.preventDefault();
                if (readOnly) {
                  return;
                }
                if (playable) {
                  togglePlayable(rowIndex, colIndex, true);
                } else {
                  paint(rowIndex, colIndex, "blank");
                }
              }}
              onPointerDown={(event) => {
                setPointerDown(true);
                if (readOnly) {
                  return;
                }
                if (playable) {
                  togglePlayable(rowIndex, colIndex, event.button === 2);
                } else {
                  const mode = event.button === 2 ? "blank" : paintMode;
                  paint(rowIndex, colIndex, mode);
                }
              }}
              onPointerEnter={() => {
                if (readOnly || !isPointerDown || playable) {
                  return;
                }
                paint(rowIndex, colIndex, paintMode);
              }}
              onPointerUp={() => setPointerDown(false)}
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
