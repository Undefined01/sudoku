use crate::solver::return_in_fast_mode;
use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::{CellIndex, CellValue};
use crate::utils::NamedCellSet;

pub fn search_rectangle_elimination(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    value: CellValue,
) {
    // 所有有且仅有两个 value 的行与列
    let rows = sudoku.rows_with_only_two_possible_places(value);
    let cols = sudoku.cols_with_only_two_possible_places(value);

    inner1(
        sudoku,
        solution,
        value,
        &rows,
        &sudoku.cells_in_rows(),
        &sudoku.cells_in_columns(),
    );
    return_in_fast_mode!(solution);
    inner1(
        sudoku,
        solution,
        value,
        &cols,
        &sudoku.cells_in_columns(),
        &sudoku.cells_in_rows(),
    );
    return_in_fast_mode!(solution);
}

fn inner1(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    value: CellValue,
    rows_with_two_places: &[(
        NamedCellSet,
        (usize, usize, CellIndex),
        (usize, usize, CellIndex),
    )],
    rows: &[NamedCellSet],
    cols: &[NamedCellSet],
) {
    if rows.is_empty() {
        return;
    }

    for (row_1, (col_1, _, _), (col_2, _, _)) in rows_with_two_places {
        // col 1 and col 2 should not be in the same block
        if (col_1 / 3) == (col_2 / 3) {
            continue;
        }
        let col_1 = &cols[*col_1];
        let col_2 = &cols[*col_2];
        inner2(sudoku, solution, value, rows, row_1, col_1, col_2);
        return_in_fast_mode!(solution);
        inner2(sudoku, solution, value, rows, row_1, col_2, col_1);
        return_in_fast_mode!(solution);
    }
}

fn inner2(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    value: CellValue,
    rows: &[NamedCellSet],
    row_1: &NamedCellSet,
    col_1: &NamedCellSet,
    col_2: &NamedCellSet,
) {
    if col_1.size() <= 1 {
        return;
    }
    for row_2 in rows {
        // row 1 and row 2 should not be in the same block
        if (row_1.idx() / 3) == (row_2.idx() / 3) {
            continue;
        }
        if !(sudoku.get_possible_cells_for_house_and_value(row_2, value) & col_1).is_empty() {
            let block_idx = sudoku
                .cell_position(sudoku.cell_of_intersection(row_2, col_2))
                .2;
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
                solution.add_elimination(
                    Technique::RectangleElimination,
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
                return_in_fast_mode!(solution);
            }
        }
    }
}
