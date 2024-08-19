use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::CellValue;
use crate::utils::{CellSet, NamedCellSet};

use itertools::Itertools;

#[inline(always)]
pub fn check_is_fish(
    sudoku: &SudokuSolver,
    solution: &mut SolutionRecorder,
    base_set: &[&NamedCellSet],
    cover_set: &[&NamedCellSet],
    base_cells: &CellSet,
    cover_cells: &CellSet,
    value: CellValue,
    rule: Technique,
) {
    let fins = base_cells - cover_cells;
    let mut eliminated_cells = cover_cells - base_cells;
    if eliminated_cells.is_empty() {
        return;
    }

    let allow_fins = rule != Technique::BasicFish;
    if !allow_fins && !fins.is_empty() {
        return;
    }
    for fin in fins.iter() {
        eliminated_cells &= &sudoku.house_union_of_cell[fin as usize];
    }
    if eliminated_cells.is_empty() {
        return;
    }

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
        solution.add_elimination(rule.clone(), reason, cell, value);
    }
}
