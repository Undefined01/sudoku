use crate::solver::{return_in_fast_mode, SolutionRecorder};
use crate::sudoku::{CellIndex, CellValue};
use crate::utils::{combinations, CombinationOptions};
use crate::{SudokuSolver, Technique};

use std::iter::FromIterator;

use arrayvec::ArrayVec;

pub fn solve_w_wing(sudoku: &SudokuSolver, solution: &mut SolutionRecorder) {
    let paired_cells =
        ArrayVec::<_, 81>::from_iter(sudoku.cells().filter(|&c| sudoku.candidates(c).size() == 2));
    for pair in combinations(&paired_cells, 2, CombinationOptions::default()) {
        let cell1 = pair[0];
        let cell2 = pair[1];

        // 两个单元格在同一行或同一列形成了一个 Naked Pair，不必再搜索
        let pos1 = sudoku.cell_position(cell1);
        let pos2 = sudoku.cell_position(cell2);
        if pos1.0 == pos2.0 || pos1.1 == pos2.1 {
            continue;
        }

        let values1 = sudoku.candidates(cell1);
        let values2 = sudoku.candidates(cell2);
        if values1 != values2 {
            continue;
        }
        let value1 = values1[0];
        let value2 = values1[1];

        inner(sudoku, solution, cell1, cell2, value1, value2);
        return_in_fast_mode!(solution);
        inner(sudoku, solution, cell1, cell2, value2, value1);
        return_in_fast_mode!(solution);
    }
}

fn inner(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    cell_a: CellIndex,
    cell_b: CellIndex,
    value1: CellValue,
    value2: CellValue,
) {
    let eliminated = &(sudoku.possible_cells(value2) & sudoku.house_union_of_cell(cell_a))
        & sudoku.house_union_of_cell(cell_b);

    if eliminated.is_empty() {
        return;
    }

    // 所有有且仅有两个 value 的行与列
    let rows = sudoku.rows_with_only_two_possible_places(value1);
    let cols = sudoku.cols_with_only_two_possible_places(value1);
    let (row_a, col_a, _) = sudoku.cell_position(cell_a);
    let (row_b, col_b, _) = sudoku.cell_position(cell_b);

    for &(_, (col_x, _, cell_x), (col_y, _, cell_y)) in rows {
        debug_assert!(cell_a < cell_b);
        debug_assert!(cell_x < cell_y);
        if cell_x != cell_a && cell_y != cell_b && col_a == col_x && col_b == col_y {
            for cell in eliminated.iter() {
                solution.add_elimination(
                    Technique::WWing,
                    format!(
                        "{} -{}- {} ={}= {} -{}- {} form a WWing",
                        sudoku.get_cell_name(cell_a),
                        value2,
                        sudoku.get_cell_name(cell_x),
                        value1,
                        sudoku.get_cell_name(cell_y),
                        value2,
                        sudoku.get_cell_name(cell_b),
                    ),
                    cell,
                    value2,
                );
            }
            return_in_fast_mode!(solution);
        }
    }

    for &(_, (row_x, _, cell_x), (row_y, _, cell_y)) in cols {
        debug_assert!(cell_a < cell_b);
        debug_assert!(cell_x < cell_y);
        if cell_x != cell_a && cell_y != cell_b && row_a == row_x && row_b == row_y {
            for cell in eliminated.iter() {
                solution.add_elimination(
                    Technique::WWing,
                    format!(
                        "{} -{}- {} ={}= {} -{}- {} form a WWing",
                        sudoku.get_cell_name(cell_a),
                        value2,
                        sudoku.get_cell_name(cell_x),
                        value1,
                        sudoku.get_cell_name(cell_y),
                        value2,
                        sudoku.get_cell_name(cell_b),
                    ),
                    cell,
                    value2,
                );
            }
            return_in_fast_mode!(solution);
        }
    }
}
