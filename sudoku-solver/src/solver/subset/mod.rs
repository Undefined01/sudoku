use crate::solver::{Step, StepKind, SudokuSolver, Technique};
use crate::utils::{comb, CellSet, ValueSet};

use arrayvec::ArrayVec;
use itertools::Itertools;

// 在一个 House 中，若任意 n 个数字只可能出现在相同 n 个（或更少）单元格中，则这 n 个单元格中不可能出现其他数字
pub fn solve_hidden_subset(sudoku: &SudokuSolver) -> Option<Step> {
    let mut step = Step::new(StepKind::CandidateEliminated, Technique::HiddenSubset);

    for house in sudoku.all_constraints.iter() {
        let mut possible_cells_in_houses = vec![];
        for value in 1..=9 {
            let possible_cells_in_house =
                sudoku.get_possible_cells_for_house_and_value(house, value);
            if !possible_cells_in_house.is_empty() {
                possible_cells_in_houses.push((value, possible_cells_in_house));
            }
        }

        for size in 2..=4 {
            let possible_house_cells_for_candidate_in_size = ArrayVec::<_, 9>::from_iter(
                possible_cells_in_houses
                    .iter()
                    .filter(|(_, cells)| cells.size() <= size),
            );

            if possible_house_cells_for_candidate_in_size.len() < size {
                continue;
            }

            for subset in comb(&possible_house_cells_for_candidate_in_size, size) {
                let cell_union = CellSet::union_multiple(subset.iter().map(|(_, cells)| &***cells));
                let values_in_subset = ValueSet::from_iter(subset.iter().map(|(value, _)| *value));

                if cell_union.size() <= size {
                    for cell in cell_union.iter() {
                        for value in 1..=9 {
                            if !values_in_subset.has(value) && sudoku.can_fill(cell, value) {
                                step.add(
                                    format!(
                                        "in {}, {} only appears in {}",
                                        house.name(),
                                        values_in_subset.iter().join(","),
                                        sudoku.get_cellset_string(&cell_union),
                                    ),
                                    cell,
                                    value,
                                );
                            }
                        }
                    }
                    if !step.is_empty() {
                        return Some(step);
                    }
                }
            }
        }
    }
    None
}

// 当一个 House 中的 n 个单元格只包含相同的 n 个（或更少）数字时，这 n 个数字不可能出现在这个 House 中的其他单元格中
pub fn solve_naked_subset(sudoku: &SudokuSolver) -> Option<Step> {
    for house in sudoku.all_constraints.iter() {
        for size in 2..=4 {
            let mut step = Step::new(StepKind::CandidateEliminated, Technique::NakedSubset);

            for subset in house
                .iter()
                .filter(|&cell| {
                    !sudoku.candidates(cell).is_empty() && sudoku.candidates(cell).size() <= size
                })
                .combinations(size)
            {
                let value_union = ValueSet::from_iter(
                    subset
                        .iter()
                        .flat_map(|&cell| sudoku.candidates(cell).iter()),
                );
                let cells_in_subset = CellSet::from_iter(subset);

                if value_union.size() > size {
                    continue;
                }

                for cell in house.iter() {
                    if cells_in_subset.has(cell) {
                        continue;
                    }
                    for value in value_union.iter().sorted() {
                        if sudoku.can_fill(cell, value) {
                            step.add(
                                format!(
                                    "in {}, {} only contains {}",
                                    house.name(),
                                    sudoku.get_cellset_string(&cells_in_subset),
                                    value_union.iter().sorted().join(","),
                                ),
                                cell,
                                value,
                            );
                        }
                    }
                }

                if !step.is_empty() {
                    return Some(step);
                }
            }
        }
    }
    None
}
