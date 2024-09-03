use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::{CellIndex, CellValue};

use dancing_links::sudoku::{Constraint, Possibility, Sudoku as DlSudoku};
use dancing_links::{latin_square, sudoku, ExactCover};

pub fn solve_dancing_links(sudoku: &SudokuSolver, solution: &mut SolutionRecorder) {
    let possibilities = sudoku.cells().flat_map(|cell| {
        let (row, column, block) = sudoku.cell_position(cell);
        sudoku.candidates(cell).iter().map(move |value| Possibility {
            row,
            column,
            square: block,
            value: value as usize,
        })
    }).collect();
    let constraints = latin_square::Constraint::all(9)
        .map(|c| c.into())
        .chain((0..9).flat_map(|block| {
            (1..=9).map(move |value| {
                sudoku::Constraint::SquareNumber { square: block, value }
            })
        }))
        .collect();
    let dl_sudoku = DlSudoku {
        possibilities: possibilities,
        constraints,
    };
    let mut solver = dl_sudoku.solver();
    if let Some(dl_solution) = solver.next_solution() {
        for poss in dl_solution {
            let cell = sudoku.cell_index(poss.row, poss.column);
            let value = poss.value as CellValue;
            solution.add_value_set(Technique::Guess, "".to_string(), cell, value);
        }
    }
}
