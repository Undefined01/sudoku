mod sudoku;
pub mod solver;
pub mod utils;

pub use sudoku::{Step, Sudoku};
pub use solver::SudokuSolver;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn sudoku_one_step(sudoku: &str) -> Option<Step> {
    let sudoku = Sudoku::from_values(sudoku);
    let solver = SudokuSolver::new(sudoku);
    solver.solve_one_step()
}
