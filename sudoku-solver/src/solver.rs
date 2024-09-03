mod chain;
mod fish;
mod intersection;
mod single;
mod single_digit_patterns;
mod subset;
mod wing;
mod guess;

use crate::sudoku::{CellIndex, CellValue, Sudoku};
use crate::utils::{CellSet, NamedCellSet, ValueSet};

use std::cell::OnceCell;
use std::collections::HashSet;
use std::fmt::Display;

use arrayvec::ArrayVec;
use itertools::Itertools;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SudokuSolver {
    sudoku: Sudoku,

    all_constraints: Vec<NamedCellSet>,
    constraints_of_cell: Vec<Vec<NamedCellSet>>,
    house_union_of_cell: Vec<CellSet>,

    filled_cells: CellSet,
    unfilled_cells: CellSet,

    cells_in_rows: Vec<NamedCellSet>,
    cells_in_columns: Vec<NamedCellSet>,
    cells_in_blocks: Vec<NamedCellSet>,
    candidate_cells_in_rows: OnceCell<Vec<Vec<NamedCellSet>>>,
    candidate_cells_in_columns: OnceCell<Vec<Vec<NamedCellSet>>>,
    candidate_cells_in_blocks: OnceCell<Vec<Vec<NamedCellSet>>>,

    rows_with_only_two_possible_places: Vec<
        OnceCell<
            ArrayVec<
                (
                    NamedCellSet,
                    (usize, usize, CellIndex),
                    (usize, usize, CellIndex),
                ),
                9,
            >,
        >,
    >,
    cols_with_only_two_possible_places: Vec<
        OnceCell<
            ArrayVec<
                (
                    NamedCellSet,
                    (usize, usize, CellIndex),
                    (usize, usize, CellIndex),
                ),
                9,
            >,
        >,
    >,

    possible_positions_for_house_and_value: Vec<OnceCell<NamedCellSet>>,
}

macro_rules! return_if_some {
    ($x:expr) => {
        if let Some(x) = $x {
            return Some(x);
        }
    };
}
macro_rules! return_in_fast_mode {
    ($x:expr) => {
        if $x.should_return() {
            return;
        }
    };
}

pub(crate) use return_if_some;
pub(crate) use return_in_fast_mode;

impl SudokuSolver {
    pub fn sudoku(&self) -> &Sudoku {
        &self.sudoku
    }

    pub(crate) fn cells(&self) -> impl Iterator<Item = CellIndex> {
        (0..81).map(|x| x as CellIndex)
    }

    pub(crate) fn cell_index(&self, row: usize, col: usize) -> CellIndex {
        self.sudoku.get_cell_position(row, col)
    }

    pub(crate) fn cell_position(&self, cell: CellIndex) -> (usize, usize, usize) {
        (
            cell as usize / 9,
            cell as usize % 9,
            (cell / 27 * 3 + cell % 9 / 3) as usize,
        )
    }

    pub(crate) fn cell_value(&self, idx: CellIndex) -> Option<CellValue> {
        self.sudoku.get_cell_value(idx)
    }

    pub(crate) fn candidates(&self, idx: CellIndex) -> &ValueSet {
        self.sudoku.get_candidates(idx)
    }

    pub(crate) fn possible_cells(&self, value: CellValue) -> &CellSet {
        self.sudoku.get_possible_cells(value)
    }

    pub(crate) fn unfilled_cells(&self) -> &CellSet {
        &self.unfilled_cells
    }

