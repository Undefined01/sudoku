mod rectangle_elimination;
mod skyscraper;
mod two_string_kite;

use crate::SudokuSolver;

use super::return_if_some;

pub fn solve_two_string_kite(sudoku: &SudokuSolver) -> Option<crate::Step> {
    for value in 1..=9 {
        return_if_some!(two_string_kite::search_two_string_kite(sudoku, value));
    }
    None
}

pub fn solve_skyscraper(sudoku: &SudokuSolver) -> Option<crate::Step> {
    for value in 1..=9 {
        return_if_some!(skyscraper::search_skyscraper(sudoku, value));
    }
    None
}

pub fn solve_rectangle_elimination(sudoku: &SudokuSolver) -> Option<crate::Step> {
    for value in 1..=9 {
        return_if_some!(rectangle_elimination::search_rectangle_elimination(
            sudoku, value
        ));
    }
    None
}
