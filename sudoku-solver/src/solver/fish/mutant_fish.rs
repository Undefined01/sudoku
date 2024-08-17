use super::fish_utils::check_is_fish;
use crate::solver::return_if_some;
use crate::solver::{Step, SudokuSolver, Technique};
use crate::sudoku::CellValue;
use crate::utils::{combinations, CellSet, CombinationOptions};

use std::cell::UnsafeCell;
use std::iter::FromIterator;

use arrayvec::ArrayVec;
use itertools::Itertools;

pub fn search_mutant_fish(sudoku: &SudokuSolver, size: usize, value: CellValue) -> Option<Step> {
    let all_houses = ArrayVec::<_, 27>::from_iter(
        sudoku
            .all_constraints()
            .iter()
            .map(|s| sudoku.get_possible_cells_for_house_and_value(s, value))
            .filter(|s| s.size() > 1),
    );

    if all_houses.is_empty() {
        return None;
    }

    let row_cells_stack = UnsafeCell::new((0u32, ArrayVec::<CellSet, 4>::new()));
    let ref mut on_selected = |pos: usize, element: usize| {
        let (used_cellset_set, row_cells_stack) = unsafe { &mut *row_cells_stack.get() };
        let cellset_index = all_houses[element].idx();
        let cellset = &**all_houses[element];
        if pos == 0 {
            row_cells_stack.push(cellset.clone());
        } else {
            // baseset 内部的 row 和 block 之间不能有相交的 candidate cell
            let union_of_previous = &row_cells_stack[pos - 1];
            if !(union_of_previous & cellset).is_empty() {
                return false;
            }
            row_cells_stack.push(union_of_previous | cellset);
        }
        *used_cellset_set |= 1 << cellset_index;
        true
    };
    let ref mut on_unselected = |pos: usize, element: usize| {
        let (used_cellset_set, row_cells_stack) = unsafe { &mut *row_cells_stack.get() };
        let cellset_index = all_houses[element].idx();
        row_cells_stack.pop().unwrap();
        *used_cellset_set &= !(1 << cellset_index);
    };
    let row_config = CombinationOptions {
        on_element_selected: Some(on_selected),
        on_element_unselected: Some(on_unselected),
    };

    for row_block_set in combinations(&all_houses, size, row_config) {
        let (used_cellset_set, row_cells_stack) = unsafe { &*row_cells_stack.get() };
        let row_block_cells = row_cells_stack.last().unwrap();

        let col_cells_stack = UnsafeCell::new(ArrayVec::<CellSet, 4>::new());
        let ref mut on_selected = |pos: usize, element: usize| {
            let col_cells_stack = unsafe { &mut *col_cells_stack.get() };
            let cellset_index = all_houses[element].idx();

            // coverset 使用的 block 和 baseset 不能重复，有重复时可以在 baseset 和 coverset 中去掉这个共同的 block 而形成一个更小的鱼
            if used_cellset_set & (1 << cellset_index) != 0 {
                return false;
            }

            let cellset = &**all_houses[element];
            if pos == 0 {
                col_cells_stack.push(cellset.clone());
            } else {
                // coverset 内部的 row 和 block 之间不能有相交的 candidate cell
                let union_of_previous = &col_cells_stack[pos - 1];
                if !(union_of_previous & cellset).is_empty() {
                    return false;
                }
                col_cells_stack.push(union_of_previous | cellset);
            }
            true
        };
        let ref mut on_unselected = |pos: usize, element: usize| {
            let col_cells_stack = unsafe { &mut *col_cells_stack.get() };
            col_cells_stack.pop().unwrap();
        };
        let col_config = CombinationOptions {
            on_element_selected: Some(on_selected),
            on_element_unselected: Some(on_unselected),
        };

        for col_block_set in combinations(&all_houses, size, col_config) {
            let col_cells_stack = unsafe { &*col_cells_stack.get() };
            let col_block_cells = col_cells_stack.last().unwrap();

            return_if_some!(check_is_fish(
                sudoku,
                row_block_set,
                col_block_set,
                &row_block_cells,
                &col_block_cells,
                value,
                Technique::MutantFish,
            ));
            return_if_some!(check_is_fish(
                sudoku,
                &col_block_set,
                &row_block_set,
                &col_block_cells,
                &row_block_cells,
                value,
                Technique::MutantFish,
            ));
        }
    }

    None
}
