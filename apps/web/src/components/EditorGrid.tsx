import { useMemo, useState, type CSSProperties } from "react";
import { cloneGrid } from "../utils/grid";
import type { Grid } from "../wasm/types";

export type EditorTool = "draw" | "erase" | "invert" | "line" | "rect" | "fill" | "zoom" | "pan";

interface EditorGridProps {
  grid: Grid;
  onChange: (next: Grid) => void;
  tool?: EditorTool;
  scalePercent?: number;
  playable?: boolean;
  readOnly?: boolean;
  marks?: boolean[][];
  onMarksChange?: (next: boolean[][]) => void;
}

export function EditorGrid({
  grid,
  onChange,
  tool = "draw",
  scalePercent = 100,
  playable = false,
  readOnly = false,
  marks,
  onMarksChange,
}: EditorGridProps) {
  const [dragPaintValue, setDragPaintValue] = useState<boolean | null>(null);
  const [dragVisited, setDragVisited] = useState<Set<string>>(() => new Set());
  const [pointerActive, setPointerActive] = useState(false);
  const [shapeStart, setShapeStart] = useState<{ row: number; col: number } | null>(null);
  const [shapeCurrent, setShapeCurrent] = useState<{ row: number; col: number } | null>(null);
  const [shapeErase, setShapeErase] = useState(false);
  const cellSize = useMemo(
    () => Math.max(14, Math.floor(450 / Math.max(grid.length, grid[0]?.length ?? 1))),
    [grid],
  );
  const gridWidth = (grid[0]?.length ?? 0) * cellSize;
  const gridHeight = grid.length * cellSize;
  const scaledWidth = gridWidth * (scalePercent / 100);
  const scaledHeight = gridHeight * (scalePercent / 100);
  const previewCells = useMemo(
    () =>
      shapeStart && shapeCurrent && !playable
        ? computeShapeCells(shapeStart, shapeCurrent, tool)
        : [],
    [playable, shapeCurrent, shapeStart, tool],
  );
  const previewLookup = useMemo(
    () => new Map(previewCells.map((cell) => [`${cell.row}-${cell.col}`, cell])),
    [previewCells],
  );

  function paint(row: number, col: number, nextValue: boolean) {
    const next = cloneGrid(grid);
    next[row][col] = nextValue;
    onChange(next);
  }

  function flip(row: number, col: number) {
    const next = cloneGrid(grid);
    next[row][col] = !next[row][col];
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

  function applyFill(row: number, col: number, nextValue: boolean) {
    if (grid[row][col] === nextValue) {
      return;
    }

    const target = grid[row][col];
    const next = cloneGrid(grid);
    const queue = [{ row, col }];
    next[row][col] = nextValue;

    while (queue.length > 0) {
      const current = queue.shift();
      if (!current) {
        continue;
      }
      const neighbors = [
        { row: current.row - 1, col: current.col },
        { row: current.row + 1, col: current.col },
        { row: current.row, col: current.col - 1 },
        { row: current.row, col: current.col + 1 },
      ];

      for (const neighbor of neighbors) {
        if (
          neighbor.row < 0 ||
          neighbor.row >= grid.length ||
          neighbor.col < 0 ||
          neighbor.col >= grid[0].length ||
          next[neighbor.row][neighbor.col] !== target
        ) {
          continue;
        }

        next[neighbor.row][neighbor.col] = nextValue;
        queue.push(neighbor);
      }
    }

    onChange(next);
  }

  function applyShape(endRow: number, endCol: number) {
    if (!shapeStart) {
      return;
    }
    const cells = computeShapeCells(shapeStart, { row: endRow, col: endCol }, tool);
    if (cells.length === 0) {
      return;
    }
    const next = cloneGrid(grid);
    for (const cell of cells) {
      next[cell.row][cell.col] = shapeErase ? false : true;
    }
    onChange(next);
  }

  function resetPointerState() {
    setPointerActive(false);
    setDragPaintValue(null);
    setDragVisited(new Set());
    setShapeStart(null);
    setShapeCurrent(null);
    setShapeErase(false);
  }

  function handleMakerPointerDown(row: number, col: number, secondary: boolean) {
    setPointerActive(true);

    if (tool === "line" || tool === "rect") {
      setShapeStart({ row, col });
      setShapeCurrent({ row, col });
      setShapeErase(secondary);
      return;
    }

    if (tool === "fill") {
      applyFill(row, col, !secondary);
      return;
    }

    if (tool === "invert") {
      const key = `${row}-${col}`;
      setDragVisited(new Set([key]));
      flip(row, col);
      return;
    }

    const nextValue = tool === "erase" || secondary ? false : true;
    setDragPaintValue(nextValue);
    paint(row, col, nextValue);
  }

  function handleMakerPointerEnter(row: number, col: number) {
    if (tool === "line" || tool === "rect") {
      if (shapeStart) {
        setShapeCurrent({ row, col });
      }
      return;
    }

    if (tool === "invert") {
      const key = `${row}-${col}`;
      if (dragVisited.has(key)) {
        return;
      }
      setDragVisited((current) => new Set(current).add(key));
      flip(row, col);
      return;
    }

    if (dragPaintValue !== null) {
      paint(row, col, dragPaintValue);
    }
  }

  function handleMakerPointerUp(row: number, col: number) {
    if (tool === "line" || tool === "rect") {
      applyShape(row, col);
    }
    resetPointerState();
  }

  return (
    <div className="grid-shell">
      <div
        className="editor-grid-scale-shell"
        style={{ width: scaledWidth, height: scaledHeight }}
      >
        <div
          className="editor-grid-scale"
          style={{ "--editor-grid-scale": scalePercent / 100 } as CSSProperties}
        >
        <div
          className={`editor-grid ${playable ? "play-mode" : ""} ${tool === "zoom" ? "zoom-mode" : ""} ${tool === "pan" ? "pan-mode" : ""}`}
          style={{ gridTemplateColumns: `repeat(${grid[0]?.length ?? 0}, ${cellSize}px)` }}
          onPointerLeave={() => {
            setPointerActive(false);
            setDragPaintValue(null);
            setDragVisited(new Set());
            if (tool !== "line" && tool !== "rect") {
              setShapeStart(null);
              setShapeCurrent(null);
            }
          }}
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
                    previewLookup.has(`${rowIndex}-${colIndex}`) && !shapeErase ? "preview-filled" : "",
                    previewLookup.has(`${rowIndex}-${colIndex}`) && shapeErase ? "preview-cleared" : "",
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
                      handleMakerPointerDown(rowIndex, colIndex, true);
                    }
                  }}
                  onPointerDown={(event) => {
                    if (readOnly) {
                      return;
                    }
                    if (playable) {
                      togglePlayable(rowIndex, colIndex, event.button === 2);
                    } else {
                      handleMakerPointerDown(rowIndex, colIndex, event.button === 2);
                    }
                  }}
                  onPointerEnter={() => {
                    if (readOnly || playable || !pointerActive || (dragPaintValue === null && tool === "draw") || (dragPaintValue === null && tool === "erase")) {
                      if (!readOnly && !playable && pointerActive && (tool === "line" || tool === "rect" || tool === "invert")) {
                        handleMakerPointerEnter(rowIndex, colIndex);
                      }
                      return;
                    }
                    handleMakerPointerEnter(rowIndex, colIndex);
                  }}
                  onPointerUp={() => {
                    if (!playable && !readOnly) {
                      handleMakerPointerUp(rowIndex, colIndex);
                      return;
                    }
                    setDragPaintValue(null);
                  }}
                  aria-label={`cell ${rowIndex + 1}-${colIndex + 1}`}
                >
                  {marks?.[rowIndex]?.[colIndex] ? "×" : ""}
                </button>
              )),
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function computeShapeCells(
  start: { row: number; col: number },
  current: { row: number; col: number },
  tool: EditorTool,
) {
  if (tool === "line") {
    return getLineCells(start, current).map((cell) => ({ ...cell, nextValue: true }));
  }

  if (tool === "rect") {
    const top = Math.min(start.row, current.row);
    const bottom = Math.max(start.row, current.row);
    const left = Math.min(start.col, current.col);
    const right = Math.max(start.col, current.col);
    const cells = [];

    for (let row = top; row <= bottom; row += 1) {
      for (let col = left; col <= right; col += 1) {
        cells.push({ row, col, nextValue: true });
      }
    }

    return cells;
  }

  return [];
}

function getLineCells(start: { row: number; col: number }, end: { row: number; col: number }) {
  const cells = [];
  const deltaCol = Math.abs(end.col - start.col);
  const deltaRow = Math.abs(end.row - start.row);
  const stepCol = start.col < end.col ? 1 : -1;
  const stepRow = start.row < end.row ? 1 : -1;
  let error = deltaCol - deltaRow;
  let col = start.col;
  let row = start.row;

  while (true) {
    cells.push({ row, col });
    if (col === end.col && row === end.row) {
      break;
    }
    const doubled = error * 2;
    if (doubled > -deltaRow) {
      error -= deltaRow;
      col += stepCol;
    }
    if (doubled < deltaCol) {
      error += deltaCol;
      row += stepRow;
    }
  }

  return cells;
}
