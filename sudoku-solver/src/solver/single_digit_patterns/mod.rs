mod rectangle_elimination;
mod skyscraper;
mod two_string_kite;

use crate::solver::{return_in_fast_mode, SolutionRecorder, SudokuSolver};

pub fn solve_two_string_kite(sudoku: &SudokuSolver, solution: &mut SolutionRecorder) {
    for value in 1..=9 {
        two_string_kite::search_two_string_kite(sudoku, solution, value);
        return_in_fast_mode!(solution);
    }
}

pub fn solve_skyscraper(sudoku: &SudokuSolver, solution: &mut SolutionRecorder) {
    for value in 1..=9 {
        skyscraper::search_skyscraper(sudoku, solution, value);
        return_in_fast_mode!(solution);
    }
}

pub fn solve_rectangle_elimination(sudoku: &SudokuSolver, solution: &mut SolutionRecorder) {
    for value in 1..=9 {
        rectangle_elimination::search_rectangle_elimination(sudoku, solution, value);
        return_in_fast_mode!(solution);
    }
}
