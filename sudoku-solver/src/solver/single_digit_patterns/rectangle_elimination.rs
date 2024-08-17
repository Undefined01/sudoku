use crate::solver::return_if_some;
use crate::solver::{Step, StepKind, SudokuSolver, Technique};
use crate::sudoku::{CellIndex, CellValue};
use crate::utils::NamedCellSet;

pub fn search_rectangle_elimination(sudoku: &SudokuSolver, value: CellValue) -> Option<Step> {
    // 所有有且仅有两个 value 的行与列
    let rows = sudoku.rows_with_only_two_possible_places(value);
    let cols = sudoku.cols_with_only_two_possible_places(value);

    return_if_some!(inner1(
        sudoku,
        value,
        &rows,
        &sudoku.cells_in_rows(),
        &sudoku.cells_in_columns()
    ));
    return_if_some!(inner1(
        sudoku,
        value,
        &cols,
        &sudoku.cells_in_columns(),
        &sudoku.cells_in_rows()
    ));

    None
}

fn inner1(
    sudoku: &SudokuSolver,
    value: CellValue,
    rows_with_two_places: &[(NamedCellSet, (usize, usize), (CellIndex, CellIndex))],
    rows: &[NamedCellSet],
    cols: &[NamedCellSet],
) -> Option<Step> {
    if rows.is_empty() {
        return None;
    }

    for (row_1, (col_1, col_2), _) in rows_with_two_places {
        // col 1 and col 2 should not be in the same block
        if (col_1 / 3) == (col_2 / 3) {
            continue;
        }
        let col_1 = &cols[*col_1];
        let col_2 = &cols[*col_2];
        return_if_some!(inner2(sudoku, value, rows, cols, row_1, col_1, col_2));
        return_if_some!(inner2(sudoku, value, rows, cols, row_1, col_2, col_1));
    }

    None
}

fn inner2(
    sudoku: &SudokuSolver,
    value: CellValue,
    rows: &[NamedCellSet],
    cols: &[NamedCellSet],
    row_1: &NamedCellSet,
    col_1: &NamedCellSet,
    col_2: &NamedCellSet,
) -> Option<Step> {
    if col_1.size() <= 1 {
        return None;
    }
    for row_2 in rows {
        // row 1 and row 2 should not be in the same block
        if (row_1.idx() / 3) == (row_2.idx() / 3) {
            continue;
        }
        if !(sudoku.get_possible_cells_for_house_and_value(row_2, value) & col_1).is_empty() {
            let block_idx = sudoku.block_id_of_cell(sudoku.cell_of_intersection(row_2, col_2));
            let block = &sudoku.cells_in_blocks()[block_idx];
            if sudoku
                .get_possible_cells_for_house_and_value(block, value)
                .is_empty()
            {
                continue;
            }
            if sudoku
                .get_possible_cells_for_house_and_value(block, value)
                .is_subset_of(&(row_2 | col_2))
            {
                let mut step = Step::new(
                    StepKind::CandidateEliminated,
                    Technique::RectangleElimination,
                );
                step.add(
                    format!(
                        "if {} is {}, then {} cannot be {}, and {} must be {}, which eliminates all possible places for {} in {}",
                        sudoku.get_cell_name(sudoku.cell_of_intersection(row_2, col_1)), value,
                        sudoku.get_cell_name(sudoku.cell_of_intersection(row_1, col_1)), value,
                        sudoku.get_cell_name(sudoku.cell_of_intersection(row_1, col_2)), value,
                        value, block.name(),
                    ),
                    sudoku.cell_of_intersection(row_2, col_1),
                    value
                );
                return Some(step);
            }
        }
    }

    None
}
