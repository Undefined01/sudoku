use super::fish_utils::check_is_fish;
use crate::solver::return_if_some;
use crate::sudoku::{CellValue, Step, StepRule};
use crate::utils::{comb, CellSet, NamedCellSet};
use crate::SudokuSolver;

use std::iter::FromIterator;

use arrayvec::ArrayVec;
use itertools::Itertools;

pub fn search_franken_fish(sudoku: &SudokuSolver, size: usize, value: CellValue) -> Option<Step> {
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

    return_if_some!(search_franken_fish_with(
        sudoku, size, value, &rows, &cols, &blocks
    ));
    return_if_some!(search_franken_fish_with(
        sudoku, size, value, &cols, &rows, &blocks
    ));

    None
}

fn search_franken_fish_with(
    sudoku: &SudokuSolver,
    size: usize,
    value: CellValue,
    rows: &ArrayVec<&NamedCellSet, 9>,
    cols: &ArrayVec<&NamedCellSet, 9>,
    blocks: &ArrayVec<&NamedCellSet, 9>,
) -> Option<Step> {
    let col_sets = comb(&cols, size)
        .map(|col_set| {
            let col_cells = CellSet::union_multiple(col_set.iter().map(|c| &***c));
            (col_set, col_cells)
        })
        .collect_vec();

    for row_set in (0..size).flat_map(|row_size| comb(&rows, row_size)) {
        let row_cells = CellSet::union_multiple(row_set.iter().map(|r| &***r));
        let blocks_not_in_row_set = ArrayVec::<_, 9>::from_iter(
            blocks
                .iter()
                .copied()
                .filter(|&b| row_set.iter().all(|&r| (r & b).is_empty())),
        );
        for block_set in comb(&blocks_not_in_row_set, size - row_set.len()) {
            let row_block_set =
                ArrayVec::<_, 4>::from_iter(row_set.iter().chain(block_set.iter()).cloned());
            let row_block_cells =
                &row_cells | &CellSet::union_multiple(block_set.iter().map(|r| &***r));
            for (col_set, col_cells) in &col_sets {
                return_if_some!(check_is_fish(
                    sudoku,
                    &row_block_set,
                    &col_set,
                    &row_block_cells,
                    &col_cells,
                    value,
                    StepRule::FrankenFish,
                ));
                return_if_some!(check_is_fish(
                    sudoku,
                    &col_set,
                    &row_block_set,
                    &col_cells,
                    &row_block_cells,
                    value,
                    StepRule::FrankenFish,
                ));
            }
        }
    }

    None
}
