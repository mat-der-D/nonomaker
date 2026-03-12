import { useEffect, useMemo, useState } from "react";
import { EditorGrid } from "./components/EditorGrid";
import { PuzzleBoard } from "./components/PuzzleBoard";
import { useWasm } from "./hooks/useWasm";
import { createGrid, equalGrid, puzzleDimensions } from "./utils/grid";
import {
  gridToId,
  gridToPuzzle,
  idToGrid,
  imageToGrid,
  solveComplete,
  solvePartial,
} from "./wasm/api";
import type { Grid, ImageToGridParams, Puzzle, Solution } from "./wasm/types";

type Route =
  | { kind: "maker" }
  | { kind: "play"; id: string };

interface AnalysisState {
  solution: Solution | null;
  partialRatio: number | null;
  message: string | null;
}

const defaultImageParams: ImageToGridParams = {
  smooth_strength: 1,
  threshold: 128,
  edge_strength: 0.3,
  noise_removal: 0,
  grid_width: 20,
  grid_height: 20,
};

export default function App() {
  const wasm = useWasm();
  const route = useMemo<Route>(() => {
    const [, head, maybeId] = window.location.pathname.split("/");
    if (head === "play" && maybeId) {
      return { kind: "play", id: maybeId };
    }
    return { kind: "maker" };
  }, []);

  if (wasm.isLoading) {
    return <div className="screen-state">WASM を読み込み中...</div>;
  }

  if (!wasm.isReady) {
    return <div className="screen-state">WASM の初期化に失敗しました。</div>;
  }

  return route.kind === "play" ? <PlayPage id={route.id} /> : <MakerPage />;
}

