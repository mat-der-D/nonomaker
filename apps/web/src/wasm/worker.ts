import * as Comlink from "comlink";
import init, * as wasm from "@pkg";

let ready: Promise<void> | null = null;

function ensureReady() {
  if (!ready) {
    ready = init();
  }
  return ready;
}

const api = {
  async solveComplete(puzzleJson: string, solver: string) {
    await ensureReady();
    return wasm.solve_complete(puzzleJson, solver);
  },
  async solvePartial(puzzleJson: string, solver: string) {
    await ensureReady();
    return wasm.solve_partial(puzzleJson, solver);
  },
  async imageToGrid(imageBytes: Uint8Array, paramsJson: string) {
    await ensureReady();
    return wasm.image_to_grid(imageBytes, paramsJson);
  },
};

export type WorkerApi = typeof api;

Comlink.expose(api);
