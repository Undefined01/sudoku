use crate::utils::{CellSet, ValueSet};

use itertools::Itertools;
use wasm_bindgen::prelude::*;

pub type CellIndex = u8;
pub type CellValue = u8;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Sudoku {
    board: Vec<Option<CellValue>>,
    // cell position -> possible values at that cell
    candidates: Vec<ValueSet>,
    // value -> possible cell positions for that value
    possible_positions: Vec<CellSet>,
}

#[wasm_bindgen]
impl Sudoku {
    pub(crate) fn get_candidates(&self, idx: CellIndex) -> &ValueSet {
        &self.candidates[idx as usize]
    }

    pub(crate) fn add_candidate(&mut self, idx: CellIndex, value: CellValue) {
        self.candidates[idx as usize].add(value);
        self.possible_positions[value as usize].add(idx);
    }

    pub(crate) fn remove_candidate(&mut self, idx: CellIndex, value: CellValue) {
        self.candidates[idx as usize].delete(value);
        self.possible_positions[value as usize].remove(idx);
    }

    pub(crate) fn can_fill(&self, idx: CellIndex, value: CellValue) -> bool {
        self.possible_positions[value as usize].has(idx)
    }

    pub(crate) fn fill(&mut self, idx: CellIndex, value: CellValue) {
        self.board[idx as usize] = Some(value);
        for candidate in self.candidates[idx as usize].iter() {
            self.possible_positions[candidate as usize].remove(idx);
        }
        self.candidates[idx as usize].clear();
    }

    pub(crate) fn get_possible_cells(&self, value: CellValue) -> &CellSet {
        &self.possible_positions[value as usize]
    }

    pub(crate) fn get_cell_value(&self, idx: CellIndex) -> Option<CellValue> {
        self.board[idx as usize]
    }

    pub(crate) fn get_cell_position(&self, row: usize, col: usize) -> CellIndex {
        (row * 9 + col) as u8
    }

    pub(crate) fn get_cell_name(&self, idx: CellIndex) -> String {
        format!("r{}c{}", idx / 9 + 1, idx % 9 + 1)
    }

    pub fn from_values(str: &str) -> Self {
        let mut board = Vec::with_capacity(81);
        for ch in str.chars() {
            if ch.is_digit(10) {
                let digit = ch.to_digit(10).unwrap() as u8;
                board.push(Some(digit));
            } else if ch == '.' || ch == '_' {
                board.push(None);
            }
        }
        let candidates = vec![ValueSet::new(); 81];
        let possible_positions = vec![CellSet::new(); 10];
        Self {
            board,
            candidates,
            possible_positions,
        }
    }

    pub fn from_candidates(str: &str) -> Self {
        let mut board = vec![None; 81];
        let mut candidates = vec![ValueSet::new(); 81];
        let mut possible_positions = vec![CellSet::new(); 10];
        let mut chars = str.chars();
        let mut idx = 0;
        let mut waiting_next_digit = false;
        while let Some(ch) = chars.next() {
            if ch.is_digit(10) {
                waiting_next_digit = true;
                let digit = ch.to_digit(10).unwrap() as CellValue;
                candidates[idx].add(digit);
                possible_positions[digit as usize].add(idx as CellIndex);
            } else if ch == '.' {
                debug_assert!(!waiting_next_digit);
                for digit in 1..=9 {
                    candidates[idx].add(digit);
                    possible_positions[digit as usize].add(idx as CellIndex);
                }
                idx += 1;
            } else {
                if waiting_next_digit {
                    assert!(candidates[idx].size() > 0);
                    if candidates[idx].size() == 1 {
                        let value = candidates[idx].iter().next().unwrap();
                        board[idx] = Some(value);
                        candidates[idx].clear();
                        possible_positions[value as usize].remove(idx as CellIndex);
                    }
                    idx += 1;
                }
                waiting_next_digit = false;
            }
        }
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
                if let Some(value) = value {
                    s.push_str(&value.to_string());
                } else {
                    s.push('.');
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
                if let Some(value) = self.get_cell_value(idx as u8) {
                    return format!("{}", value);
                }
                return candidates.iter().map(|x| x.to_string()).join("");
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
