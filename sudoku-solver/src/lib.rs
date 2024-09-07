#![feature(portable_simd)]
#![feature(const_for)]
#![feature(core_intrinsics)]

pub mod solver;
mod sudoku;
pub mod utils;

use solver::Techniques;
pub use solver::{SolutionRecorder, SudokuSolver, Technique};
pub use sudoku::Sudoku;

use wasm_bindgen::prelude::*;
use std::ffi::CStr;
use std::os::raw::c_char;

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

// #[no_mangle]
// pub extern "C" fn hudoku_solve(input: *const c_char, limit: usize) -> usize {
//     let line = unsafe { CStr::from_ptr(input) };
//     let sudoku = Sudoku::from_values(&line.to_str().unwrap());
//     let solver = SudokuSolver::new(sudoku);
//     let techniques = Techniques::new();
//     while let Some(_) = solver.solve_one_step(&techniques) {}
//     return solver.is_completed() as usize;
// }

#[no_mangle]
pub extern "C" fn hudoku_solve(input: *const c_char, limit: usize) -> usize {
    let line = unsafe { CStr::from_ptr(input) };
    let mut sudoku = solver::guess::State::from_values(&line.to_str().unwrap());
    return sudoku.solve().is_ok() as usize;
}
