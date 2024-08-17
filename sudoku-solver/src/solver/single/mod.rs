use crate::solver::{Step, SudokuSolver, Technique};

pub fn solve_full_house(sudoku: &SudokuSolver) -> Option<Step> {
    for house in sudoku.all_constraints().iter() {
        let unfilled_cells = house & sudoku.unfilled_cells();
        if unfilled_cells.size() == 1 {
            let unfilled_cell = unfilled_cells.iter().next().unwrap();
            let missing_value = sudoku.candidates(unfilled_cell).iter().next().unwrap();
            return Some(Step::new_value_set(
                Technique::FullHouse,
                format!(
                    "{} is the only missing cell in {}",
                    sudoku.get_cell_name(unfilled_cell),
                    house.name()
                ),
                unfilled_cell,
                missing_value,
            ));
        }
    }
    None
}

pub fn solve_naked_single(sudoku: &SudokuSolver) -> Option<Step> {
    for house in sudoku.all_constraints.iter() {
        for cell in house.iter() {
            if sudoku.candidates(cell).size() == 1 {
                let value = sudoku.candidates(cell).iter().next().unwrap();
                return Some(Step::new_value_set(
                    Technique::NakedSingle,
                    format!(
                        "{} is the only possible value to fill {}",
                        value,
                        sudoku.get_cell_name(cell)
                    ),
                    cell,
                    value,
                ));
            }
        }
    }
    None
}

pub fn solve_hidden_single(sudoku: &SudokuSolver) -> Option<Step> {
    for house in sudoku.all_constraints.iter() {
        for value in 1..=9 {
            let possible_cells = sudoku.get_possible_cells_for_house_and_value(house, value);
            if possible_cells.size() == 1 {
                let target_cell = possible_cells.iter().next().unwrap();
                return Some(Step::new_value_set(
                    Technique::HiddenSingle,
                    format!(
                        "in {}, {} is the only possible cell that can be {}",
                        house.name(),
                        sudoku.get_cell_name(target_cell),
                        value,
                    ),
                    target_cell,
                    value,
                ));
            }
        }
    }
    None
}