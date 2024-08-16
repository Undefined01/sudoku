use crate::sudoku::{CellValue, Step, StepKind, StepRule};
use crate::utils::{CellSet, NamedCellSet};
use crate::SudokuSolver;

use itertools::Itertools;

#[inline(always)]
pub fn check_is_fish(
    sudoku: &SudokuSolver,
    base_set: &[&NamedCellSet],
    cover_set: &[&NamedCellSet],
    base_cells: &CellSet,
    cover_cells: &CellSet,
    value: CellValue,
    rule: StepRule,
) -> Option<Step> {
    let fins = base_cells - cover_cells;
    let mut eliminated_cells = cover_cells - base_cells;
    if eliminated_cells.is_empty() {
        return None;
    }

    let allow_fins = rule != StepRule::BasicFish;
    if !allow_fins && !fins.is_empty() {
        return None;
    }
    for fin in fins.iter() {
        eliminated_cells &= &sudoku.house_union_of_cell[fin as usize];
    }
    if eliminated_cells.is_empty() {
        return None;
    }

    let mut step = Step::new(StepKind::CandidateEliminated, rule.clone());
    for cell in eliminated_cells.iter() {
        let reason = if fins.is_empty() {
            format!(
                "for {}, {} is covered by {}",
                value,
                base_set.iter().map(|s| s.name()).join(","),
                cover_set.iter().map(|s| s.name()).join(","),
            )
        } else {
            format!(
                "for {}, {} is covered by {} with fins {}",
                value,
                base_set.iter().map(|s| s.name()).join(","),
                cover_set.iter().map(|s| s.name()).join(","),
                sudoku.get_cellset_string(&fins),
            )
        };
        step.add(reason, cell, value);
    }
    return Some(step);
}
