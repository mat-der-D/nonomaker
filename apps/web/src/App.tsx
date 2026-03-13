import { useEffect, useMemo, useRef, useState, type CSSProperties, type PointerEvent as ReactPointerEvent } from "react";
import { FaFacebook, FaLine } from "react-icons/fa";
import { FaBluesky, FaXTwitter } from "react-icons/fa6";
import { SiMastodon, SiMisskey } from "react-icons/si";
import { EditorGrid, type EditorTool } from "./components/EditorGrid";
import { PuzzleBoard, type PlayCell } from "./components/PuzzleBoard";
import { useWasm } from "./hooks/useWasm";
import { createGrid, equalGrid, maxClueDepth, puzzleDimensions } from "./utils/grid";
import {
  gridToId,
  gridToPuzzle,
  idToGrid,
  imageToGrid,
  solveComplete,
} from "./wasm/api";
import { terminateWorker } from "./wasm/workerClient";
import type { Grid, ImageToGridParams, Puzzle, Solution } from "./wasm/types";

type Route =
  | { kind: "maker" }
  | { kind: "play"; id: string };

interface AnalysisState {
  solution: Solution | null;
  message: string | null;
}

interface CheckDialogState {
  status: "idle" | "running" | "done" | "error" | "cancelled";
  message: string;
  solution: Solution | null;
}

type ExportFormat = "puzzle-png" | "puzzle-solution-png" | "puzzle-json" | "solution-json";

interface ExportDialogState {
  open: boolean;
  selected: ExportFormat;
}

interface ShareDialogState {
  open: boolean;
  url: string;
}

const makerGuideSeenKey = "nonomaker-maker-guide-seen";
const makerBoardStateKey = "nonomaker-maker-board-v1";
const playBoardStatePrefix = "nonomaker-play-board-v1:";

const defaultImageParams: ImageToGridParams = {
  smooth_strength: 1,
  threshold: 128,
  edge_strength: 0.3,
  noise_removal: 0,
  grid_width: 20,
  grid_height: 20,
};

