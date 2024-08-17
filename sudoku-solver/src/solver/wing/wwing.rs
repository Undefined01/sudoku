use crate::solver::{return_if_some, Step};
use crate::sudoku::{CellIndex, CellValue};
use crate::utils::{combinations, CombinationOptions};
use crate::{SudokuSolver, Technique};

use std::iter::FromIterator;

use arrayvec::ArrayVec;

pub fn solve_w_wing(sudoku: &SudokuSolver) -> Option<Step> {
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

        return_if_some!(inner(sudoku, cell1, cell2, value1, value2));
        return_if_some!(inner(sudoku, cell1, cell2, value2, value1));
    }

    None
}

fn inner(
    sudoku: &SudokuSolver,
    cell1: CellIndex,
    cell2: CellIndex,
    value1: CellValue,
    value2: CellValue,
) -> Option<Step> {
    let eliminated = &(sudoku.possible_cells(value2) & sudoku.house_union_of_cell(cell1))
        & sudoku.house_union_of_cell(cell2);

    if eliminated.is_empty() {
        return None;
    }

    // 所有有且仅有两个 value 的行与列
    let rows = sudoku.rows_with_only_two_possible_places(value1);
    let cols = sudoku.cols_with_only_two_possible_places(value1);

    for &(_, (col1, col2), (cell_x, cell_y)) in rows {
        let pos1 = sudoku.cell_position(cell1);
        let pos2 = sudoku.cell_position(cell2);
        debug_assert!(cell1 < cell2);
        debug_assert!(cell_x < cell_y);
        if cell_x != cell1 && cell_y != cell2 && pos1.1 == col1 && pos2.1 == col2 {
            let mut step = Step::new_elimination(Technique::WWing);
            for cell in eliminated.iter() {
                step.add(
                    format!(
                        "{} -{}- {} ={}= {} -{}- {} form a WWing",
                        sudoku.get_cell_name(cell1),
                        value2,
                        sudoku.get_cell_name(cell_x),
                        value1,
                        sudoku.get_cell_name(cell_y),
                        value2,
                        sudoku.get_cell_name(cell2),
                    ),
                    cell,
                    value2,
                );
            }
            return Some(step);
        }
    }

    for &(_, (row1, row2), (cell_x, cell_y)) in cols {
        let pos1 = sudoku.cell_position(cell1);
        let pos2 = sudoku.cell_position(cell2);
        debug_assert!(cell1 < cell2);
        debug_assert!(cell_x < cell_y);
        if cell_x != cell1 && cell_y != cell2 && pos1.0 == row1 && pos2.0 == row2 {
            let mut step = Step::new_elimination(Technique::WWing);
            for cell in eliminated.iter() {
                step.add(
                    format!(
                        "{} -{}- {} ={}= {} -{}- {} form a WWing",
                        sudoku.get_cell_name(cell1),
                        value2,
                        sudoku.get_cell_name(cell_x),
                        value1,
                        sudoku.get_cell_name(cell_y),
                        value2,
                        sudoku.get_cell_name(cell2),
                    ),
                    cell,
                    value2,
                );
            }
            return Some(step);
        }
    }

    None
}
