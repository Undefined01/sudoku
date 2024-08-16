pub mod fish;
pub mod single_digit_patterns;
use crate::sudoku::{CellIndex, CellValue, Step, StepKind, StepRule, Sudoku};
use crate::utils::{CellSet, NamedCellSet};

use std::cell::OnceCell;
use std::collections::HashSet;

use itertools::Itertools;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SudokuSolver {
    sudoku: Sudoku,

    all_constraints: Vec<NamedCellSet>,
    constraints_of_cell: Vec<Vec<NamedCellSet>>,
    house_union_of_cell: Vec<CellSet>,
    block_id_of_cell: Vec<usize>,
    row_id_of_cell: Vec<usize>,
    column_id_of_cell: Vec<usize>,

    cells_in_rows: Vec<NamedCellSet>,
    cells_in_columns: Vec<NamedCellSet>,
    cells_in_blocks: Vec<NamedCellSet>,
    candidate_cells_in_rows: OnceCell<Vec<Vec<NamedCellSet>>>,
    candidate_cells_in_columns: OnceCell<Vec<Vec<NamedCellSet>>>,
    candidate_cells_in_blocks: OnceCell<Vec<Vec<NamedCellSet>>>,

    possible_positions_for_house_and_value: Vec<OnceCell<NamedCellSet>>,
}

macro_rules! return_if_some {
    ($x:expr) => {
        if let Some(x) = $x {
            return Some(x);
        }
    };
}

pub(crate) use return_if_some;

impl SudokuSolver {
    pub fn sudoku(&self) -> &Sudoku {
        &self.sudoku
    }

    pub(crate) fn cell_index(&self, row: usize, col: usize) -> CellIndex {
        self.sudoku.get_cell_position(row, col)
    }

    pub(crate) fn cell_value(&self, idx: CellIndex) -> Option<CellValue> {
        self.sudoku.get_cell_value(idx)
    }

    pub(crate) fn candidates(&self, idx: CellIndex) -> &Vec<CellValue> {
        self.sudoku.get_candidates(idx)
    }

    pub(crate) fn possible_cells(&self, value: CellValue) -> &CellSet {
        self.sudoku.get_possible_cells(value)
    }

    pub(crate) fn can_fill(&self, idx: CellIndex, value: CellValue) -> bool {
        self.sudoku.can_fill(idx, value)
    }

    pub(crate) fn all_constraints(&self) -> &[NamedCellSet] {
        &self.all_constraints
    }

    pub(crate) fn constraints_of_cell(&self, idx: CellIndex) -> &[NamedCellSet] {
        &self.constraints_of_cell[idx as usize]
    }

    pub(crate) fn house_union_of_cell(&self, idx: CellIndex) -> &CellSet {
        &self.house_union_of_cell[idx as usize]
    }

    pub(crate) fn block_id_of_cell(&self, idx: CellIndex) -> usize {
        self.block_id_of_cell[idx as usize]
    }

    pub(crate) fn cell_of_intersection(
        &self,
        house_1: &NamedCellSet,
        house_2: &NamedCellSet,
    ) -> CellIndex {
        let (row_idx, col_idx) = if house_1.idx() < 18 {
            debug_assert!(house_1.idx() >= 9);
            debug_assert!(house_2.idx() >= 18);
            debug_assert!(house_2.idx() < 27);
            (house_1.idx() - 9, house_2.idx() - 18)
        } else {
            debug_assert!(house_2.idx() >= 9);
            debug_assert!(house_1.idx() >= 18);
            debug_assert!(house_1.idx() < 27);
            (house_2.idx() - 9, house_1.idx() - 18)
        };
        return (row_idx * 9 + col_idx) as CellIndex;
    }

    pub(crate) fn row_id_of_cell(&self, idx: CellIndex) -> usize {
        self.row_id_of_cell[idx as usize]
    }

    pub(crate) fn column_id_of_cell(&self, idx: CellIndex) -> usize {
        self.column_id_of_cell[idx as usize]
    }

    pub(crate) fn cells_in_rows(&self) -> &[NamedCellSet] {
        &self.cells_in_rows
    }

    pub(crate) fn cells_in_columns(&self) -> &[NamedCellSet] {
        &self.cells_in_columns
    }

    pub(crate) fn cells_in_blocks(&self) -> &[NamedCellSet] {
        &self.cells_in_blocks
    }

    pub(crate) fn candidate_cells_in_rows(&self, value: CellValue) -> &[NamedCellSet] {
        &self.candidate_cells_in_rows.get_or_init(|| {
            (1..=9)
                .map(|value| {
                    self.cells_in_rows
                        .iter()
                        .map(|row| {
                            NamedCellSet::from_cellset(row, self.possible_cells(value) & row)
                        })
                        .collect()
                })
                .collect()
        })[value as usize - 1]
    }