const exportOptions: Array<{
  id: ExportFormat;
  label: string;
  description: string;
}> = [
  { id: "puzzle-png", label: "問題 PNG", description: "プレイヤー向けの問題盤面を PNG 画像で保存します。" },
  { id: "puzzle-solution-png", label: "問題 + 解答 PNG", description: "問題盤面と解答を載せた PNG 画像を保存します。" },
  { id: "puzzle-json", label: "問題 JSON", description: "プレイヤー向けの問題データを JSON で保存します。" },
  { id: "solution-json", label: "解答 JSON", description: "完成した盤面データを JSON で保存します。" },
];

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
  const [grid, setGrid] = useState<Grid>(() => loadMakerGridFromStorage() ?? createGrid(20, 20));
  const [history, setHistory] = useState<Grid[]>([]);
  const [future, setFuture] = useState<Grid[]>([]);
  const [tool, setTool] = useState<EditorTool>("draw");
  const [canvasScale, setCanvasScale] = useState("100");
  const [size, setSize] = useState({ width: 20, height: 20 });
  const [sizeDraft, setSizeDraft] = useState({ width: "20", height: "20" });
  const [analysis, setAnalysis] = useState<AnalysisState>({
    solution: null,
    message: null,
  });
  const [busy, setBusy] = useState<string | null>(null);
  const [guideOpen, setGuideOpen] = useState(false);
  const [imageModalOpen, setImageModalOpen] = useState(false);
  const [exportDialog, setExportDialog] = useState<ExportDialogState>({
    open: false,
    selected: "puzzle-png",
  });
  const [shareDialog, setShareDialog] = useState<ShareDialogState>({
    open: false,
    url: "",
  });
  const [selectedSolutionIndex, setSelectedSolutionIndex] = useState(0);
  const [checkDialog, setCheckDialog] = useState<CheckDialogState>({
    status: "idle",
    message: "",
    solution: null,
  });
  const checkRunRef = useRef(0);
  const importRunRef = useRef(0);
  const canvasViewportRef = useRef<HTMLDivElement | null>(null);
  const panStateRef = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    scrollLeft: number;
    scrollTop: number;
  } | null>(null);

  useEffect(() => {
    setSize({ width: grid[0]?.length ?? 0, height: grid.length });
    setSizeDraft({
      width: String(grid[0]?.length ?? 0),
      height: String(grid.length),
    });
  }, [grid]);

  useEffect(() => {
    setSelectedSolutionIndex(0);
  }, [checkDialog.solution]);

  useEffect(() => {
    if (window.localStorage.getItem(makerGuideSeenKey) === "1") {
      return;
    }
    setGuideOpen(true);
    window.localStorage.setItem(makerGuideSeenKey, "1");
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(makerBoardStateKey, JSON.stringify(grid));
    } catch {
      // Ignore storage failures so editing stays available.
    }
  }, [grid]);

  function commit(next: Grid) {
    setHistory((current) => [...current, grid]);
    setFuture([]);
    setGrid(next);
  }

  function resizeGrid() {
    const nextSize = {
      width: clampSize(sizeDraft.width),
      height: clampSize(sizeDraft.height),
    };
    setSize(nextSize);
    setSizeDraft({
      width: String(nextSize.width),
      height: String(nextSize.height),
    });

    const next = createGrid(nextSize.width, nextSize.height);
    for (let row = 0; row < Math.min(grid.length, next.length); row += 1) {
      for (let col = 0; col < Math.min(grid[0].length, next[0].length); col += 1) {
        next[row][col] = grid[row][col];
      }
    }
    commit(next);
  }

  async function runCheck() {
    checkRunRef.current += 1;
    const runId = checkRunRef.current;
    setBusy("checking");
    setCheckDialog({
      status: "running",
      message: "解答を解析しています。しばらくお待ちください。",
      solution: null,
    });
    try {
      const puzzle = await gridToPuzzle(grid);
      const solution = await solveComplete(puzzle, "fp2-backtracking");
      if (runId !== checkRunRef.current) {
        return;
      }
      const message =
        solution.status === "unique"
          ? "一意解です。共有とエクスポートを有効化しました。"
          : solution.status === "multiple"
            ? "複数解です。"
            : "解なしです。";
      setAnalysis({ solution, message });
      setCheckDialog({
        status: "done",
        message,
        solution,
      });
    } catch (error) {
      if (runId !== checkRunRef.current) {
        return;
      }
      setAnalysis({ solution: null, message: String(error) });
      setCheckDialog({
        status: "error",
        message: String(error),
        solution: null,
      });
    } finally {
      if (runId === checkRunRef.current) {
        setBusy(null);
      }
    }
  }

  function cancelCheck() {
    checkRunRef.current += 1;
    terminateWorker();
    setBusy(null);
    setCheckDialog({
      status: "cancelled",
      message: "解答チェックを中止しました。",
      solution: null,
    });
  }

  async function generateShare() {
    setBusy("share");
    try {
      const id = await gridToId(grid);
      const url = `${window.location.origin}/play/${id}`;
      setShareDialog({ open: true, url });
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

  async function exportArtifact(format: ExportFormat) {
    try {
      if (format === "puzzle-png") {
        const puzzle = await gridToPuzzle(grid);
        downloadBlob("puzzle.png", await renderPuzzlePng(puzzle), "image/png");
        setAnalysis((current) => ({ ...current, message: "問題 PNG を出力しました。" }));
        return;
      }

      if (format === "puzzle-solution-png") {
        const puzzle = await gridToPuzzle(grid);
        downloadBlob("puzzle-with-solution.png", await renderPuzzlePng(puzzle, grid), "image/png");
        setAnalysis((current) => ({ ...current, message: "問題 + 解答 PNG を出力しました。" }));
        return;
      }

      if (format === "puzzle-json") {
        const puzzle = await gridToPuzzle(grid);
        downloadBlob("puzzle.json", JSON.stringify(puzzle), "application/json");
        setAnalysis((current) => ({ ...current, message: "問題 JSON を出力しました。" }));
        return;
      }

      if (format === "solution-json") {
        downloadBlob("solution.json", JSON.stringify(grid), "application/json");
        setAnalysis((current) => ({ ...current, message: "解答 JSON を出力しました。" }));
        return;
      }
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
      importRunRef.current += 1;
      const runId = importRunRef.current;
      setBusy("import");
      try {
        const solution = await solveComplete(parsed, "fp2-backtracking");
        if (runId !== importRunRef.current) {
          return;
        }
        if (solution.status !== "unique") {
          throw new Error("読み込んだ問題は一意解ではありません。");
        }
        commit(solution.grids[0]);
        setAnalysis((current) => ({ ...current, message: "問題JSONを読み込みました。" }));
      } catch (error) {
        if (runId !== importRunRef.current) {
          return;
        }
        setAnalysis((current) => ({ ...current, message: String(error) }));
      } finally {
        if (runId === importRunRef.current) {
          setBusy(null);
        }
      }
      return;
    }

    throw new Error("サポートしていない JSON 形式です。");
  }

  function cancelImport() {
    importRunRef.current += 1;
    terminateWorker();
    setBusy(null);
    setAnalysis((current) => ({ ...current, message: "JSON 読み込みを中止しました。" }));
  }

  const exportAllowed = analysis.solution?.status === "unique";
  const canvasScaleValue = clampCanvasScale(canvasScale);
  const toolItems: Array<{ id: EditorTool; icon: string; label: string; hint: string }> = [
    { id: "draw", icon: "■", label: "ペン", hint: "塗る" },
    { id: "erase", icon: "□", label: "消しゴム", hint: "消す" },
    { id: "invert", icon: "◪", label: "反転", hint: "白黒反転" },
    { id: "line", icon: "／", label: "直線", hint: "ドラッグで線" },
    { id: "rect", icon: "▦", label: "矩形", hint: "ドラッグで面" },
    { id: "fill", icon: "▨", label: "バケツ", hint: "連結塗り" },
    { id: "zoom", icon: "⊕", label: "拡大", hint: "拡大・縮小" },
    { id: "pan", icon: "✥", label: "移動", hint: "ドラッグで移動" },
  ];
  const activeTool = toolItems.find((item) => item.id === tool) ?? toolItems[0];

  function updateCanvasScale(next: number) {
    setCanvasScale(String(Math.min(300, Math.max(50, next))));
  }

  function handleCanvasPointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    const viewport = canvasViewportRef.current;
    if (!viewport) {
      return;
    }

    if (tool === "pan") {
      panStateRef.current = {
        pointerId: event.pointerId,
        startX: event.clientX,
        startY: event.clientY,
        scrollLeft: viewport.scrollLeft,
        scrollTop: viewport.scrollTop,
      };
      event.currentTarget.setPointerCapture(event.pointerId);
      event.preventDefault();
      return;
    }

    if (tool === "zoom") {
      const delta = event.button === 2 ? -10 : 10;
      updateCanvasScale(canvasScaleValue + delta);
      event.preventDefault();
    }
  }

  function handleCanvasPointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    const viewport = canvasViewportRef.current;
    const panState = panStateRef.current;
    if (tool !== "pan" || !viewport || !panState || panState.pointerId !== event.pointerId) {
      return;
    }

    viewport.scrollLeft = panState.scrollLeft - (event.clientX - panState.startX);
    viewport.scrollTop = panState.scrollTop - (event.clientY - panState.startY);
  }

  function handleCanvasPointerUp(event: ReactPointerEvent<HTMLDivElement>) {
    if (panStateRef.current?.pointerId !== event.pointerId) {
      return;
    }

    panStateRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">Nonogram puzzle maker</p>
          <h1>Nono<span>Maker</span></h1>
        </div>
        <button type="button" className="btn btn-primary" onClick={() => setGuideOpen(true)}>
          使い方
        </button>
      </header>

      <section className="toolbar">
        <div className="toolbar-section">
          <p className="toolbar-title">Canvas</p>
          <div className="toolbar-group canvas-toolbar-group">
            <div className="size-fields">
              <label className="inline-number-field">
                <span>縦</span>
                <input
                  type="number"
                  min={5}
                  max={50}
                  value={sizeDraft.height}
                  onChange={(event) => setSizeDraft((current) => ({ ...current, height: event.target.value }))}
                  onBlur={() =>
                    setSizeDraft((current) => ({ ...current, height: String(clampSize(current.height)) }))
                  }
                />
              </label>
              <span className="toolbar-x">×</span>
              <label className="inline-number-field">
                <span>横</span>
                <input
                  type="number"
                  min={5}
                  max={50}
                  value={sizeDraft.width}
                  onChange={(event) => setSizeDraft((current) => ({ ...current, width: event.target.value }))}
                  onBlur={() =>
                    setSizeDraft((current) => ({ ...current, width: String(clampSize(current.width)) }))
                  }
                />
              </label>
            </div>
            <button type="button" className="btn btn-subtle" onClick={resizeGrid}>
              サイズ適用
            </button>
            <label className="inline-number-field canvas-scale-field">
              <span>表示</span>
              <input
                type="number"
                min={50}
                max={300}
                step={10}
                value={canvasScale}
                onChange={(event) => setCanvasScale(event.target.value)}
                onBlur={() => setCanvasScale(String(canvasScaleValue))}
              />
              <strong className="inline-unit">%</strong>
            </label>
            <button type="button" className="btn btn-danger" onClick={() => window.confirm("盤面をクリアしますか？") && commit(createGrid(grid[0].length, grid.length))}>
              クリア
            </button>
          </div>
        </div>

        <div className="toolbar-section">
          <p className="toolbar-title">Import</p>
          <div className="toolbar-group">
            <button
              type="button"
              className="btn btn-subtle"
              onClick={() => setImageModalOpen(true)}
            >
              画像から盤面作成
            </button>
            <label className="file-button btn btn-subtle">
              JSON 読み込み
              <input
                type="file"
                accept=".json,application/json"
                title=""
                onChange={(event) => {
                  const file = event.target.files?.[0];
                  event.target.value = "";
                  if (file) {
                    void importJson(file);
                  }
                }}
              />
            </label>
          </div>
        </div>

        <div className="toolbar-section toolbar-section-wide">
          <p className="toolbar-title">Export</p>
          <div className="toolbar-group">
            <span
              className="tooltip-trigger"
              title={!exportAllowed ? "解答チェックで一意解を確認すると使えます。" : undefined}
            >
              <button
                type="button"
                className="btn btn-subtle"
                onClick={() => setExportDialog((current) => ({ ...current, open: true }))}
                disabled={!exportAllowed}
              >
                ファイル出力
              </button>
            </span>
            <span
              className="tooltip-trigger"
              title={!exportAllowed ? "解答チェックで一意解を確認すると使えます。" : undefined}
            >
              <button type="button" className="btn btn-primary" onClick={() => void generateShare()} disabled={!exportAllowed}>
                共有
              </button>
            </span>
          </div>
        </div>
      </section>

      <section className="content">
        <div className="editor-panel maker-editor-panel">
          <div className="maker-workbench">
            <aside className="maker-sidebar">
              <section className="tool-actions tool-actions-plain">
                <div className="tool-actions-inline">
                  <button type="button" className="btn btn-ghost tool-action-btn" onClick={() => history.length && (setFuture((current) => [grid, ...current]), setGrid(history[history.length - 1]), setHistory((current) => current.slice(0, -1)))} disabled={history.length === 0}>
                    Undo
                  </button>
                  <button type="button" className="btn btn-ghost tool-action-btn" onClick={() => future.length && (setHistory((current) => [...current, grid]), setGrid(future[0]), setFuture((current) => current.slice(1)))} disabled={future.length === 0}>
                    Redo
                  </button>
                </div>
              </section>

              <section className="toolbox card">
                <div className="toolbox-header">
                  <h2>Tools</h2>
                  <p>{activeTool.label}{tool === "fill" || tool === "zoom" || tool === "pan" ? ` / ${activeTool.hint}` : ""}</p>
                </div>
                <div className="toolbox-grid" role="toolbar" aria-label="drawing tools">
                  {toolItems.map((item) => (
                    <button
                      key={item.id}
                      type="button"
                      className={`tool-button ${item.id === tool ? "active" : ""}`}
                      onClick={() => setTool(item.id)}
                      aria-pressed={item.id === tool}
                      title={`${item.label}: ${item.hint}`}
                    >
                      <span className="tool-button-icon" aria-hidden="true">{item.icon}</span>
                    </button>
                  ))}
                </div>
              </section>
            </aside>

            <div
              ref={canvasViewportRef}
              className={`maker-canvas-viewport ${tool === "pan" ? "pan-active" : ""} ${tool === "zoom" ? "zoom-active" : ""}`}
              onContextMenu={(event) => {
                if (tool === "pan" || tool === "zoom") {
                  event.preventDefault();
                }
              }}
              onPointerDown={handleCanvasPointerDown}
              onPointerMove={handleCanvasPointerMove}
              onPointerUp={handleCanvasPointerUp}
              onPointerCancel={handleCanvasPointerUp}
            >
              <EditorGrid
                grid={grid}
                tool={tool}
                scalePercent={canvasScaleValue}
                onChange={(next) => {
                  if (!equalGrid(next, grid)) {
                    commit(next);
                  }
                }}
              />
            </div>
          </div>
        </div>

        <aside className="side-panel">
          <section className="card">
            <h2>Answer Check</h2>
            <div className="tool-actions-row side-panel-actions">
              {checkDialog.status === "running" ? (
                <button type="button" className="btn btn-ghost tool-action-btn" onClick={cancelCheck}>
                  中止
                </button>
              ) : (
                <button type="button" className="btn btn-primary tool-action-btn" onClick={() => void runCheck()} disabled={busy !== null}>
                  解答チェック
                </button>
              )}
            </div>
            <div className={`check-status-panel side-check-status ${checkDialog.solution?.status === "multiple" ? "compact" : ""}`}>
              <div className={`check-status-icon ${checkDialog.status} ${checkDialog.solution?.status === "multiple" ? "negative" : ""}`}>
                {checkDialog.status === "running"
                  ? "◌"
                  : checkDialog.status === "cancelled"
                    ? "■"
                    : checkDialog.status === "error" || checkDialog.solution?.status === "multiple"
                      ? "×"
                      : checkDialog.solution?.status === "unique"
                        ? "✓"
                        : "…"}
              </div>
              <div className="side-check-status-copy">
                <p className="check-status-label">
                  {checkDialog.status === "running"
                    ? "解析中"
                    : checkDialog.status === "done"
                      ? "解析完了"
                      : checkDialog.status === "cancelled"
                        ? "中止"
                        : checkDialog.status === "error"
                          ? "エラー"
                          : "未実行"}
                </p>
                <p className="modal-status">{checkDialog.message || "まだ解析していません。"}</p>
              </div>
            </div>
            {checkDialog.solution?.grids.length ? (
              <div className="side-check-solution-stack">
                <div className="preview-panel check-solution-panel side-check-solution-panel">
                  {checkDialog.solution.status === "multiple" ? (
                    <div className="solution-tabs segmented-solution-tabs" role="tablist" aria-label="solutions">
                      {checkDialog.solution.grids.map((_, index) => (
                        <button
                          key={index}
                          type="button"
                          role="tab"
                          className={`solution-tab ${index === selectedSolutionIndex ? "active" : ""}`}
                          aria-selected={index === selectedSolutionIndex}
                          onClick={() => setSelectedSolutionIndex(index)}
                        >
                          Solution {index + 1}
                        </button>
                      ))}
                    </div>
                  ) : (
                    <h3>Solution</h3>
                  )}
                  <div className="preview-frame side-check-preview-frame">
                    <StaticGridPreview
                      grid={checkDialog.solution.grids[Math.min(selectedSolutionIndex, checkDialog.solution.grids.length - 1)]}
                      fitSquare
                    />
                  </div>
                </div>
              </div>
            ) : null}
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
              message: "画像から作成した盤面を適用しました。",
            }));
            setImageModalOpen(false);
          }}
          onClose={() => setImageModalOpen(false)}
        />
      )}
      {exportDialog.open && (
        <ExportFormatModal
          selected={exportDialog.selected}
          options={exportOptions}
          onSelect={(selected) => setExportDialog({ open: true, selected })}
          onClose={() => setExportDialog((current) => ({ ...current, open: false }))}
          onConfirm={() => {
            const { selected } = exportDialog;
            setExportDialog((current) => ({ ...current, open: false }));
            void exportArtifact(selected);
          }}
        />
      )}

      {shareDialog.open && (
        <ShareModal
          url={shareDialog.url}
          onClose={() => setShareDialog((current) => ({ ...current, open: false }))}
        />
      )}

      {guideOpen && <MakerGuideModal onClose={() => setGuideOpen(false)} />}

      {busy === "import" && (
        <ProgressModal
          title="JSON 読み込み"
          message="問題を解析して盤面を復元しています。"
          onCancel={cancelImport}
        />
      )}
    </div>
  );
}

