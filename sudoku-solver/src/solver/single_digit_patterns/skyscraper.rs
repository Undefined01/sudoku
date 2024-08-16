use crate::solver::return_if_some;
use crate::sudoku::{CellIndex, CellValue, Step, StepKind, StepRule};
use crate::utils::{comb, NamedCellSet};
use crate::SudokuSolver;

use std::iter::FromIterator;

use arrayvec::ArrayVec;

pub fn search_skyscraper(sudoku: &SudokuSolver, value: CellValue) -> Option<Step> {
    // 所有有且仅有两个 value 的行与列
    let rows = ArrayVec::<_, 9>::from_iter(
        sudoku
            .candidate_cells_in_rows(value)
            .iter()
            .filter(|row| row.size() == 2)
            .map(|row| {
                let cell_ids = ArrayVec::<_, 2>::from_iter(row.iter());
                let column_ids = ArrayVec::<_, 2>::from_iter(
                    cell_ids.iter().map(|&cell| sudoku.column_id_of_cell(cell)),
                );
                (
                    row,
                    (column_ids[0], column_ids[1]),
                    (cell_ids[0], cell_ids[1]),
                )
            }),
    );
    let cols = ArrayVec::<_, 9>::from_iter(
        sudoku
            .candidate_cells_in_columns(value)
            .iter()
            .filter(|col| col.size() == 2)
            .map(|col| {
                let cell_ids = ArrayVec::<_, 2>::from_iter(col.iter());
                let row_ids = ArrayVec::<_, 2>::from_iter(
                    cell_ids.iter().map(|&cell| sudoku.row_id_of_cell(cell)),
                );
                (col, (row_ids[0], row_ids[1]), (cell_ids[0], cell_ids[1]))
            }),
    );

    return_if_some!(search_skyscraper_inner(
        sudoku,
        value,
        &rows,
        sudoku.cells_in_columns()
    ));
    return_if_some!(search_skyscraper_inner(
        sudoku,
        value,
        &cols,
        sudoku.cells_in_rows()
    ));

    None
}

fn search_skyscraper_inner(
    sudoku: &SudokuSolver,
    value: CellValue,
    rows: &ArrayVec<(&NamedCellSet, (usize, usize), (CellIndex, CellIndex)), 9>,
    cols: &[NamedCellSet],
) -> Option<Step> {
    if rows.is_empty() {
        return None;
    }

    for pair in comb(&rows, 2) {
        let (row_a, (col_a, col_b), (cell_a, cell_b)) = pair[0];
        let (row_b, (col_x, col_y), (cell_x, cell_y)) = pair[1];

        let common_col;
        let cell_1;
        let cell_2;
        if col_a == col_x {
            common_col = col_a;
            cell_1 = cell_b;
            cell_2 = cell_y;
        } else if col_a == col_y {
            common_col = col_a;
            cell_1 = cell_b;
            cell_2 = cell_x;
        } else if col_b == col_x {
            common_col = col_b;
            cell_1 = cell_a;
            cell_2 = cell_y;
        } else if col_b == col_y {
            common_col = col_b;
            cell_1 = cell_a;
            cell_2 = cell_x;
        } else {
            continue;
        }

        if sudoku.block_id_of_cell(cell_1) == sudoku.block_id_of_cell(cell_2) {
            continue;
        }

        let mut eliminated_cells =
            sudoku.house_union_of_cell(cell_1) & sudoku.house_union_of_cell(cell_2);
        if eliminated_cells.is_empty() {
            continue;
        }
        eliminated_cells &= sudoku.possible_cells(value);
        if !eliminated_cells.is_empty() {
            let mut step = Step::new(StepKind::CandidateEliminated, StepRule::Skyscraper);
            for cell in eliminated_cells.iter() {
                let common_cols_name = cols[common_col].name();
                step.add(
                    format!(
                        "there are only two possible cells to place {} in {} and {}, and two of those cells shares {}", value, row_a.name(), row_b.name(), common_cols_name),
                    cell, value);
            }
            return Some(step);
        }
    }

    None
}
