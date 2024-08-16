use crate::sudoku::{CellValue, Step, StepKind, StepRule};
use crate::SudokuSolver;

use std::iter::FromIterator;

use arrayvec::ArrayVec;

pub fn search_two_string_kite(sudoku: &SudokuSolver, value: CellValue) -> Option<Step> {
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
                let box_ids = ArrayVec::<_, 2>::from_iter(
                    cell_ids.iter().map(|&cell| sudoku.block_id_of_cell(cell)),
                );
                (
                    row,
                    (column_ids[0], column_ids[1]),
                    (box_ids[0], box_ids[1]),
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
                let box_ids = ArrayVec::<_, 2>::from_iter(
                    cell_ids.iter().map(|&cell| sudoku.block_id_of_cell(cell)),
                );
                (col, (row_ids[0], row_ids[1]), (box_ids[0], box_ids[1]))
            }),
    );

    if rows.is_empty() || cols.is_empty() {
        return None;
    }

    for (row, col_ids, (box_a, box_b)) in &rows {
        for (col, row_ids, (box_x, box_y)) in &cols {
            if !(*row & *col).is_empty() {
                continue;
            }

            let col_b;
            let row_b;
            if box_a == box_x {
                col_b = col_ids.1;
                row_b = row_ids.1;
            } else if box_a == box_y {
                col_b = col_ids.1;
                row_b = row_ids.0;
            } else if box_b == box_x {
                col_b = col_ids.0;
                row_b = row_ids.1;
            } else if box_b == box_y {
                col_b = col_ids.0;
                row_b = row_ids.0;
            } else {
                continue;
            }

            let eliminated_cell = sudoku.cell_index(row_b, col_b);
            if sudoku.can_fill(eliminated_cell, value) {
                let mut step = Step::new(StepKind::CandidateEliminated, StepRule::TwoStringKite);
                step.add(
                    format!(
                        "for {}, there are only two places in {} and {}",
                        value,
                        row.name(),
                        col.name(),
                    ),
                    eliminated_cell,
                    value,
                );
                return Some(step);
            }
        }
    }

    None
}