    pub(crate) fn filled_cells(&self) -> &CellSet {
        &self.filled_cells
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

    pub(crate) fn house_union_of_cell<'a, 'b>(&'a self, idx: CellIndex) -> &'b CellSet {
        // To prevent the return value holding the reference to self, we use unsafe code here.
        // SAFETY: house_union_of_cell is initialized and never changed
        unsafe { &*(&self.house_union_of_cell[idx as usize] as *const _) }
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

    pub(crate) fn rows_with_only_two_possible_places(
        &self,
        value: CellValue,
    ) -> &[(
        NamedCellSet,
        (usize, usize, CellIndex),
        (usize, usize, CellIndex),
    )] {
        self.rows_with_only_two_possible_places[value as usize - 1].get_or_init(|| {
            ArrayVec::<_, 9>::from_iter(
                self.candidate_cells_in_rows(value)
                    .iter()
                    .filter(|row| row.size() == 2)
                    .map(|row| {
                        let cell_ids = ArrayVec::<_, 2>::from_iter(row.iter());
                        let pos = ArrayVec::<_, 2>::from_iter(
                            cell_ids.iter().map(|&cell| self.cell_position(cell)),
                        );
                        (
                            row.clone(),
                            (pos[0].1, pos[0].2, cell_ids[0]),
                            (pos[1].1, pos[1].2, cell_ids[1]),
                        )
                    }),
            )
        })
    }

    pub(crate) fn cols_with_only_two_possible_places(
        &self,
        value: CellValue,
    ) -> &[(
        NamedCellSet,
        (usize, usize, CellIndex),
        (usize, usize, CellIndex),
    )] {
        self.cols_with_only_two_possible_places[value as usize - 1].get_or_init(|| {
            ArrayVec::<_, 9>::from_iter(
                self.candidate_cells_in_columns(value)
                    .iter()
                    .filter(|col| col.size() == 2)
                    .map(|col| {
                        let cell_ids = ArrayVec::<_, 2>::from_iter(col.iter());
                        let pos = ArrayVec::<_, 2>::from_iter(
                            cell_ids.iter().map(|&cell| self.cell_position(cell)),
                        );
                        (
                            col.clone(),
                            (pos[0].0, pos[0].2, cell_ids[0]),
                            (pos[1].0, pos[1].2, cell_ids[1]),
                        )
                    }),
            )
        })
    }

    pub(crate) fn get_possible_cells_for_house_and_value(
        &self,
        house: &NamedCellSet,
        value: CellValue,
    ) -> &NamedCellSet {
        debug_assert!(house.idx() < 27);
        debug_assert!(value >= 1 && value <= 9);
        let idx = house.idx() * 9 + value as usize - 1;
        debug_assert!(
            self.possible_positions_for_house_and_value[idx]
                .get()
                .map_or(true, |x| x
                    == &NamedCellSet::from_cellset(
                        house,
                        self.possible_cells(value) & house
                    )),
            "idx: {}, house: {}, value: {}",
            idx,
            house.idx(),
            value
        );
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
        let mut cells_in_rows = vec![];
        let mut cells_in_columns = vec![];
        let mut cells_in_blocks = vec![];
        let possible_positions_for_house_and_value = vec![OnceCell::new(); 27 * 9];

        let filled_cells = CellSet::from_iter(
            (0..81)
                .filter(|&cell| sudoku.get_cell_value(cell).is_some())
                .collect_vec(),
        );
        let unfilled_cells = CellSet::from_iter(
            (0..81)
                .filter(|&cell| sudoku.get_cell_value(cell).is_none())
                .collect_vec(),
        );

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
            }
            all_constraints.push(row_set.clone());
            cells_in_rows.push(row_set);
        }

        for col in 0..9 {
            let mut col_set = NamedCellSet::new(format!("c{}", col + 1), 18 + col);
            for row in 0..9 {
                let pos = sudoku.get_cell_position(row, col);
                col_set.add(pos);
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
                house_union_of_cell[pos].remove(pos as CellIndex);
            }
        }

        SudokuSolver {
            sudoku,

            all_constraints,
            constraints_of_cell,
            house_union_of_cell,

            filled_cells,
            unfilled_cells,

            cells_in_rows,
            cells_in_columns,
            cells_in_blocks,
            candidate_cells_in_rows: OnceCell::new(),
            candidate_cells_in_columns: OnceCell::new(),
            candidate_cells_in_blocks: OnceCell::new(),

            rows_with_only_two_possible_places: vec![OnceCell::new(); 9],
            cols_with_only_two_possible_places: vec![OnceCell::new(); 9],

            possible_positions_for_house_and_value,
        }
    }

    pub fn take_sudoku(&self) -> Sudoku {
        self.sudoku.clone()
    }

    pub fn get_invalid_positions(&self) -> Vec<CellIndex> {
        let mut invalid_positions = vec![];
        for house in self.all_constraints.iter() {
            for (i, cell1) in house.iter().enumerate() {
                if self.cell_value(cell1).is_none() {
                    if self.candidates(cell1).size() == 0 || self.candidates(cell1).size() > 9 {
                        invalid_positions.push(cell1);
                        debug_assert!(false, "Invalid candidates for cell {}", cell1);
                    }
                    for value in self.candidates(cell1).iter() {
                        if !self.possible_cells(value).has(cell1) {
                            invalid_positions.push(cell1);
                            debug_assert!(false, "Invalid candidates for cell {}", cell1);
                        }
                        if !self
                            .get_possible_cells_for_house_and_value(house, value)
                            .has(cell1)
                        {
                            invalid_positions.push(cell1);
                            debug_assert!(false, "Invalid candidates for cell {}", cell1);
                        }
                    }
                    continue;
                }
                for cell2 in house.iter().take(i) {
                    if self.cell_value(cell1) == self.cell_value(cell2) {
                        invalid_positions.push(cell1);
                        debug_assert!(
                            false,
                            "Invalid candidates for cell {} {} {:?}",
                            self.get_cell_name(cell1),
                            self.get_cell_name(cell2),
                            self.cell_value(cell2)
                        );
                    }
                }
            }

            for value in 1..=9 {
                for cell in self
                    .get_possible_cells_for_house_and_value(house, value)
                    .iter()
                {
                    if !self.candidates(cell).has(value) {
                        invalid_positions.push(cell);
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

    pub fn apply_step(&mut self, step: &SolutionRecorder) {
        self.candidate_cells_in_rows.take();
        self.candidate_cells_in_columns.take();
        self.candidate_cells_in_blocks.take();
        self.rows_with_only_two_possible_places
            .iter_mut()
            .for_each(|x| {
                x.take();
            });
        self.rows_with_only_two_possible_places
            .iter_mut()
            .for_each(|x| {
                x.take();
            });

        let reset_possible_positions_for_cell = |this: &mut SudokuSolver, cell: CellIndex| {
            let (row, col, block) = this.cell_position(cell);
            let row_set = this.cells_in_rows()[row].idx();
            let col_set = this.cells_in_columns()[col].idx();
            let block_set = this.cells_in_blocks()[block].idx();
            for value in 1..=9 {
                let value_idx = value as usize - 1;
                this.possible_positions_for_house_and_value[row_set * 9 + value_idx].take();
                this.possible_positions_for_house_and_value[col_set * 9 + value_idx].take();
                this.possible_positions_for_house_and_value[block_set * 9 + value_idx].take();
            }
        };

        let remove_candidate = |this: &mut SudokuSolver, cell: CellIndex, value: CellValue| {
            if !this.sudoku.can_fill(cell, value) {
                return;
            }
            this.sudoku.remove_candidate(cell, value);
            let (row, col, block) = this.cell_position(cell);
            let row_set = this.cells_in_rows()[row].idx();
            let col_set = this.cells_in_columns()[col].idx();
            let block_set = this.cells_in_blocks()[block].idx();
            let value_idx = value as usize - 1;
            this.possible_positions_for_house_and_value[row_set * 9 + value_idx].take();
            this.possible_positions_for_house_and_value[col_set * 9 + value_idx].take();
            this.possible_positions_for_house_and_value[block_set * 9 + value_idx].take();
        };
        for position in step.steps.iter() {
            match position.kind {
                StepKind::ValueSet => {
                    let value = position.value;
                    reset_possible_positions_for_cell(self, position.cell_index);
                    self.sudoku.fill(position.cell_index, value);
                    self.filled_cells.add(position.cell_index);
                    self.unfilled_cells.remove(position.cell_index);
                    for cell in self.house_union_of_cell(position.cell_index).iter() {
                        remove_candidate(self, cell, position.value);
                    }
                }
                StepKind::CandidateEliminated => {
                    for position in step.steps.iter() {
                        remove_candidate(self, position.cell_index, position.value);
                    }
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

    pub fn solve_one_step(&self, techniques: &Techniques) -> Option<SolutionRecorder> {
        let mut solution = SolutionRecorder::new();
        for technique in techniques.0.iter() {
            technique(self, &mut solution);
            if solution.should_return() {
                break;
            }
        }
        if solution.is_empty() {
            return None;
        }
        return Some(solution);
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct SolutionRecorder {
    /// If fast_mode is true, the solver will return as soon as a new step is added.
    fast_mode: bool,
    new_step_start_idx: usize,
    pub steps: Vec<Step>,
}

#[wasm_bindgen]
impl SolutionRecorder {
    pub fn new() -> Self {
        Self {
            fast_mode: true,
            new_step_start_idx: 0,
            steps: vec![],
        }
    }

    pub fn reset_new_step(&mut self) {
        self.new_step_start_idx = self.steps.len();
    }

    pub fn has_new_step(&self) -> bool {
        self.new_step_start_idx < self.steps.len()
    }

    pub(crate) fn should_return(&self) -> bool {
        self.fast_mode && self.new_step_start_idx < self.steps.len()
    }

    pub(crate) fn add_value_set(
        &mut self,
        technique: Technique,
        reason: String,
        cell_index: CellIndex,
        value: CellValue,
    ) {
        self.steps.push(Step {
            kind: StepKind::ValueSet,
            technique,
            reason,
            cell_index,
            value,
        });
    }

    pub(crate) fn add_elimination(
        &mut self,
        technique: Technique,
        reason: String,
        cell_index: CellIndex,
        value: CellValue,
    ) {
        self.steps.push(Step {
            kind: StepKind::CandidateEliminated,
            technique: technique,
            reason,
            cell_index,
            value,
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn to_string(&self, sudoku: &Sudoku) -> String {
        let mut f = String::new();
        use std::fmt::Write;
        for position in self.steps.iter() {
            match position.kind {
                StepKind::ValueSet => {
                    write!(
                        f,
                        "[{:?}] {} => {}={}\n",
                        position.technique,
                        position.reason,
                        sudoku.get_cell_name(position.cell_index),
                        position.value,
                    )
                    .unwrap();
                }
                StepKind::CandidateEliminated => {
                    write!(
                        f,
                        "[{:?}] {} => {}<>{}\n",
                        position.technique,
                        position.reason,
                        sudoku.get_cell_name(position.cell_index),
                        position.value,
                    )
                    .unwrap();
                }
            }
        }
        f
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Step {
    pub kind: StepKind,
    pub technique: Technique,
    pub reason: String,
    pub cell_index: CellIndex,
    pub value: CellValue,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum StepKind {
    ValueSet,
    CandidateEliminated,
}

pub type SolverFn = fn(sudoku: &SudokuSolver, solution: &mut SolutionRecorder);

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub enum Technique {
    // Single
    FullHouse,
    NakedSingle,
    HiddenSingle,

    // Intersection
    LockedCandidates,

    // Subset
    HiddenSubset,
    NakedSubset,

    // Fish
    BasicFish,
    FinnedFish,
    FrankenFish,
    MutantFish,

    // Single digit patterns
    TwoStringKite,
    Skyscraper,
    RectangleElimination,

    // Wing
    WWing,
    XYWing,
    XYZWing,

    // Chain
    ForcedChain,

    Guess,
}

impl Technique {
    pub fn solver_fn(&self) -> SolverFn {
        match self {
            Technique::FullHouse => single::solve_full_house,
            Technique::NakedSingle => single::solve_naked_single,
            Technique::HiddenSingle => single::solve_hidden_single,
            Technique::LockedCandidates => intersection::solve_locked_candidates,
            Technique::HiddenSubset => subset::solve_hidden_subset,
            Technique::NakedSubset => subset::solve_naked_subset,
            Technique::BasicFish => fish::solve_basic_fish,
            Technique::FinnedFish => fish::solve_finned_fish,
            Technique::FrankenFish => fish::solve_franken_fish,
            Technique::MutantFish => fish::solve_mutant_fish,
            Technique::TwoStringKite => single_digit_patterns::solve_two_string_kite,
            Technique::Skyscraper => single_digit_patterns::solve_skyscraper,
            Technique::RectangleElimination => single_digit_patterns::solve_rectangle_elimination,
            Technique::WWing => wing::solve_w_wing,
            Technique::XYWing => wing::solve_xy_wing,
            Technique::XYZWing => wing::solve_xyz_wing,
            Technique::ForcedChain => chain::solve_forced_chain,
            Technique::Guess => guess::solve_dancing_links,
        }
    }
}

impl<S: AsRef<str> + Display> From<S> for Technique {
    fn from(name: S) -> Self {
        match name.as_ref() {
            "FullHouse" => Technique::FullHouse,
            "full_house" => Technique::FullHouse,
            "NakedSingle" => Technique::NakedSingle,
            "naked_single" => Technique::NakedSingle,
            "HiddenSingle" => Technique::HiddenSingle,
            "hidden_single" => Technique::HiddenSingle,

            "LockedCandidates" => Technique::LockedCandidates,
            "locked_candidates" => Technique::LockedCandidates,

            "HiddenSubset" => Technique::HiddenSubset,
            "hidden_subset" => Technique::HiddenSubset,
            "NakedSubset" => Technique::NakedSubset,
            "naked_subset" => Technique::NakedSubset,

            "BasicFish" => Technique::BasicFish,
            "basic_fish" => Technique::BasicFish,
            "FinnedFish" => Technique::FinnedFish,
            "finned_fish" => Technique::FinnedFish,
            "FrankenFish" => Technique::FrankenFish,
            "franken_fish" => Technique::FrankenFish,
            "MutantFish" => Technique::MutantFish,
            "mutant_fish" => Technique::MutantFish,

            "TwoStringKite" => Technique::TwoStringKite,
            "two_string_kite" => Technique::TwoStringKite,
            "Skyscraper" => Technique::Skyscraper,
            "skyscraper" => Technique::Skyscraper,
            "RectangleElimination" => Technique::RectangleElimination,
            "rectangle_elimination" => Technique::RectangleElimination,

            "WWing" => Technique::WWing,
            "w_wing" => Technique::WWing,
            "XYWing" => Technique::XYWing,
            "xy_wing" => Technique::XYWing,
            "XYZWing" => Technique::XYZWing,
            "xyz_wing" => Technique::XYZWing,

            "ForcedChain" => Technique::ForcedChain,
            "forced_chain" => Technique::ForcedChain,

            _ => panic!("Unknown technique: {}", name),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Techniques(Vec<fn(sudoku: &SudokuSolver, solution: &mut SolutionRecorder)>);

impl Techniques {
    pub fn new() -> Self {
        let default_techniques = [
            Technique::NakedSingle,
            Technique::HiddenSingle,
            Technique::LockedCandidates,
            Technique::HiddenSubset,
            Technique::NakedSubset,
            Technique::TwoStringKite,
            Technique::Skyscraper,
            Technique::RectangleElimination,
            Technique::WWing,
            Technique::XYWing,
            Technique::XYZWing,
            Technique::BasicFish,
            Technique::FinnedFish,
            Technique::FrankenFish,
        ];
        Self::from(default_techniques.into_iter())
    }

    pub fn from(techniques: impl Iterator<Item = impl Into<Technique>>) -> Self {
        let mut funcs: Vec<SolverFn> = vec![];
        for technique in techniques {
            funcs.push(technique.into().solver_fn());
        }
        Self(funcs)
    }
}

#[wasm_bindgen]
impl Techniques {
    pub fn default_techniques() -> Self {
        Self::new()
    }

    pub fn from_slice(techniques: Vec<Technique>) -> Self {
        let mut funcs: Vec<SolverFn> = vec![];
        for technique in techniques {
            funcs.push(technique.solver_fn());
        }
        Self(funcs)
    }
}
