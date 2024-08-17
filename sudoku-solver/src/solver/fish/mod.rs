mod fish_utils;
mod franken_fish;
mod mutant_fish;
mod simple_fish;

use crate::solver::return_if_some;
use crate::solver::{Step, SudokuSolver, Technique};

// 鱼需要选取一个数字和两个集合：base set 和 cover set。集合中的元素都是 House，且集合内部的 House 不相互重叠。
// 要形成鱼，base set 和 cover set 的大小需要相同。且 candidate 在 base set 中的出现位置必须被 cover set 覆盖。
// 而基本的鱼是指 House 不包含 Block 的鱼，因此基本的鱼由 n 个 Row 和 n 个 Column 组成，且基础集所覆盖的单元格数量正好等于 n。
pub fn solve_basic_fish(sudoku: &SudokuSolver) -> Option<Step> {
    for size in 2..=4 {
        for value in 1..=9 {
            return_if_some!(simple_fish::search_simple_fish(
                sudoku,
                size,
                value,
                Technique::BasicFish
            ));
        }
    }
    None
}

pub fn solve_finned_fish(sudoku: &SudokuSolver) -> Option<Step> {
    for size in 2..=4 {
        for value in 1..=9 {
            return_if_some!(simple_fish::search_simple_fish(
                sudoku,
                size,
                value,
                Technique::FinnedFish
            ));
        }
    }
    None
}

pub fn solve_franken_fish(sudoku: &SudokuSolver) -> Option<Step> {
    // Every Franken X-Wing is degenerate to a finned X-Wing.
    for size in 3..=4 {
        for value in 1..=9 {
            return_if_some!(franken_fish::search_franken_fish(sudoku, size, value));
        }
    }
    None
}

pub fn solve_mutant_fish(sudoku: &SudokuSolver) -> Option<Step> {
    for size in 3..=4 {
        for value in 1..=9 {
            return_if_some!(mutant_fish::search_mutant_fish(sudoku, size, value));
        }
    }
    None
}
