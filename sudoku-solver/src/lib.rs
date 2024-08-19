pub mod solver;
mod sudoku;
pub mod utils;

use solver::Techniques;
pub use solver::{SolutionRecorder, SudokuSolver, Technique};
pub use sudoku::Sudoku;

use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn sudoku_one_step(sudoku: &str) -> Option<SolutionRecorder> {
    let sudoku = Sudoku::from_values(sudoku);
    let solver = SudokuSolver::new(sudoku);
    let techniques = Techniques::new();
    solver.solve_one_step(&techniques)
}