    pub(crate) fn candidate_cells_in_columns(&self, value: CellValue) -> &[NamedCellSet] {
        &self.candidate_cells_in_columns.get_or_init(|| {
            (1..=9)
                .map(|value| {
                    self.cells_in_columns
                        .iter()
                        .map(|col| {
                            NamedCellSet::from_cellset(col, self.possible_cells(value) & col)
                        })
                        .collect()
                })
                .collect()
        })[value as usize - 1]
    }

    pub(crate) fn candidate_cells_in_blocks(&self, value: CellValue) -> &[NamedCellSet] {
        &self.candidate_cells_in_blocks.get_or_init(|| {
            (1..=9)
                .map(|value| {
                    self.cells_in_blocks
                        .iter()
                        .map(|block| {
                            NamedCellSet::from_cellset(block, self.possible_cells(value) & block)
                        })
                        .collect()
                })
                .collect()
        })[value as usize - 1]
    }

    pub(crate) fn get_possible_cells_for_house_and_value(
        &self,
        house: &NamedCellSet,
        value: CellValue,
    ) -> &NamedCellSet {
        debug_assert!(house.idx() < 27);
        debug_assert!(value >= 1 && value <= 9);
        let idx = house.idx() * 9 + value as usize - 1;
        self.possible_positions_for_house_and_value[idx]
            .get_or_init(|| NamedCellSet::from_cellset(house, self.possible_cells(value) & house))
    }

    pub(crate) fn get_cell_name(&self, idx: CellIndex) -> String {
        format!("r{}c{}", idx / 9 + 1, idx % 9 + 1)
    }

    pub(crate) fn get_cellset_string(&self, cellset: &CellSet) -> String {
        cellset.iter().map(|idx| self.get_cell_name(idx)).join(",")
    }
}

#[wasm_bindgen]
impl SudokuSolver {
    pub fn new(sudoku: Sudoku) -> Self {
        let mut all_constraints = vec![];
        let mut constraints_of_cell = (0..81).map(|_| vec![]).collect::<Vec<_>>();
        let mut house_union_of_cell = (0..81).map(|_| CellSet::new()).collect::<Vec<_>>();
        let mut block_id_of_cell = vec![0; 81];
        let mut row_id_of_cell = vec![0; 81];
        let mut column_id_of_cell = vec![0; 81];
        let mut cells_in_rows = vec![];
        let mut cells_in_columns = vec![];
        let mut cells_in_blocks = vec![];
        let possible_positions_for_house_and_value = vec![OnceCell::new(); 27 * 9];

        for block_x in (0..9).step_by(3) {
            for block_y in (0..9).step_by(3) {
                let mut block_set = NamedCellSet::new(
                    format!("b{}", block_x + block_y / 3 + 1),
                    block_x + block_y / 3,
                );
                for x in 0..3 {
                    for y in 0..3 {
                        let pos = sudoku.get_cell_position(block_x + x, block_y + y);
                        block_set.add(pos);
                        block_id_of_cell[pos as usize] = block_x + block_y / 3;
                    }
                }
                all_constraints.push(block_set.clone());
                cells_in_blocks.push(block_set);
            }
        }

        for row in 0..9 {
            let mut row_set = NamedCellSet::new(format!("r{}", row + 1), 9 + row);
            for col in 0..9 {
                let pos = sudoku.get_cell_position(row, col);
                row_set.add(pos);
                row_id_of_cell[pos as usize] = row;
            }
            all_constraints.push(row_set.clone());
            cells_in_rows.push(row_set);
        }

        for col in 0..9 {
            let mut col_set = NamedCellSet::new(format!("c{}", col + 1), 18 + col);
            for row in 0..9 {
                let pos = sudoku.get_cell_position(row, col);
                col_set.add(pos);
                column_id_of_cell[pos as usize] = col;
            }
            all_constraints.push(col_set.clone());
            cells_in_columns.push(col_set);
        }

        for row in 0..9 {
            for col in 0..9 {
                let pos = sudoku.get_cell_position(row, col) as usize;
                let block_x = row / 3;
                let block_y = col / 3;
                let block_idx = block_x * 3 + block_y;
                constraints_of_cell[pos].push(cells_in_rows[row].clone());
                constraints_of_cell[pos].push(cells_in_columns[col].clone());
                constraints_of_cell[pos].push(cells_in_blocks[block_idx].clone());
                house_union_of_cell[pos] =
                    &(&cells_in_rows[row] | &cells_in_columns[col]) | &cells_in_blocks[block_idx];
            }
        }

        SudokuSolver {
            sudoku,

            all_constraints,
            constraints_of_cell,
            house_union_of_cell,
            block_id_of_cell,
            row_id_of_cell,
            column_id_of_cell,

            cells_in_rows,
            cells_in_columns,
            cells_in_blocks,
            candidate_cells_in_rows: OnceCell::new(),
            candidate_cells_in_columns: OnceCell::new(),
            candidate_cells_in_blocks: OnceCell::new(),

            possible_positions_for_house_and_value,
        }
    }

