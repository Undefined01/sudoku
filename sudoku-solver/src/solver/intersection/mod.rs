use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::utils::NamedCellSet;

use super::return_in_fast_mode;

// 当 House A 中的一个数字只出现在 House A & House B （A 和 B的交集）中时，这个数字不可能再出现在 House B 中的其他单元格中
pub fn solve_locked_candidates(sudoku: &SudokuSolver, solution: &mut SolutionRecorder) {
    for block in &sudoku.cells_in_blocks {
        for row in &sudoku.cells_in_rows {
            check(sudoku, solution, block, row);
            return_in_fast_mode!(solution);
            check(sudoku, solution, row, block);
            return_in_fast_mode!(solution);
        }
        for column in &sudoku.cells_in_columns {
            check(sudoku, solution, block, column);
            return_in_fast_mode!(solution);
            check(sudoku, solution, column, block);
            return_in_fast_mode!(solution);
        }
    }
}

fn check(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    house_a: &NamedCellSet,
    house_b: &NamedCellSet,
) {
    let intersection = house_a & house_b;
    if intersection.is_empty() {
        return;
    }

    for value in 1..=9 {
        let possible_cells_in_a = sudoku.get_possible_cells_for_house_and_value(house_a, value);
        if possible_cells_in_a.is_empty() || !possible_cells_in_a.is_subset_of(&intersection) {
            continue;
        }
        for cell in house_b.iter() {
            if intersection.has(cell) {
                continue;
            }
            if sudoku.can_fill(cell, value) {
                solution.add_elimination(
                    Technique::LockedCandidates,
                    format!(
                        "in {}, {} can only be in {} & {}",
                        house_a.name(),
                        value,
                        house_a.name(),
                        house_b.name(),
                    ),
                    cell,
                    value,
                );
            }
        }
        return_in_fast_mode!(solution);
    }
}
