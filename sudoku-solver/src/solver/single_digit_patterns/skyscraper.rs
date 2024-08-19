use crate::solver::return_in_fast_mode;
use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::{CellIndex, CellValue};
use crate::utils::{comb_ref, NamedCellSet};

pub fn search_skyscraper(sudoku: &SudokuSolver, solution: &mut SolutionRecorder, value: CellValue) {
    // 所有有且仅有两个 value 的行与列
    let rows = sudoku.rows_with_only_two_possible_places(value);
    let cols = sudoku.cols_with_only_two_possible_places(value);
    search_skyscraper_inner(sudoku, solution, value, &rows, sudoku.cells_in_columns());
    return_in_fast_mode!(solution);
    search_skyscraper_inner(sudoku, solution, value, &cols, sudoku.cells_in_rows());
    return_in_fast_mode!(solution);
}

fn search_skyscraper_inner(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    value: CellValue,
    rows: &[(
        NamedCellSet,
        (usize, usize, CellIndex),
        (usize, usize, CellIndex),
    )],
    cols: &[NamedCellSet],
) {
    if rows.is_empty() {
        return;
    }

    for pair in comb_ref(&rows, 2) {
        let &(ref row_a, (col_a, _, cell_a), (col_b, _, cell_b)) = pair[0];
        let &(ref row_b, (col_x, _, cell_x), (col_y, _, cell_y)) = pair[1];

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

        let block_1 = sudoku.cell_position(cell_1).2;
        let block_2 = sudoku.cell_position(cell_2).2;
        if block_1 == block_2 {
            continue;
        }

        let mut eliminated_cells =
            sudoku.house_union_of_cell(cell_1) & sudoku.house_union_of_cell(cell_2);
        if eliminated_cells.is_empty() {
            continue;
        }
        eliminated_cells &= sudoku.possible_cells(value);
        if !eliminated_cells.is_empty() {
            for cell in eliminated_cells.iter() {
                let common_cols_name = cols[common_col].name();
                solution.add_elimination(
                    Technique::Skyscraper,
                    format!(
                        "there are only two possible cells to place {} in {} and {}, and two of those cells shares {}",
                        value,
                        row_a.name(),
                        row_b.name(),
                        common_cols_name
                    ),
                    cell, value);
            }
            return_in_fast_mode!(solution);
        }
    }
}
