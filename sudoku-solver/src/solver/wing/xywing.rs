use crate::solver::Step;
use crate::utils::CellSet;
use crate::{SudokuSolver, Technique};

use std::iter::FromIterator;

pub fn solve_xy_wing(sudoku: &SudokuSolver) -> Option<Step> {
    let bivalue_cells =
        CellSet::from_iter(sudoku.cells().filter(|&c| sudoku.candidates(c).size() == 2));

    if bivalue_cells.size() < 2 {
        return None;
    }

    let pivots = &bivalue_cells;

    for cell_xy in pivots {
        let possible_pincers = &bivalue_cells & sudoku.house_union_of_cell(cell_xy);
        if possible_pincers.is_empty() {
            continue;
        }

        let xy = sudoku.candidates(cell_xy);
        for cell_xz in possible_pincers.iter() {
            let xz = sudoku.candidates(cell_xz);
            let xyz = xy | xz;
            let x = xy & xz;
            if xy == xz || x.is_empty() {
                continue;
            }

            for cell_yz in possible_pincers.iter() {
                let yz = sudoku.candidates(cell_yz);
                let z = xz & yz;
                if !yz.is_subset_of(&xyz) || yz == xz || yz == xy || z.is_empty() {
                    continue;
                }

                let z = z.single_value();
                let eliminated = &(sudoku.possible_cells(z) & sudoku.house_union_of_cell(cell_xz))
                    & sudoku.house_union_of_cell(cell_yz);
                if eliminated.is_empty() {
                    continue;
                }

                let mut step = Step::new_elimination(Technique::XYWing);
                for cell in eliminated.iter() {
                    step.add(
                        format!(
                            "the pivot {} and the pincers {} and {} form an XY-Wing with xyz={}{}{}",
                            sudoku.get_cell_name(cell_xy),
                            sudoku.get_cell_name(cell_xz),
                            sudoku.get_cell_name(cell_yz),
                            x.single_value(),
                            (xy & yz).single_value(),
                            z,
                        ),
                        cell,
                        z,
                    );
                }
                return Some(step);
            }
        }
    }

    None
}

pub fn solve_xyz_wing(sudoku: &SudokuSolver) -> Option<Step> {
    let bivalue_cells =
        CellSet::from_iter(sudoku.cells().filter(|&c| sudoku.candidates(c).size() == 2));

    if bivalue_cells.size() < 2 {
        return None;
    }

    let pivots = CellSet::from_iter(sudoku.cells().filter(|&c| sudoku.candidates(c).size() == 3));

    for cell_xy in &pivots {
        let possible_pincers = &bivalue_cells & sudoku.house_union_of_cell(cell_xy);
        if possible_pincers.is_empty() {
            continue;
        }

        let xyz = sudoku.candidates(cell_xy);
        for cell_xz in possible_pincers.iter() {
            let xz = sudoku.candidates(cell_xz);
            if !xz.is_subset_of(xyz) {
                continue;
            }

            for cell_yz in possible_pincers.iter() {
                let yz = sudoku.candidates(cell_yz);
                let z = xz & yz;
                if !yz.is_subset_of(&xyz) || yz == xz {
                    continue;
                }
                debug_assert!(z.size() == 1);

                let z_value = z.single_value();
                let eliminated = &(sudoku.possible_cells(z_value)
                    & sudoku.house_union_of_cell(cell_xz))
                    & sudoku.house_union_of_cell(cell_yz);
                let eliminated = &eliminated & sudoku.house_union_of_cell(cell_xy);
                if eliminated.is_empty() {
                    continue;
                }

                let mut step = Step::new_elimination(Technique::XYZWing);
                for cell in eliminated.iter() {
                    step.add(
                        format!(
                            "the pivot {} and the pincers {} and {} form an XY-Wing with xyz={}{}{}",
                            sudoku.get_cell_name(cell_xy),
                            sudoku.get_cell_name(cell_xz),
                            sudoku.get_cell_name(cell_yz),
                            (xyz - yz).single_value(),
                            (yz - &z).single_value(),
                            z_value,
                        ),
                        cell,
                        z_value,
                    );
                }
                return Some(step);
            }
        }
    }

    None
}
