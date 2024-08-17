use crate::solver::{Step, StepKind, SudokuSolver, Technique};
use crate::utils::NamedCellSet;

// 当 House A 中的一个数字只出现在 House A & House B （A 和 B的交集）中时，这个数字不可能再出现在 House B 中的其他单元格中
pub fn solve_locked_candidates(sudoku: &SudokuSolver) -> Option<Step> {
    let check = |house_a: &NamedCellSet, house_b: &NamedCellSet| -> Option<Step> {
        let intersection = house_a & house_b;
        if intersection.is_empty() {
            return None;
        }

        let mut step = Step::new(StepKind::CandidateEliminated, Technique::LockedCandidates);

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
                    step.add(
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
            if !step.is_empty() {
                return Some(step);
            }
        }
        None
    };

    for block in &sudoku.cells_in_blocks {
        for row in &sudoku.cells_in_rows {
            let step = check(block, row);
            if step.is_some() {
                return step;
            }
            let step = check(row, block);
            if step.is_some() {
                return step;
            }
        }
        for column in &sudoku.cells_in_columns {
            let step = check(block, column);
            if step.is_some() {
                return step;
            }
            let step = check(column, block);
            if step.is_some() {
                return step;
            }
        }
    }
    None
}
