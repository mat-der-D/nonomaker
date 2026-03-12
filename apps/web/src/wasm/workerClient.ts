import * as Comlink from "comlink";
import type { WorkerApi } from "./worker";

let worker: Worker | null = null;
let workerProxy: Comlink.Remote<WorkerApi> | null = null;

export function getWorker() {
  if (!workerProxy) {
    worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
    workerProxy = Comlink.wrap<WorkerApi>(worker);
  }
  return workerProxy;
}

export function terminateWorker() {
  worker?.terminate();
  worker = null;
  workerProxy = null;
}
