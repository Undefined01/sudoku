use super::fish_utils::check_is_fish;
use crate::solver::return_in_fast_mode;
use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::CellValue;
use crate::utils::{comb, CellSet};

use std::iter::FromIterator;

use arrayvec::ArrayVec;
use itertools::Itertools;

/// Search for simple fish in the sudoku.
pub fn search_simple_fish(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    size: usize,
    value: CellValue,
    rule: Technique,
) {
    debug_assert!(size >= 2 && size <= 4);
    debug_assert!(value >= 1 && value <= 9);
    debug_assert!(rule == Technique::BasicFish || rule == Technique::FinnedFish);

    let rows_in_size = ArrayVec::<_, 9>::from_iter(
        sudoku
            .cells_in_rows()
            .iter()
            .map(|s| sudoku.get_possible_cells_for_house_and_value(s, value))
            .filter(|s| s.size() > 1),
    );
    let cols_in_size = ArrayVec::<_, 9>::from_iter(
        sudoku
            .cells_in_columns()
            .iter()
            .map(|s| sudoku.get_possible_cells_for_house_and_value(s, value))
            .filter(|s| s.size() > 1),
    );

    let row_sets = comb(&rows_in_size, size)
        .map(|row_set| {
            let row_cells = CellSet::union_multiple(row_set.iter().map(|r| &***r));
            (row_set, row_cells)
        })
        .collect_vec();
    let col_sets = comb(&cols_in_size, size)
        .map(|col_set| {
            let col_cells = CellSet::union_multiple(col_set.iter().map(|c| &***c));
            (col_set, col_cells)
        })
        .collect_vec();

    for (row_set, row_cells) in &row_sets {
        for (col_set, col_cells) in &col_sets {
            check_is_fish(
                sudoku,
                solution,
                row_set,
                col_set,
                row_cells,
                col_cells,
                value,
                rule.clone(),
            );
            return_in_fast_mode!(solution);
            check_is_fish(
                sudoku,
                solution,
                col_set,
                row_set,
                col_cells,
                row_cells,
                value,
                rule.clone(),
            );
            return_in_fast_mode!(solution);
        }
    }
}
