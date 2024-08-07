mod solver;
mod utils;

pub use solver::SudokuSolver;
use utils::CellSet;

use itertools::Itertools;
use std::fmt::Debug;
use wasm_bindgen::prelude::*;

const UNSET: u8 = 255;

type CellIndex = u8;
type CellValue = u8;

#[wasm_bindgen]
pub struct Sudoku {
    board: Vec<CellValue>,
    // cell position -> possible values at that cell
    candidates: Vec<Vec<CellIndex>>,
    // value -> possible cell positions for that value
    possible_positions: Vec<CellSet>,
}

#[wasm_bindgen]
impl Sudoku {
    pub(crate) fn get_candidates(&self, idx: CellIndex) -> &Vec<CellValue> {
        &self.candidates[idx as usize]
    }

    pub(crate) fn add_candidate(&mut self, idx: CellIndex, value: CellValue) {
        self.candidates[idx as usize].push(value);
        self.possible_positions[value as usize].add(idx);
    }

    pub(crate) fn remove_candidate(&mut self, idx: CellIndex, value: CellValue) {
        self.candidates[idx as usize].retain(|&x| x != value);
        self.possible_positions[value as usize].delete(idx);
    }

    pub(crate) fn can_fill(&self, idx: CellIndex, value: CellValue) -> bool {
        self.possible_positions[value as usize].has(idx)
    }

    pub(crate) fn get_possible_cells(&self, value: CellValue) -> &CellSet {
        &self.possible_positions[value as usize]
    }

    pub(crate) fn get_cell_value(&self, idx: CellIndex) -> CellValue {
        self.board[idx as usize]
    }

    pub(crate) fn get_cell_position(&self, row: usize, col: usize) -> CellIndex {
        (row * 9 + col) as u8
    }

    pub(crate) fn get_cell_name(&self, idx: CellIndex) -> String {
        format!("r{}c{}", idx / 9 + 1, idx % 9 + 1)
    }

    pub fn from_str(str: &str) -> Self {
        let mut board = Vec::with_capacity(81);
        for ch in str.chars() {
            if ch.is_digit(10) {
                let digit = ch.to_digit(10).unwrap() as u8;
                board.push(digit);
            } else if ch == '.' || ch == '_' {
                board.push(UNSET);
            }
        }
        let candidates = vec![vec![]; 81];
        let possible_positions = vec![CellSet::new(); 10];
        Self {
            board,
            candidates,
            possible_positions,
        }
    }

    pub fn to_value_string(&self) -> String {
        let mut s = String::new();
        for row in 0..9 {
            for col in 0..9 {
                let idx = self.get_cell_position(row, col);
                let value = self.get_cell_value(idx);
                if value == UNSET {
                    s.push('.');
                } else {
                    s.push_str(&value.to_string());
                }
            }
        }
        s
    }

    pub fn to_candidate_string(&self) -> String {
        let candidates = self
            .candidates
            .iter()
            .enumerate()
            .map(|(idx, candidates)| {
                if candidates.is_empty() {
                    return format!("{}", self.board[idx]);
                }
                return candidates.iter().map(|&x| x.to_string()).join("");
            })
            .collect_vec();

        let mut s = String::new();
        let col_widths = (0..9)
            .map(|col| {
                (0..9)
                    .map(|row| {
                        let idx = self.get_cell_position(row, col);
                        candidates[idx as usize].len()
                    })
                    .max()
                    .unwrap()
                    + 1
            })
            .collect_vec();

        let push_horizontal_line = |s: &mut String| {
            s.push('+');
            for col in 0..9 {
                for _ in 0..col_widths[col] {
                    s.push('-');
                }
                if col % 3 == 2 {
                    s.push_str("-+");
                }
            }
            s.push('\n');
        };

        push_horizontal_line(&mut s);
        for row in 0..9 {
            s.push('|');
            for col in 0..9 {
                let idx = self.get_cell_position(row, col);
                for _ in 0..col_widths[col] - candidates[idx as usize].len() {
                    s.push(' ');
                }
                s.push_str(&candidates[idx as usize]);
                if col % 3 == 2 {
                    s.push_str(" |");
                }
            }
            s.push('\n');
            if row % 3 == 2 {
                push_horizontal_line(&mut s);
            }
        }
        s
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct Step {
    pub kind: StepKind,
    pub rule: StepRule,
    pub positions: Vec<StepPosition>,
}

#[wasm_bindgen(getter_with_clone)]
impl Step {
    pub(crate) fn new(kind: StepKind, rule: StepRule) -> Self {
        Self {
            kind,
            rule,
            positions: vec![],
        }
    }

    pub(crate) fn new_value_set(
        rule: StepRule,
        reason: String,
        position: CellIndex,
        value: CellValue,
    ) -> Self {
        let mut step = Self::new(StepKind::ValueSet, rule);
        step.add(reason, position, value);
        step
    }

    pub(crate) fn add(&mut self, reason: String, cell_index: CellIndex, value: CellValue) {
        self.positions.push(StepPosition {
            reason,
            cell_index,
            value,
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    pub fn to_string(&self, sudoku: &Sudoku) -> String {
        let mut f = String::new();
        use std::fmt::Write;
        match self.kind {
            StepKind::ValueSet => {
                for position in self.positions.iter() {
                    write!(
                        f,
                        "[{:?}] {} => {}={}\n",
                        self.rule,
                        position.reason,
                        sudoku.get_cell_name(position.cell_index),
                        position.value
                    )
                    .unwrap();
                }
            }
            StepKind::CandidateEliminated => {
                for position in self.positions.iter() {
                    write!(
                        f,
                        "[{:?}] {} => {}<>{}\n",
                        self.rule,
                        position.reason,
                        sudoku.get_cell_name(position.cell_index),
                        position.value
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
pub struct StepPosition {
    pub reason: String,
    pub cell_index: CellIndex,
    pub value: CellValue,
}

impl StepPosition {
    pub(crate) fn new(reason: String, cell_index: CellIndex, value: CellValue) -> Self {
        Self {
            reason,
            cell_index,
            value,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub enum StepKind {
    ValueSet,
    CandidateEliminated,
}

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub enum StepRule {
    FullHouse,
    NakedSingle,
    HiddenSingle,
    LockedCandidates,
    HiddenSubset,
    NakedSubset,
    BasicFish,
    FinnedFish,
    ComplexFish,
}