function ExportFormatModal({
  selected,
  options,
  onSelect,
  onClose,
  onConfirm,
}: {
  selected: ExportFormat;
  options: typeof exportOptions;
  onSelect: (format: ExportFormat) => void;
  onClose: () => void;
  onConfirm: () => void;
}) {
  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <section className="modal-card export-modal" onClick={(event) => event.stopPropagation()}>
        <header className="modal-header">
          <div>
            <p className="eyebrow">Export</p>
            <h2>保存形式を選択</h2>
          </div>
        </header>

        <div className="export-modal-body">
          {options.map((option) => (
            <label key={option.id} className={`export-option ${option.id === selected ? "active" : ""}`}>
              <input
                type="radio"
                name="export-format"
                checked={option.id === selected}
                onChange={() => onSelect(option.id)}
              />
              <div>
                <strong>{option.label}</strong>
                <p>{option.description}</p>
              </div>
            </label>
          ))}
        </div>

        <footer className="modal-footer">
          <p className="modal-status">1 つだけ選んで保存します。</p>
          <div className="toolbar-group">
            <button type="button" className="btn btn-ghost" onClick={onClose}>
              キャンセル
            </button>
            <button type="button" className="btn btn-primary" onClick={onConfirm}>
              保存
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}

function ShareModal({
  url,
  onClose,
}: {
  url: string;
  onClose: () => void;
}) {
  const shareText = "お絵かきロジックの問題を作りました。ぜひ遊んでみてください。";
  const encodedUrl = encodeURIComponent(url);
  const encodedText = encodeURIComponent(shareText);
  const shareTargets = [
    {
      icon: <FaXTwitter className="share-icon" aria-hidden="true" />,
      brandClass: "x",
      label: "X",
      href: `https://twitter.com/intent/tweet?text=${encodedText}&url=${encodedUrl}`,
    },
    {
      icon: <FaFacebook className="share-icon" aria-hidden="true" />,
      brandClass: "facebook",
      label: "Facebook",
      href: `https://www.facebook.com/sharer/sharer.php?u=${encodedUrl}`,
    },
    {
      icon: <FaLine className="share-icon" aria-hidden="true" />,
      brandClass: "line",
      label: "LINE",
      href: `https://social-plugins.line.me/lineit/share?url=${encodedUrl}`,
    },
    {
      icon: <FaBluesky className="share-icon" aria-hidden="true" />,
      brandClass: "bluesky",
      label: "Bluesky",
      href: `https://bsky.app/intent/compose?text=${encodedText}%20${encodedUrl}`,
    },
    {
      icon: <SiMastodon className="share-icon" aria-hidden="true" />,
      brandClass: "mastodon",
      label: "Mastodon",
      href: `https://mastodonshare.com/?text=${encodedText}%20${encodedUrl}`,
    },
    {
      icon: <SiMisskey className="share-icon" aria-hidden="true" />,
      brandClass: "misskey",
      label: "Misskey",
      href: `https://misskey-hub.net/share/?text=${encodedText}%20${encodedUrl}`,
    },
  ];

  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <section className="modal-card share-modal" onClick={(event) => event.stopPropagation()}>
        <header className="modal-header">
          <div>
            <p className="eyebrow">Share</p>
            <h2>共有URL</h2>
          </div>
        </header>

        <div className="share-modal-body">
          <input className="share-input" readOnly value={url} />
          <div className="share-targets">
            {shareTargets
              .filter((target) => target.href !== null || (typeof navigator !== "undefined" && "share" in navigator))
              .map((target) => (
              <button
                key={target.label}
                type="button"
                className={`btn btn-subtle share-target-button ${target.brandClass ? `share-target-${target.brandClass}` : ""}`}
                aria-label={target.label}
                title={target.label}
                onClick={() => {
                  if (target.href) {
                    window.open(target.href, "_blank", "noopener,noreferrer");
                  }
                }}
              >
                {target.icon}
              </button>
              ))}
          </div>
        </div>

        <footer className="modal-footer">
          <p className="modal-status">クリップボードにコピー済みです。</p>
          <div className="toolbar-group">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => window.open(url, "_blank", "noopener,noreferrer")}
            >
              テストプレイ
            </button>
            <button type="button" className="btn btn-primary" onClick={onClose}>
              閉じる
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}