    pub fn get_invalid_positions(&self) -> Vec<CellIndex> {
        let mut invalid_positions = vec![];
        for house in self.all_constraints.iter() {
            for (i, cell1) in house.iter().enumerate() {
                if self.cell_value(cell1).is_none() {
                    continue;
                }
                for cell2 in house.iter().take(i) {
                    if cell1 == cell2 {
                        invalid_positions.push(cell1);
                    }
                }
            }
        }
        invalid_positions
    }

    pub fn initialize_candidates(&mut self) {
        for cell in 0..81 {
            if self.cell_value(cell).is_none() {
                let mut candidates: HashSet<_> = (1..=9).collect();

                for constraint in self.constraints_of_cell(cell).iter() {
                    for other_cell in constraint.iter() {
                        if cell == other_cell {
                            continue;
                        }
                        if let Some(other_value) = self.cell_value(other_cell) {
                            candidates.remove(&other_value);
                        }
                    }
                }

                for &candidate in candidates.iter().sorted() {
                    self.sudoku.add_candidate(cell, candidate);
                }
            }
        }
    }

    pub fn apply_step(&mut self, step: &Step) {
        self.possible_positions_for_house_and_value = vec![OnceCell::new(); 27 * 9];
        self.candidate_cells_in_rows.take();
        self.candidate_cells_in_columns.take();
        self.candidate_cells_in_blocks.take();
        match step.kind {
            StepKind::ValueSet => {
                for position in step.positions.iter() {
                    self.sudoku.fill(position.cell_index, position.value);
                    for cell in self.house_union_of_cell[position.cell_index as usize].iter() {
                        if cell == position.cell_index {
                            continue;
                        }
                        self.sudoku.remove_candidate(cell, position.value);
                    }
                }
            }
            StepKind::CandidateEliminated => {
                for position in step.positions.iter() {
                    self.sudoku
                        .remove_candidate(position.cell_index, position.value);
                }
            }
        }
    }

    pub fn is_completed(&self) -> bool {
        for cell in 0..81 {
            if self.cell_value(cell).is_none() {
                return false;
            }
        }
        true
    }

    pub fn solve_full_house(&self) -> Option<Step> {
        for house in self.all_constraints.iter() {
            let unfilled_cells_count = house
                .iter()
                .filter(|&cell| self.cell_value(cell).is_none())
                .count();
            if unfilled_cells_count == 1 {
                let unfilled_cell = house
                    .iter()
                    .filter(|&cell| self.cell_value(cell).is_none())
                    .next()
                    .unwrap();
                let missing_value = self
                    .candidates(unfilled_cell)
                    .iter()
                    .cloned()
                    .next()
                    .unwrap();
                return Some(Step::new_value_set(
                    StepRule::FullHouse,
                    format!(
                        "{} is the only missing cell in {}",
                        self.get_cell_name(unfilled_cell),
                        house.name()
                    ),
                    unfilled_cell,
                    missing_value,
                ));
            }
        }
        None
    }

    pub fn solve_naked_single(&self) -> Option<Step> {
        for house in self.all_constraints.iter() {
            for cell in house.iter() {
                if self.candidates(cell).len() == 1 {
                    let &value = self.candidates(cell).iter().next().unwrap();
                    return Some(Step::new_value_set(
                        StepRule::NakedSingle,
                        format!(
                            "{} is the only possible value to fill {}",
                            value,
                            self.get_cell_name(cell)
                        ),
                        cell,
                        value,
                    ));
                }
            }
        }
        None
    }