function MakerPage() {
  const [grid, setGrid] = useState<Grid>(() => createGrid(20, 20));
  const [history, setHistory] = useState<Grid[]>([]);
  const [future, setFuture] = useState<Grid[]>([]);
  const [size, setSize] = useState({ width: 20, height: 20 });
  const [analysis, setAnalysis] = useState<AnalysisState>({
    solution: null,
    partialRatio: null,
    message: null,
  });
  const [busy, setBusy] = useState<string | null>(null);
  const [shareUrl, setShareUrl] = useState("");
  const [imageModalOpen, setImageModalOpen] = useState(false);

  useEffect(() => {
    setSize({ width: grid[0]?.length ?? 0, height: grid.length });
  }, [grid]);

  function commit(next: Grid) {
    setHistory((current) => [...current, grid]);
    setFuture([]);
    setGrid(next);
  }

  function resizeGrid() {
    const next = createGrid(size.width, size.height);
    for (let row = 0; row < Math.min(grid.length, next.length); row += 1) {
      for (let col = 0; col < Math.min(grid[0].length, next[0].length); col += 1) {
        next[row][col] = grid[row][col];
      }
    }
    commit(next);
  }

  async function runCheck() {
    setBusy("checking");
    try {
      const puzzle = await gridToPuzzle(grid);
      const [solution, partial] = await Promise.all([
        solveComplete(puzzle, "backtracking"),
        solvePartial(puzzle, "linear"),
      ]);
      const width = grid[0].length;
      const height = grid.length;
      const resolved =
        partial?.flat().filter((cell) => cell !== null).length ?? 0;
      const partialRatio = resolved / (width * height);
      const message =
        solution.status === "unique"
          ? "一意解です。共有とエクスポートを有効化しました。"
          : solution.status === "multiple"
            ? `複数解です (${solution.grids.length}件)。`
            : "解なしです。";
      setAnalysis({ solution, partialRatio, message });
    } catch (error) {
      setAnalysis({ solution: null, partialRatio: null, message: String(error) });
    } finally {
      setBusy(null);
    }
  }

  async function generateShare() {
    setBusy("share");
    try {
      const id = await gridToId(grid);
      const url = `${window.location.origin}/play/${id}`;
      setShareUrl(url);
      await navigator.clipboard.writeText(url);
      setAnalysis((current) => ({
        ...current,
        message: "共有URLをコピーしました。",
      }));
    } catch (error) {
      setAnalysis((current) => ({ ...current, message: String(error) }));
    } finally {
      setBusy(null);
    }
  }

  async function exportArtifacts() {
    try {
      const puzzle = await gridToPuzzle(grid);
      downloadBlob("puzzle.json", JSON.stringify(puzzle), "application/json");
      downloadBlob("solution.json", JSON.stringify(grid), "application/json");
      downloadBlob("solution.svg", renderGridSvg(grid), "image/svg+xml");
      downloadBlob("solution.png", await renderGridPng(grid), "image/png");
      setAnalysis((current) => ({ ...current, message: "JSON / SVG / PNG を出力しました。" }));
    } catch (error) {
      setAnalysis((current) => ({ ...current, message: String(error) }));
    }
  }

  async function importJson(file: File) {
    const text = await file.text();
    const parsed = JSON.parse(text) as unknown;

    if (isGrid(parsed)) {
      commit(parsed);
      return;
    }

    if (isPuzzle(parsed)) {
      setBusy("import");
      try {
        const solution = await solveComplete(parsed, "backtracking");
        if (solution.status !== "unique") {
          throw new Error("読み込んだ問題は一意解ではありません。");
        }
        commit(solution.grids[0]);
        setAnalysis((current) => ({ ...current, message: "問題JSONを読み込みました。" }));
      } finally {
        setBusy(null);
      }
      return;
    }

    throw new Error("サポートしていない JSON 形式です。");
  }

  const exportAllowed = analysis.solution?.status === "unique";

  return (
    <div className="app-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">Nonogram puzzle maker</p>
          <h1>Nono<span>Maker</span></h1>
        </div>
        <a className="ghost-link" href="/maker">
          /maker
        </a>
      </header>

      <section className="toolbar">
        <div className="toolbar-group">
          <input
            type="number"
            min={5}
            max={50}
            value={size.width}
            onChange={(event) => setSize((current) => ({ ...current, width: clampSize(event.target.value) }))}
          />
          <span className="toolbar-x">×</span>
          <input
            type="number"
            min={5}
            max={50}
            value={size.height}
            onChange={(event) => setSize((current) => ({ ...current, height: clampSize(event.target.value) }))}
          />
          <button type="button" className="btn btn-subtle" onClick={resizeGrid}>
            ✓ 適用
          </button>
          <button
            type="button"
            className="btn btn-subtle"
            onClick={() => setImageModalOpen(true)}
          >
            🖼 画像変換
          </button>
          <label className="file-button btn btn-subtle">
            📂 読み込み
            <input type="file" accept=".json,application/json" onChange={(event) => event.target.files?.[0] && void importJson(event.target.files[0])} />
          </label>
        </div>

        <div className="toolbar-sep" />

        <div className="toolbar-group">
          <button type="button" className="btn btn-ghost" onClick={() => history.length && (setFuture((current) => [grid, ...current]), setGrid(history[history.length - 1]), setHistory((current) => current.slice(0, -1)))} disabled={history.length === 0}>
            ↩ Undo
          </button>
          <button type="button" className="btn btn-ghost" onClick={() => future.length && (setHistory((current) => [...current, grid]), setGrid(future[0]), setFuture((current) => current.slice(1)))} disabled={future.length === 0}>
            ↪ Redo
          </button>
          <button type="button" className="btn btn-ghost" onClick={() => window.confirm("盤面をクリアしますか？") && commit(createGrid(grid[0].length, grid.length))}>
            🗑 クリア
          </button>
        </div>

        <div className="toolbar-sep" />

        <div className="toolbar-group">
          <button type="button" className="btn btn-subtle" onClick={() => void runCheck()} disabled={busy !== null}>
            ✔ 解答チェック
          </button>
          <button type="button" className="btn btn-subtle" onClick={() => void runCheck()} disabled={busy !== null}>
            📊 難易度チェック
          </button>
          <button type="button" className="btn btn-subtle" onClick={() => void exportArtifacts()} disabled={!exportAllowed}>
            ⬇ ファイル出力
          </button>
          <button type="button" className="btn btn-primary" onClick={() => void generateShare()} disabled={!exportAllowed}>
            🔗 共有
          </button>
        </div>
      </section>

      <section className="content">
        <div className="editor-panel">
          <EditorGrid
            grid={grid}
            onChange={(next) => {
              if (!equalGrid(next, grid)) {
                commit(next);
              }
            }}
          />
        </div>

        <aside className="side-panel">
          <section className="card">
            <h2>Status</h2>
            <p>{busy ? `${busy}...` : analysis.message ?? "盤面を編集して解答チェックを実行してください。"}</p>
            {analysis.partialRatio !== null && (
              <>
                <p>論理確定率: {Math.round(analysis.partialRatio * 100)}%</p>
                <div className="ratio-bar-track">
                  <div className="ratio-bar-fill" style={{ width: `${Math.round(analysis.partialRatio * 100)}%` }} />
                </div>
              </>
            )}
            {shareUrl && <input className="share-input" readOnly value={shareUrl} />}
          </section>
        </aside>
      </section>

      {imageModalOpen && (
        <ImageConvertModal
          initialParams={{
            ...defaultImageParams,
            grid_width: grid[0].length,
            grid_height: grid.length,
          }}
          onApply={(next) => {
            commit(next);
            setAnalysis((current) => ({
              ...current,
              message: "画像変換の結果を適用しました。",
            }));
            setImageModalOpen(false);
          }}
          onClose={() => setImageModalOpen(false)}
        />
      )}
    </div>
  );
}

