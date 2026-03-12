import { useEffect, useState } from "react";
import { ensureWasm } from "../wasm/api";

type WasmState = "idle" | "loading" | "ready" | "error";

export function useWasm() {
  const [state, setState] = useState<WasmState>("idle");

  useEffect(() => {
    let cancelled = false;
    setState("loading");
    ensureWasm()
      .then(() => {
        if (!cancelled) {
          setState("ready");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setState("error");
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return {
    state,
    isLoading: state === "loading" || state === "idle",
    isReady: state === "ready",
  };
}
