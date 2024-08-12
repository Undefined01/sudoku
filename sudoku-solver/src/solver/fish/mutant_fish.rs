use super::fish_utils::check_is_fish;
use crate::solver::return_if_some;
use crate::sudoku::{CellValue, Step, StepRule};
use crate::utils::{CellSet, NamedCellSet};
use crate::SudokuSolver;

use std::iter::FromIterator;

use arrayvec::ArrayVec;
use itertools::Itertools;

pub fn search_mutant_fish(sudoku: &SudokuSolver, size: usize, value: CellValue) -> Option<Step> {
    let rows = ArrayVec::<_, 9>::from_iter(
        sudoku
            .cells_in_rows()
            .iter()
            .map(|s| sudoku.get_possible_cells_for_house_and_value(s, value))
            .filter(|s| s.size() > 1),
    );
    let cols = ArrayVec::<_, 9>::from_iter(
        sudoku
            .cells_in_columns()
            .iter()
            .map(|s| sudoku.get_possible_cells_for_house_and_value(s, value))
            .filter(|s| s.size() > 1),
    );
    let blocks = ArrayVec::<_, 9>::from_iter(
        sudoku
            .cells_in_blocks()
            .iter()
            .map(|s| sudoku.get_possible_cells_for_house_and_value(s, value))
            .filter(|s| s.size() > 1),
    );

    for row_set in (0..size).flat_map(|row_size| sudoku.combinations(&rows, row_size)) {
        let row_cells = CellSet::union_multiple(row_set.iter().map(|r| &****r));
        let blocks_not_in_row_set = ArrayVec::<_, 9>::from_iter(
            blocks
                .iter()
                .copied()
                .filter(|&b| row_set.iter().all(|&&r| (r & b).is_empty())),
        );
        for block_set_1 in sudoku.combinations(&blocks_not_in_row_set, size - row_set.len()) {
            let row_block_set =
                ArrayVec::<_, 4>::from_iter(row_set.iter().chain(block_set_1.iter()).cloned());
            let row_block_cells =
                &row_cells | &CellSet::union_multiple(block_set_1.iter().map(|r| &****r));

            for col_set in (0..size).flat_map(|col_size| sudoku.combinations(&cols, col_size)) {
                let col_cells = CellSet::union_multiple(col_set.iter().map(|c| &****c));
                let blocks_not_in_col_set = ArrayVec::<_, 9>::from_iter(
                    blocks
                        .iter()
                        .copied()
                        .filter(|&b| col_set.iter().all(|&&c| (c & b).is_empty())),
                );
                for block_set_2 in sudoku.combinations(&blocks_not_in_col_set, size - col_set.len())
                {
                    let col_block_set = ArrayVec::<_, 4>::from_iter(
                        col_set.iter().chain(block_set_2.iter()).cloned(),
                    );
                    let col_block_cells =
                        &col_cells | &CellSet::union_multiple(block_set_2.iter().map(|c| &****c));

                    return_if_some!(check_is_fish(
                        sudoku,
                        &row_block_set,
                        &col_block_set,
                        &row_block_cells,
                        &col_block_cells,
                        value,
                        StepRule::MutantFish,
                    ));
                    return_if_some!(check_is_fish(
                        sudoku,
                        &col_block_set,
                        &row_block_set,
                        &col_block_cells,
                        &row_block_cells,
                        value,
                        StepRule::MutantFish,
                    ));
                }
            }
        }
    }

    None
}