function ImageConvertModal({
  initialParams,
  onApply,
  onClose,
}: {
  initialParams: ImageToGridParams;
  onApply: (grid: Grid) => void;
  onClose: () => void;
}) {
  const [params, setParams] = useState<ImageToGridParams>(initialParams);
  const [source, setSource] = useState<{
    bytes: Uint8Array;
    url: string;
    name: string;
  } | null>(null);
  const [preview, setPreview] = useState<Grid | null>(null);
  const [status, setStatus] = useState("画像を選択してください。");

  useEffect(() => {
    return () => {
      if (source) {
        URL.revokeObjectURL(source.url);
      }
    };
  }, [source]);

  useEffect(() => {
    if (!source) {
      setPreview(null);
      return;
    }

    let active = true;
    const timer = window.setTimeout(() => {
      setStatus("プレビューを生成中...");
      void (async () => {
        try {
          const next = await imageToGrid(new Uint8Array(source.bytes), params);
          if (active) {
            setPreview(next);
            setStatus("スライダーで調整して、良ければ適用してください。");
          }
        } catch (error) {
          if (active) {
            setPreview(null);
            setStatus(String(error));
          }
        }
      })();
    }, 180);

    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [params, source]);

  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <section className="modal-card image-modal" onClick={(event) => event.stopPropagation()}>
        <header className="modal-header">
          <div>
            <p className="eyebrow">Image Convert</p>
            <h2>画像からグリッドを作成</h2>
          </div>
          <button type="button" className="btn btn-ghost" onClick={onClose}>
            閉じる ×
          </button>
        </header>

        <div className="image-modal-body">
          <div className="image-preview-stack">
            <label className="file-button inline-file-button btn btn-subtle">
              🖼 画像を選択
              <input
                type="file"
                accept="image/*"
                onChange={(event) => {
                  const file = event.target.files?.[0];
                  if (!file) {
                    return;
                  }
                  if (source) {
                    URL.revokeObjectURL(source.url);
                  }
                  void file.arrayBuffer().then((buffer) => {
                    setSource({
                      bytes: new Uint8Array(buffer),
                      url: URL.createObjectURL(file),
                      name: file.name,
                    });
                  });
                }}
              />
            </label>
            <p className="modal-status">{source?.name ?? "画像未選択"}</p>
            <div className="preview-panels">
              <div className="preview-panel">
                <h3>Original</h3>
                {source ? <img src={source.url} alt={source.name} /> : <p>画像を選択してください。</p>}
              </div>
              <div className="preview-panel">
                <h3>Preview</h3>
                {preview ? (
                  <StaticGridPreview grid={preview} />
                ) : (
                  <p>{status}</p>
                )}
              </div>
            </div>
          </div>

          <div className="slider-panel">
            <SliderField
              label="Blur"
              min={0}
              max={5}
              step={0.1}
              value={params.smooth_strength}
              onChange={(value) => setParams((current) => ({ ...current, smooth_strength: value }))}
            />
            <SliderField
              label="Threshold"
              min={0}
              max={255}
              step={1}
              value={params.threshold}
              onChange={(value) => setParams((current) => ({ ...current, threshold: value }))}
            />
            <SliderField
              label="Edge"
              min={0}
              max={1}
              step={0.05}
              value={params.edge_strength}
              onChange={(value) => setParams((current) => ({ ...current, edge_strength: value }))}
            />
            <SliderField
              label="Noise"
              min={0}
              max={20}
              step={1}
              value={params.noise_removal}
              onChange={(value) => setParams((current) => ({ ...current, noise_removal: value }))}
            />
            <SliderField
              label="Width"
              min={5}
              max={50}
              step={1}
              value={params.grid_width}
              onChange={(value) => setParams((current) => ({ ...current, grid_width: value }))}
            />
            <SliderField
              label="Height"
              min={5}
              max={50}
              step={1}
              value={params.grid_height}
              onChange={(value) => setParams((current) => ({ ...current, grid_height: value }))}
            />
          </div>
        </div>

        <footer className="modal-footer">
          <p className="modal-status">{status}</p>
          <div className="toolbar-group">
            <button type="button" className="btn btn-ghost" onClick={onClose}>
              キャンセル
            </button>
            <button type="button" className="btn btn-primary" onClick={() => preview && onApply(preview)} disabled={!preview}>
              適用
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}

function SliderField({
  label,
  min,
  max,
  step,
  value,
  onChange,
}: {
  label: string;
  min: number;
  max: number;
  step: number;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="slider-field">
      <span>
        {label}
        <strong>{value}</strong>
      </span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function StaticGridPreview({ grid }: { grid: Grid }) {
  const columns = grid[0]?.length ?? 0;

  return (
    <div
      className="static-grid-preview"
      style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }}
    >
      {grid.flatMap((row, rowIndex) =>
        row.map((cell, colIndex) => (
          <span
            key={`${rowIndex}-${colIndex}`}
            className={`static-grid-cell ${cell ? "filled" : ""}`}
          />
        )),
      )}
    </div>
  );
}

function PlayPage({ id }: { id: string }) {
  const [solution, setSolution] = useState<Grid | null>(null);
  const [puzzle, setPuzzle] = useState<Puzzle | null>(null);
  const [progress, setProgress] = useState<Grid | null>(null);
  const [marks, setMarks] = useState<boolean[][] | null>(null);
  const [message, setMessage] = useState("問題を読み込み中...");

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const solved = await idToGrid(id);
        const nextPuzzle = await gridToPuzzle(solved);
        const { width, height } = puzzleDimensions(nextPuzzle);
        if (!cancelled) {
          setSolution(solved);
          setPuzzle(nextPuzzle);
          setProgress(createGrid(width, height));
          setMarks(createGrid(width, height));
          setMessage("右クリックで × を置けます。");
        }
      } catch (error) {
        if (!cancelled) {
          setMessage(String(error));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id]);

  useEffect(() => {
    if (!solution || !progress) {
      return;
    }
    if (equalGrid(solution, progress)) {
      setMarks(progress.map((row) => row.map((cell) => !cell)));
      setMessage("完成です。おめでとう。");
    }
  }, [progress, solution]);

  if (!puzzle || !progress || !marks) {
    return <div className="screen-state">{message}</div>;
  }

  return (
    <div className="app-shell play-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">Solve from shared URL</p>
          <h1>Nono<span>Maker</span> Play</h1>
        </div>
        <a className="ghost-link" href="/maker">
          ← maker に戻る
        </a>
      </header>
      <section className="play-layout">
        <PuzzleBoard
          puzzle={puzzle}
          progress={progress}
          marks={marks}
          onProgressChange={setProgress}
          onMarksChange={setMarks}
        />
        <div className="card">
          <h2>Play</h2>
          <p>{message}</p>
          <button type="button" className="btn btn-subtle" onClick={() => navigator.clipboard.writeText(window.location.href)}>
            📋 URL をコピー
          </button>
        </div>
      </section>
    </div>
  );
}

