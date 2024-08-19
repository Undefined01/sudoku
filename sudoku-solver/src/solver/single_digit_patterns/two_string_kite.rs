use crate::solver::{return_in_fast_mode, SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::CellValue;

pub fn search_two_string_kite(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    value: CellValue,
) {
    // 所有有且仅有两个 value 的行与列
    let rows = sudoku.rows_with_only_two_possible_places(value);
    let cols = sudoku.cols_with_only_two_possible_places(value);

    if rows.is_empty() || cols.is_empty() {
        return;
    }

    for (row, (col_a, block_a, _), (col_b, block_b, _)) in rows {
        for (col, (row_x, block_x, _), (row_y, block_y, _)) in cols {
            if !(row & col).is_empty() {
                continue;
            }

            let eliminated_cell;
            if block_a == block_x {
                eliminated_cell = sudoku.cell_index(*row_y, *col_b);
            } else if block_a == block_y {
                eliminated_cell = sudoku.cell_index(*row_x, *col_b);
            } else if block_b == block_x {
                eliminated_cell = sudoku.cell_index(*row_y, *col_a);
            } else if block_b == block_y {
                eliminated_cell = sudoku.cell_index(*row_x, *col_a);
            } else {
                continue;
            }

            if sudoku.can_fill(eliminated_cell, value) {
                solution.add_elimination(
                    Technique::TwoStringKite,
                    format!(
                        "for {}, there are only two places in {} and {}",
                        value,
                        row.name(),
                        col.name(),
                    ),
                    eliminated_cell,
                    value,
                );
                return_in_fast_mode!(solution);
            }
        }
    }
}