function ProgressModal({
  title,
  message,
  onCancel,
}: {
  title: string;
  message: string;
  onCancel: () => void;
}) {
  return (
    <div className="modal-backdrop" role="presentation">
      <section className="modal-card progress-modal" onClick={(event) => event.stopPropagation()}>
        <header className="modal-header">
          <div>
            <p className="eyebrow">Processing</p>
            <h2>{title}</h2>
          </div>
        </header>

        <div className="progress-modal-body">
          <div className="check-status-panel side-check-status progress-status-panel">
            <div className="check-status-icon running">◌</div>
            <div className="side-check-status-copy">
              <p className="check-status-label">処理中</p>
              <p className="modal-status">{message}</p>
            </div>
          </div>
        </div>

        <footer className="modal-footer">
          <p className="modal-status">時間がかかる場合は中止できます。</p>
          <div className="toolbar-group">
            <button type="button" className="btn btn-ghost" onClick={onCancel}>
              中止
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}

function MakerGuideModal({ onClose }: { onClose: () => void }) {
  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <section className="modal-card guide-modal" onClick={(event) => event.stopPropagation()}>
        <header className="modal-header">
          <div>
            <p className="eyebrow">Guide</p>
            <h2>NonoMaker の使い方</h2>
          </div>
        </header>

        <div className="guide-modal-body">
          <section className="guide-section">
            <h3>1. 盤面を作る</h3>
            <p>左のツールで塗る、消す、反転、直線、矩形、塗りつぶしが使えます。<strong>サイズ適用</strong> で盤面サイズを変え、<strong>画像から盤面作成</strong> で画像から下書きを作ることもできます。</p>
          </section>
          <section className="guide-section">
            <h3>2. 解答を確認する</h3>
            <p>右の <strong>解答チェック</strong> を押すと、一意解・複数解・解なしを確認できます。複数解の場合は、タブで候補の解を切り替えて確認できます。</p>
          </section>
          <section className="guide-section">
            <h3>3. 保存と共有</h3>
            <p><strong>ファイル出力</strong> では問題 PNG、問題 + 解答 PNG、問題 JSON、解答 JSON を選んで保存できます。<strong>共有</strong> はプレイ用 URL をコピーし、そのままテストプレイできます。</p>
          </section>
          <section className="guide-section">
            <h3>4. JSON を読み込む</h3>
            <p><strong>解答 JSON</strong> はすぐに編集用盤面として開きます。<strong>問題 JSON</strong> は解いてから盤面を復元するため、時間がかかる場合があります。読み込み中は中止できます。</p>
          </section>
        </div>

        <footer className="modal-footer">
          <p className="modal-status">必要になったら右上の <strong>使い方</strong> からいつでも開けます。</p>
          <div className="toolbar-group">
            <button type="button" className="btn btn-primary" onClick={onClose}>
              閉じる
            </button>
          </div>
        </footer>
      </section>
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
  const [sizeDraft, setSizeDraft] = useState({
    width: String(initialParams.grid_width),
    height: String(initialParams.grid_height),
  });
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
    setSizeDraft({
      width: String(params.grid_width),
      height: String(params.grid_height),
    });
  }, [params.grid_width, params.grid_height]);

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

  function commitImageSize(nextWidth: string, nextHeight: string) {
    const width = clampSize(nextWidth);
    const height = clampSize(nextHeight);
    setParams((current) => ({
      ...current,
      grid_width: width,
      grid_height: height,
    }));
  }

  return (
    <div className="modal-backdrop" role="presentation" onClick={onClose}>
      <section className="modal-card image-modal" onClick={(event) => event.stopPropagation()}>
        <header className="modal-header">
          <div>
            <p className="eyebrow">Image To Board</p>
            <h2>画像から盤面作成</h2>
          </div>
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
                <div className="preview-frame">
                  {source ? (
                    <div
                      className="preview-image-surface"
                      role="img"
                      aria-label={source.name}
                      style={{ backgroundImage: `url("${source.url}")` }}
                    />
                  ) : <p>画像を選択してください。</p>}
                </div>
              </div>
              <div className="preview-panel">
                <h3>Preview</h3>
                <div className="preview-frame">
                  {preview ? (
                    <StaticGridPreview grid={preview} maxSide={296} />
                  ) : (
                    <p>{status}</p>
                  )}
                </div>
              </div>
            </div>
          </div>

          <div className="slider-panel">
            <div className="dimension-fields">
              <label className="number-field">
                <span>縦</span>
                <input
                  type="number"
                  min={5}
                  max={50}
                  step={1}
                  value={sizeDraft.height}
                  onChange={(event) =>
                    setSizeDraft((current) => ({
                      ...current,
                      height: event.target.value,
                    }))
                  }
                  onBlur={() => commitImageSize(sizeDraft.width, sizeDraft.height)}
                />
              </label>
              <label className="number-field">
                <span>横</span>
                <input
                  type="number"
                  min={5}
                  max={50}
                  step={1}
                  value={sizeDraft.width}
                  onChange={(event) =>
                    setSizeDraft((current) => ({
                      ...current,
                      width: event.target.value,
                    }))
                  }
                  onBlur={() => commitImageSize(sizeDraft.width, sizeDraft.height)}
                />
              </label>
            </div>
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

function StaticGridPreview({
  grid,
  maxSide,
  fitSquare = false,
}: {
  grid: Grid;
  maxSide?: number;
  fitSquare?: boolean;
}) {
  const columns = grid[0]?.length ?? 0;
  const rows = grid.length;
  const longest = Math.max(columns, rows, 1);
  const side = maxSide ?? 220;
  const width = `${(side * (columns / longest)).toFixed(2)}px`;
  const height = `${(side * (rows / longest)).toFixed(2)}px`;
  const fitWidth = `${((columns / longest) * 100).toFixed(4)}%`;
  const fitHeight = `${((rows / longest) * 100).toFixed(4)}%`;

  return (
    <div
      className={`static-grid-preview ${fitSquare ? "fit-square" : ""}`}
      style={{
        gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))`,
        "--preview-width": width,
        "--preview-height": height,
        "--preview-fit-width": fitWidth,
        "--preview-fit-height": fitHeight,
      } as CSSProperties}
    >
      {grid.flatMap((row, rowIndex) =>
        row.map((cell, colIndex) => (
          <span
            key={`${rowIndex}-${colIndex}`}
            className={[
              "static-grid-cell",
              cell ? "filled" : "",
              rowIndex > 0 && rowIndex % 5 === 0 ? "major-top" : "",
              colIndex > 0 && colIndex % 5 === 0 ? "major-left" : "",
            ]
              .filter(Boolean)
              .join(" ")}
          />
        )),
      )}
    </div>
  );
}

function PlayPage({ id }: { id: string }) {
  const [solutionGrid, setSolutionGrid] = useState<Grid | null>(null);
  const [puzzle, setPuzzle] = useState<Puzzle | null>(null);
  const [playCells, setPlayCells] = useState<PlayCell[][] | null>(null);
  const [statusMessage, setStatusMessage] = useState("問題を読み込み中...");
  const [playScale, setPlayScale] = useState("100");

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const decodedSolutionGrid = await idToGrid(id);
        const nextPuzzle = await gridToPuzzle(decodedSolutionGrid);
        const { width, height } = puzzleDimensions(nextPuzzle);
        const savedPlayCells = loadPlayGridFromStorage(id, width, height);
        if (!cancelled) {
          setSolutionGrid(decodedSolutionGrid);
          setPuzzle(nextPuzzle);
          setPlayCells(savedPlayCells ?? createPlayGrid(width, height));
          setStatusMessage("左クリックで入力、右クリックで反対の記号を置けます。");
        }
      } catch (error) {
        if (!cancelled) {
          setStatusMessage(String(error));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id]);

  useEffect(() => {
    if (!playCells) {
      return;
    }
    try {
      window.localStorage.setItem(playBoardStorageKey(id), JSON.stringify(playCells));
    } catch {
      // Ignore storage failures so gameplay stays available.
    }
  }, [id, playCells]);

  if (!puzzle || !playCells || !solutionGrid) {
    return <div className="screen-state">{statusMessage}</div>;
  }

  const playStats = computePlayStats(playCells, solutionGrid);
  const playScaleValue = clampCanvasScale(playScale);
  const boardCellSize = Math.max(10, Math.round((24 * playScaleValue) / 100));
  const progressRatio =
    playStats.targetFilled === 0 ? 100 : Math.round((playStats.correctFilled / playStats.targetFilled) * 100);

  function nudgePlayScale(delta: number) {
    setPlayScale(String(clampCanvasScale(String(playScaleValue + delta))));
  }

  function resetBoard() {
    const { width, height } = puzzleDimensions(puzzle);
    setPlayCells(createPlayGrid(width, height));
    setStatusMessage("盤面をリセットしました。");
  }

  function handlePlayCellsChange(nextPlayCells: PlayCell[][]) {
    setPlayCells(nextPlayCells);
    if (solvedBoard(nextPlayCells, solutionGrid)) {
      setStatusMessage("完成です。");
      return;
    }

    const nextPlayStats = computePlayStats(nextPlayCells, solutionGrid);
    if (nextPlayStats.wrongFilled > 0) {
      setStatusMessage(`誤って塗っているマスが ${nextPlayStats.wrongFilled} 個あります。`);
    } else {
      setStatusMessage("左クリックで入力、右クリックで反対の記号を置けます。");
    }
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
        <div className="play-main">
          <div className="editor-panel play-board-panel">
            <PuzzleBoard
              puzzle={puzzle}
              cells={playCells}
              cellSize={boardCellSize}
              onCellsChange={handlePlayCellsChange}
            />
          </div>
        </div>
        <div className="card play-sidebar">
          <h2>Play</h2>
          <p>{statusMessage}</p>
          <button
            type="button"
            className="btn btn-ghost play-reset-button"
            onClick={() => window.confirm("盤面をリセットしますか？") && resetBoard()}
          >
            リセット
          </button>
          <div className="play-stats">
            <div className="play-scale-control">
              <label className="number-field">
                <span>表示倍率</span>
                <input
                  type="number"
                  min={50}
                  max={300}
                  step={5}
                  value={playScale}
                  onChange={(event) => setPlayScale(event.target.value)}
                  onBlur={() => setPlayScale(String(playScaleValue))}
                />
              </label>
              <div className="play-scale-buttons">
                <button type="button" className="btn btn-ghost play-scale-btn" onClick={() => nudgePlayScale(5)}>
                  +
                </button>
                <button type="button" className="btn btn-ghost play-scale-btn" onClick={() => nudgePlayScale(-5)}>
                  -
                </button>
              </div>
            </div>
            <div className="play-stat">
              <span>進捗</span>
              <strong>{progressRatio}%</strong>
            </div>
            <div className="ratio-bar-track">
              <div className="ratio-bar-fill" style={{ width: `${progressRatio}%` }} />
            </div>
            <div className="play-stat">
              <span>正しく塗れたマス</span>
              <strong>
                {playStats.correctFilled} / {playStats.targetFilled}
              </strong>
            </div>
            <div className="play-stat">
              <span>誤って塗ったマス</span>
              <strong>{playStats.wrongFilled}</strong>
            </div>
            <div className="play-stat">
              <span>× を置いたマス</span>
              <strong>{playStats.crossed}</strong>
            </div>
          </div>
          <p className="play-hint">左クリックで塗り、右クリックで ×。同じ状態をもう一度入力すると Blank に戻ります。</p>
          <button type="button" className="btn btn-subtle" onClick={() => navigator.clipboard.writeText(window.location.href)}>
            📋 URL をコピー
          </button>
        </div>
      </section>
    </div>
  );
}

function createPlayGrid(width: number, height: number): PlayCell[][] {
  return Array.from({ length: height }, () => Array.from({ length: width }, () => "unknown"));
}

function playBoardStorageKey(id: string) {
  return `${playBoardStatePrefix}${id}`;
}

function computePlayStats(playCells: PlayCell[][], solutionGrid: Grid) {
  let targetFilled = 0;
  let correctFilled = 0;
  let wrongFilled = 0;
  let crossed = 0;

  for (let rowIndex = 0; rowIndex < playCells.length; rowIndex += 1) {
    for (let colIndex = 0; colIndex < playCells[0].length; colIndex += 1) {
      if (solutionGrid[rowIndex][colIndex]) {
        targetFilled += 1;
      }
      if (playCells[rowIndex][colIndex] === "filled" && solutionGrid[rowIndex][colIndex]) {
        correctFilled += 1;
      }
      if (playCells[rowIndex][colIndex] === "filled" && !solutionGrid[rowIndex][colIndex]) {
        wrongFilled += 1;
      }
      if (playCells[rowIndex][colIndex] === "crossed") {
        crossed += 1;
      }
    }
  }

  return { targetFilled, correctFilled, wrongFilled, crossed };
}

function solvedBoard(playCells: PlayCell[][], solutionGrid: Grid) {
  return playCells.every((playRow, rowIndex) =>
    playRow.every((playCell, colIndex) =>
      solutionGrid[rowIndex][colIndex] ? playCell === "filled" : playCell !== "filled",
    ),
  );
}

function clampSize(value: string) {
  const parsed = Number(value);
  if (Number.isNaN(parsed)) {
    return 5;
  }
  return Math.min(50, Math.max(5, Math.round(parsed)));
}

function clampCanvasScale(value: string) {
  const parsed = Number(value);
  if (Number.isNaN(parsed)) {
    return 100;
  }
  return Math.min(300, Math.max(50, Math.round(parsed)));
}

function isGrid(value: unknown): value is Grid {
  return (
    Array.isArray(value) &&
    value.every(
      (row) => Array.isArray(row) && row.every((cell) => typeof cell === "boolean"),
    )
  );
}

function loadMakerGridFromStorage(): Grid | null {
  try {
    const raw = window.localStorage.getItem(makerBoardStateKey);
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!isGrid(parsed)) {
      return null;
    }
    if (parsed.length === 0 || parsed[0]?.length === 0) {
      return null;
    }
    if (parsed.length > 50 || parsed[0].length > 50) {
      return null;
    }
    if (parsed.some((row) => row.length !== parsed[0].length)) {
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}

function loadPlayGridFromStorage(id: string, width: number, height: number): PlayCell[][] | null {
  try {
    const raw = window.localStorage.getItem(playBoardStorageKey(id));
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!isPlayGrid(parsed, width, height)) {
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}

function isPlayGrid(value: unknown, width: number, height: number): value is PlayCell[][] {
  const validCells: PlayCell[] = ["unknown", "filled", "crossed"];
  return (
    Array.isArray(value) &&
    value.length === height &&
    value.every(
      (row) =>
        Array.isArray(row) &&
        row.length === width &&
        row.every((cell) => typeof cell === "string" && validCells.includes(cell as PlayCell)),
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

async function renderPuzzlePng(puzzle: Puzzle, solutionGrid?: Grid) {
  const size = 24;
  const { maxRowClueSlots, maxColClueSlots } = maxClueDepth(puzzle);
  const rowClueAreaWidth = maxRowClueSlots * size;
  const colClueAreaHeight = maxColClueSlots * size;
  const boardWidth = puzzle.col_clues.length * size;
  const boardHeight = puzzle.row_clues.length * size;
  const canvas = document.createElement("canvas");
  canvas.width = rowClueAreaWidth + boardWidth;
  canvas.height = colClueAreaHeight + boardHeight;
  const context = canvas.getContext("2d");
  if (!context) {
    throw new Error("canvas context を取得できません。");
  }

  context.fillStyle = "#d8d8d8";
  context.fillRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = "#2b3644";
  context.fillRect(0, 0, rowClueAreaWidth, colClueAreaHeight);
  context.font = `${Math.round(size * 0.5)}px "DM Mono", monospace`;
  context.textAlign = "center";
  context.textBaseline = "middle";

  for (let row = 0; row < puzzle.row_clues.length; row += 1) {
    const displayRowClue = puzzle.row_clues[row].length === 0 ? [0] : puzzle.row_clues[row];
    for (let slot = 0; slot < maxRowClueSlots; slot += 1) {
      const x = slot * size;
      const y = colClueAreaHeight + row * size;
      const clueValue = displayRowClue[displayRowClue.length - maxRowClueSlots + slot];
      context.fillStyle = "#2b3644";
      context.fillRect(x, y, size, size);
      drawCellBorder(context, x, y, size, row > 0 && row % 5 === 0, false);
      if (clueValue !== undefined) {
        context.fillStyle = "#e6edf3";
        context.fillText(String(clueValue), x + size / 2, y + size / 2);
      }
    }
  }

  for (let col = 0; col < puzzle.col_clues.length; col += 1) {
    const displayColumnClue = puzzle.col_clues[col].length === 0 ? [0] : puzzle.col_clues[col];
    for (let slot = 0; slot < maxColClueSlots; slot += 1) {
      const x = rowClueAreaWidth + col * size;
      const y = slot * size;
      const clueValue = displayColumnClue[displayColumnClue.length - maxColClueSlots + slot];
      context.fillStyle = "#2b3644";
      context.fillRect(x, y, size, size);
      drawCellBorder(context, x, y, size, false, col > 0 && col % 5 === 0);
      if (clueValue !== undefined) {
        context.fillStyle = "#e6edf3";
        context.fillText(String(clueValue), x + size / 2, y + size / 2);
      }
    }
  }

  for (let row = 0; row < puzzle.row_clues.length; row += 1) {
    for (let col = 0; col < puzzle.col_clues.length; col += 1) {
      const x = rowClueAreaWidth + col * size;
      const y = colClueAreaHeight + row * size;
      context.fillStyle = solutionGrid?.[row]?.[col] ? "#1a1a1a" : "#d8d8d8";
      context.fillRect(x, y, size, size);
      drawCellBorder(context, x, y, size, row > 0 && row % 5 === 0, col > 0 && col % 5 === 0);
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

function drawCellBorder(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  size: number,
  majorTop: boolean,
  majorLeft: boolean,
) {
  context.strokeStyle = "#1f2937";
  context.lineWidth = 1;
  context.strokeRect(x, y, size, size);

  if (majorTop) {
    context.strokeStyle = "rgba(17, 24, 39, 0.8)";
    context.lineWidth = 2;
    context.beginPath();
    context.moveTo(x, y + 0.5);
    context.lineTo(x + size, y + 0.5);
    context.stroke();
  }

  if (majorLeft) {
    context.strokeStyle = "rgba(17, 24, 39, 0.8)";
    context.lineWidth = 2;
    context.beginPath();
    context.moveTo(x + 0.5, y);
    context.lineTo(x + 0.5, y + size);
    context.stroke();
  }
}