function clampSize(value: string) {
  const parsed = Number(value);
  if (Number.isNaN(parsed)) {
    return 5;
  }
  return Math.min(50, Math.max(5, Math.round(parsed)));
}

function isGrid(value: unknown): value is Grid {
  return (
    Array.isArray(value) &&
    value.every(
      (row) => Array.isArray(row) && row.every((cell) => typeof cell === "boolean"),
    )
  );
}

function isPuzzle(value: unknown): value is Puzzle {
  if (!value || typeof value !== "object") {
    return false;
  }
  const maybe = value as Record<string, unknown>;
  return ["row_clues", "col_clues"].every(
    (key) =>
      Array.isArray(maybe[key]) &&
      (maybe[key] as unknown[]).every(
        (row) => Array.isArray(row) && row.every((cell) => typeof cell === "number"),
      ),
  );
}

function downloadBlob(name: string, data: BlobPart, type: string) {
  const blob = new Blob([data], { type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  anchor.click();
  URL.revokeObjectURL(url);
}

function renderGridSvg(grid: Grid) {
  const size = 24;
  const width = grid[0].length * size;
  const height = grid.length * size;
  const cells = grid
    .flatMap((row, rowIndex) =>
      row.map((cell, colIndex) =>
        `<rect x="${colIndex * size}" y="${rowIndex * size}" width="${size}" height="${size}" fill="${cell ? "#1f2937" : "#fff7ed"}" stroke="#d6c6b8" />`,
      ),
    )
    .join("");

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">${cells}</svg>`;
}

async function renderGridPng(grid: Grid) {
  const size = 24;
  const canvas = document.createElement("canvas");
  canvas.width = grid[0].length * size;
  canvas.height = grid.length * size;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("canvas context を取得できません。");
  }

  context.fillStyle = "#fff7ed";
  context.fillRect(0, 0, canvas.width, canvas.height);

  for (let row = 0; row < grid.length; row += 1) {
    for (let col = 0; col < grid[0].length; col += 1) {
      context.fillStyle = grid[row][col] ? "#1f2937" : "#fff7ed";
      context.fillRect(col * size, row * size, size, size);
      context.strokeStyle = "#d6c6b8";
      context.strokeRect(col * size, row * size, size, size);
    }
  }

  return new Promise<Blob>((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) {
        resolve(blob);
      } else {
        reject(new Error("PNG 変換に失敗しました。"));
      }
    }, "image/png");
  });
}
