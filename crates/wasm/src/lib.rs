use nonomaker_core::{
    ImageConvertParams, format, grid_to_id as core_grid_to_id,
    grid_to_puzzle as core_grid_to_puzzle, id_to_grid as core_id_to_grid,
    image_to_grid as core_image_to_grid,
    solver::{BacktrackingSolver, CompleteSolver, PartialSolver, PropagationSolver, SatSolver},
};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
struct ImageToGridParamsJson {
    smooth_strength: f32,
    threshold: u8,
    edge_strength: f32,
    noise_removal: u32,
    grid_width: u32,
    grid_height: u32,
}

#[wasm_bindgen(start)]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn solve_partial(puzzle_json: &str, solver: &str) -> Result<Option<String>, JsValue> {
    let puzzle = parse_puzzle(puzzle_json)?;
    match solver {
        "linear" | "fp2" => Ok(PropagationSolver
            .solve_partial(&puzzle)
            .map(|grid| format::partial_grid_to_json(&grid))),
        other => Err(JsValue::from_str(&format!(
            "unsupported partial solver: {other}"
        ))),
    }
}

#[wasm_bindgen]
pub fn solve_complete(puzzle_json: &str, solver: &str) -> Result<String, JsValue> {
    let puzzle = parse_puzzle(puzzle_json)?;
    let solution = match solver {
        "backtracking" => BacktrackingSolver::new(2).solve_complete(&puzzle),
        "sat" => SatSolver::new(2).solve_complete(&puzzle),
        other => {
            return Err(JsValue::from_str(&format!(
                "unsupported complete solver: {other}"
            )));
        }
    };
    format::solution_to_json(&solution).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn grid_to_puzzle(grid_json: &str) -> Result<String, JsValue> {
    let grid = format::grid_from_json(grid_json).map_err(to_js_error)?;
    let puzzle = core_grid_to_puzzle(&grid).map_err(to_js_error)?;
    Ok(format::puzzle_to_json(&puzzle))
}

#[wasm_bindgen]
pub fn image_to_grid(image_bytes: &[u8], params_json: &str) -> Result<String, JsValue> {
    let params: ImageToGridParamsJson = serde_json::from_str(params_json)
        .map_err(|err| JsValue::from_str(&format!("invalid image params JSON: {err}")))?;
    let grid = core_image_to_grid(
        image_bytes,
        &ImageConvertParams {
            smooth_strength: params.smooth_strength,
            threshold: params.threshold,
            edge_strength: params.edge_strength,
            noise_removal: params.noise_removal,
            grid_width: params.grid_width,
            grid_height: params.grid_height,
        },
    )
    .map_err(to_js_error)?;
    format::grid_to_json(&grid).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn grid_to_id(grid_json: &str) -> Result<String, JsValue> {
    let grid = format::grid_from_json(grid_json).map_err(to_js_error)?;
    core_grid_to_id(&grid).map_err(to_js_error)
}

#[wasm_bindgen]
pub fn id_to_grid(id: &str) -> Result<String, JsValue> {
    let grid = core_id_to_grid(id).map_err(to_js_error)?;
    format::grid_to_json(&grid).map_err(to_js_error)
}

fn parse_puzzle(puzzle_json: &str) -> Result<nonomaker_core::Puzzle, JsValue> {
    format::puzzle_from_json(puzzle_json).map_err(to_js_error)
}

fn to_js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
