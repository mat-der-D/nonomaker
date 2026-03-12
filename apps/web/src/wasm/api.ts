import * as Comlink from "comlink";
import init, * as wasm from "@pkg";
import { getWorker } from "./workerClient";
import type {
  CompleteSolverType,
  Grid,
  ImageToGridParams,
  PartialGrid,
  PartialSolverType,
  Puzzle,
  Solution,
} from "./types";

let wasmReady: Promise<void> | null = null;

export class WasmError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "WasmError";
  }
}

export async function ensureWasm() {
  if (!wasmReady) {
    wasmReady = init();
  }
  await wasmReady;
}

export async function solvePartial(
  puzzle: Puzzle,
  solver: PartialSolverType,
): Promise<PartialGrid | null> {
  try {
    const worker = getWorker();
    const json = await worker.solvePartial(JSON.stringify(puzzle), solver);
    return json ? (JSON.parse(json) as PartialGrid) : null;
  } catch (error) {
    throw new WasmError(String(error));
  }
}

export async function solveComplete(
  puzzle: Puzzle,
  solver: CompleteSolverType,
): Promise<Solution> {
  try {
    const worker = getWorker();
    const json = await worker.solveComplete(JSON.stringify(puzzle), solver);
    return JSON.parse(json) as Solution;
  } catch (error) {
    throw new WasmError(String(error));
  }
}

export async function gridToPuzzle(grid: Grid): Promise<Puzzle> {
  await ensureWasm();
  try {
    const json = await wasm.grid_to_puzzle(JSON.stringify(grid));
    return JSON.parse(json) as Puzzle;
  } catch (error) {
    throw new WasmError(String(error));
  }
}

export async function imageToGrid(
  imageBytes: Uint8Array,
  params: ImageToGridParams,
): Promise<Grid> {
  try {
    const worker = getWorker();
    const transferred = Comlink.transfer(imageBytes, [imageBytes.buffer]);
    const json = await worker.imageToGrid(transferred, JSON.stringify(params));
    return JSON.parse(json) as Grid;
  } catch (error) {
    throw new WasmError(String(error));
  }
}

export async function gridToId(grid: Grid): Promise<string> {
  await ensureWasm();
  try {
    return await wasm.grid_to_id(JSON.stringify(grid));
  } catch (error) {
    throw new WasmError(String(error));
  }
}

export async function idToGrid(id: string): Promise<Grid> {
  await ensureWasm();
  try {
    const json = await wasm.id_to_grid(id);
    return JSON.parse(json) as Grid;
  } catch (error) {
    throw new WasmError(String(error));
  }
}
