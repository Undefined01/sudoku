mod sudoku;

pub use sudoku::{Step, Sudoku, SudokuSolver};

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, sudoku-solver!");
}

#[wasm_bindgen]
pub fn sudoku_one_step(sudoku: &str) -> Option<Step> {
    let mut sudoku = Sudoku::from_str(sudoku);
    let solver = SudokuSolver::new(&mut sudoku);
    solver.solve_one_step(&mut sudoku)
}
