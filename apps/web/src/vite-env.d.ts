/// <reference types="vite/client" />

declare module "@pkg" {
  export default function init(): Promise<void>;

  export function solve_partial(
    puzzleJson: string,
    solver: string,
  ): Promise<string | null> | string | null;

  export function solve_complete(
    puzzleJson: string,
    solver: string,
  ): Promise<string> | string;

  export function grid_to_puzzle(gridJson: string): Promise<string> | string;
  export function image_to_grid(
    imageBytes: Uint8Array,
    paramsJson: string,
  ): Promise<string> | string;
  export function grid_to_id(gridJson: string): Promise<string> | string;
  export function id_to_grid(id: string): Promise<string> | string;
}