    pub fn solve_hidden_single(&self) -> Option<Step> {
        for house in self.all_constraints.iter() {
            for value in 1..=9 {
                let possible_cells = self.get_possible_cells_for_house_and_value(house, value);
                if possible_cells.size() == 1 {
                    let target_cell = possible_cells.iter().next().unwrap();
                    return Some(Step::new_value_set(
                        StepRule::HiddenSingle,
                        format!(
                            "in {}, {} is the only possible cell that can be {}",
                            house.name(),
                            self.get_cell_name(target_cell),
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

    // 当 House A 中的一个数字只出现在 House A & House B （A 和 B的交集）中时，这个数字不可能再出现在 House B 中的其他单元格中
    pub fn solve_locked_candidates(&self) -> Option<Step> {
        let check = |house_a: &NamedCellSet, house_b: &NamedCellSet| -> Option<Step> {
            let intersection = house_a & house_b;
            if intersection.is_empty() {
                return None;
            }

            let mut step = Step::new(StepKind::CandidateEliminated, StepRule::LockedCandidates);

            for value in 1..=9 {
                let possible_cells_in_a =
                    self.get_possible_cells_for_house_and_value(house_a, value);
                if possible_cells_in_a.is_empty()
                    || !possible_cells_in_a.is_subset_of(&intersection)
                {
                    continue;
                }
                for cell in house_b.iter() {
                    if intersection.has(cell) {
                        continue;
                    }
                    if self.can_fill(cell, value) {
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

        for block in &self.cells_in_blocks {
            for row in &self.cells_in_rows {
                let step = check(block, row);
                if step.is_some() {
                    return step;
                }
                let step = check(row, block);
                if step.is_some() {
                    return step;
                }
            }
            for column in &self.cells_in_columns {
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

    // 在一个 House 中，若任意 n 个数字只可能出现在相同 n 个（或更少）单元格中，则这 n 个单元格中不可能出现其他数字
    pub fn solve_hidden_subset(&self) -> Option<Step> {
        let mut step = Step::new(StepKind::CandidateEliminated, StepRule::HiddenSubset);

        for house in self.all_constraints.iter() {
            let mut possible_cells_in_houses = vec![];
            for value in 1..=9 {
                let possible_cells_in_house =
                    self.get_possible_cells_for_house_and_value(house, value);
                if !possible_cells_in_house.is_empty() {
                    possible_cells_in_houses.push((value, possible_cells_in_house));
                }
            }

            for size in 2..=4 {
                let possible_house_cells_for_candidate_in_size: Vec<_> = possible_cells_in_houses
                    .iter()
                    .filter(|(_, cells)| cells.size() <= size)
                    .collect();

                if possible_house_cells_for_candidate_in_size.len() < size {
                    continue;
                }

                for subset in possible_house_cells_for_candidate_in_size
                    .iter()
                    .combinations(size)
                {
                    let cell_union =
                        CellSet::union_multiple(subset.iter().map(|(_, cells)| &***cells));
                    let values_in_subset: HashSet<_> =
                        subset.iter().map(|(value, _)| *value).collect();

                    if cell_union.size() <= size {
                        for cell in cell_union.iter() {
                            for value in 1..=9 {
                                if !values_in_subset.contains(&value) && self.can_fill(cell, value)
                                {
                                    step.add(
                                        format!(
                                            "in {}, {} only appears in {}",
                                            house.name(),
                                            values_in_subset.iter().sorted().join(","),
                                            self.get_cellset_string(&cell_union),
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
    pub fn solve_naked_subset(&self) -> Option<Step> {
        for house in self.all_constraints.iter() {
            for size in 2..=4 {
                let mut step = Step::new(StepKind::CandidateEliminated, StepRule::NakedSubset);

                for subset in house
                    .iter()
                    .filter(|&cell| {
                        !self.candidates(cell).is_empty() && self.candidates(cell).len() <= size
                    })
                    .combinations(size)
                {
                    let value_union: HashSet<_> = subset
                        .iter()
                        .flat_map(|&cell| self.candidates(cell).iter().copied())
                        .collect();
                    let cells_in_subset = CellSet::from_cells(subset);

                    if value_union.len() > size {
                        continue;
                    }

                    for cell in house.iter() {
                        if cells_in_subset.has(cell) {
                            continue;
                        }
                        for &value in value_union.iter().sorted() {
                            if self.can_fill(cell, value) {
                                step.add(
                                    format!(
                                        "in {}, {} only contains {}",
                                        house.name(),
                                        self.get_cellset_string(&cells_in_subset),
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

    pub fn solve_one_step(&self) -> Option<Step> {
        let solving_techniques = [
            // SudokuSolver::solve_full_house,
            SudokuSolver::solve_naked_single,
            SudokuSolver::solve_hidden_single,
            SudokuSolver::solve_locked_candidates,
            SudokuSolver::solve_hidden_subset,
            SudokuSolver::solve_naked_subset,
            fish::solve_basic_fish,
            fish::solve_finned_fish,
            fish::solve_franken_fish,
            // fish::solve_mutant_fish,
        ];
        for technique in solving_techniques.iter() {
            if let Some(step) = technique(self) {
                return Some(step);
            }
        }
        None
    }
}
