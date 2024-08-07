use super::utils::{CellSet, NamedCellSet};
use super::{CellIndex, CellValue, Step, StepKind, StepRule, Sudoku, UNSET};

use std::collections::HashSet;

use itertools::Itertools;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SudokuSolver {
    pub(crate) all_constraints: Vec<NamedCellSet>,
    pub(crate) constraints_of_cell: Vec<Vec<NamedCellSet>>,
    pub(crate) house_union_of_cell: Vec<CellSet>,

    pub(crate) cells_in_rows: Vec<NamedCellSet>,
    pub(crate) cells_in_columns: Vec<NamedCellSet>,
    pub(crate) cells_in_blocks: Vec<NamedCellSet>,
}

#[wasm_bindgen]
impl SudokuSolver {
    pub fn new(sudoku: &Sudoku) -> Self {
        let mut all_constraints = vec![];
        let mut constraints_of_cell = (0..81).map(|_| vec![]).collect::<Vec<_>>();
        let mut house_union_of_cell = (0..81).map(|_| CellSet::new()).collect::<Vec<_>>();
        let mut cells_in_rows = vec![];
        let mut cells_in_columns = vec![];
        let mut cells_in_blocks = vec![];

        for block_x in (0..9).step_by(3) {
            for block_y in (0..9).step_by(3) {
                let mut block_set = NamedCellSet::new(format!("b{}", block_x + block_y / 3 + 1));
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
            let mut row_set = NamedCellSet::new(format!("r{}", row + 1));
            for col in 0..9 {
                let pos = sudoku.get_cell_position(row, col);
                row_set.add(pos);
            }
            all_constraints.push(row_set.clone());
            cells_in_rows.push(row_set);
        }

        for col in 0..9 {
            let mut col_set = NamedCellSet::new(format!("c{}", col + 1));
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
            }
        }

        SudokuSolver {
            all_constraints,
            constraints_of_cell,
            house_union_of_cell,
            cells_in_rows,
            cells_in_columns,
            cells_in_blocks,
        }
    }

    pub fn get_invalid_positions(&self, sudoku: &Sudoku) -> Vec<CellIndex> {
        let mut invalid_positions = vec![];
        for house in self.all_constraints.iter() {
            for (i, cell1) in house.iter().enumerate() {
                if sudoku.get_cell_value(cell1) == UNSET {
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

    pub fn initialize_candidates(&self, sudoku: &mut Sudoku) {
        for cell in 0..81 {
            if sudoku.get_cell_value(cell) == UNSET {
                let mut candidates: HashSet<_> = (1..=9).collect();

                for constraint in self.constraints_of_cell[cell as usize].iter() {
                    for other_cell in constraint.iter() {
                        if cell == other_cell {
                            continue;
                        }
                        let other_value = sudoku.get_cell_value(other_cell);
                        candidates.remove(&other_value);
                    }
                }

                for &candidate in candidates.iter().sorted() {
                    sudoku.add_candidate(cell, candidate);
                }
            }
        }
    }

    pub fn apply_step(&mut self, sudoku: &mut Sudoku, step: &Step) {
        match step.kind {
            StepKind::ValueSet => {
                for position in step.positions.iter() {
                    sudoku.board[position.cell_index as usize] = position.value;
                    for cell in self.house_union_of_cell[position.cell_index as usize].iter() {
                        if cell == position.cell_index {
                            continue;
                        }
                        sudoku.candidates[cell as usize].retain(|&x| x != position.value);
                        sudoku.possible_positions[position.value as usize].delete(cell);
                    }
                    sudoku.candidates[position.cell_index as usize].clear();
                    for value in 1..=9 {
                        sudoku.possible_positions[value as usize].delete(position.cell_index);
                    }
                }
            }
            StepKind::CandidateEliminated => {
                for position in step.positions.iter() {
                    sudoku.candidates[position.cell_index as usize]
                        .retain(|&x| x != position.value);
                    sudoku.possible_positions[position.value as usize].delete(position.cell_index);
                }
            }
        }
    }

    pub fn solve_full_house(&self, sudoku: &mut Sudoku) -> Option<Step> {
        for house in self.all_constraints.iter() {
            let unfilled_cells_count = house
                .iter()
                .filter(|&cell| sudoku.get_cell_value(cell) == UNSET)
                .count();
            if unfilled_cells_count == 1 {
                let unfilled_cell = house
                    .iter()
                    .filter(|&cell| sudoku.get_cell_value(cell) == UNSET)
                    .next()
                    .unwrap();
                let missing_value = sudoku
                    .get_candidates(unfilled_cell)
                    .iter()
                    .cloned()
                    .next()
                    .unwrap();
                return Some(Step::new_value_set(
                    StepRule::FullHouse,
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

    pub fn solve_naked_single(&self, sudoku: &mut Sudoku) -> Option<Step> {
        for house in self.all_constraints.iter() {
            for cell in house.iter() {
                if sudoku.get_cell_value(cell) == UNSET {
                    continue;
                }
                if sudoku.get_candidates(cell).len() == 1 {
                    let &value = sudoku.get_candidates(cell).iter().next().unwrap();
                    return Some(Step::new_value_set(
                        StepRule::NakedSingle,
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

    pub fn solve_hidden_single(&self, sudoku: &mut Sudoku) -> Option<Step> {
        for house in self.all_constraints.iter() {
            for value in 1..=9 {
                let possible_cells = sudoku.get_possible_cells(value) & house;
                // println!("in {}, value {} can be in {}", house.name(), value, possible_cells.iter().map(|cell| sudoku.get_cell_name(cell)).join(","));
                if possible_cells.size() == 1 {
                    let target_cell = possible_cells.iter().next().unwrap();
                    return Some(Step::new_value_set(
                        StepRule::HiddenSingle,
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

    // 当 House A 中的一个数字只出现在 House A & House B （A 和 B的交集）中时，这个数字不可能再出现在 House B 中的其他单元格中
    pub fn solve_locked_candidates(&self, sudoku: &mut Sudoku) -> Option<Step> {
        let check = |house_a: &NamedCellSet, house_b: &NamedCellSet| -> Option<Step> {
            let intersection = house_a & house_b;
            if intersection.is_empty() {
                return None;
            }

            let mut step = Step::new(StepKind::CandidateEliminated, StepRule::LockedCandidates);

            for value in 1..=9 {
                let possible_cells_in_a = house_a & sudoku.get_possible_cells(value);
                if possible_cells_in_a.is_empty()
                    || !possible_cells_in_a.is_subset_of(&intersection)
                {
                    continue;
                }
                for cell in house_b.iter() {
                    if intersection.has(cell) {
                        continue;
                    }
                    if sudoku.can_fill(cell, value) {
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
    pub fn solve_hidden_subset(&self, sudoku: &mut Sudoku) -> Option<Step> {
        let mut step = Step::new(StepKind::CandidateEliminated, StepRule::HiddenSubset);

        for house in self.all_constraints.iter() {
            let mut possible_cells_in_houses = vec![];
            for value in 1..=9 {
                let possible_cells_in_house = sudoku.get_possible_cells(value) & house;
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
                    let cell_union = CellSet::union_multiple(subset.iter().map(|(_, cells)| cells));
                    let values_in_subset: HashSet<_> =
                        subset.iter().map(|(value, _)| *value).collect();

                    if cell_union.size() <= size {
                        for cell in cell_union.iter() {
                            for value in 1..=9 {
                                if !values_in_subset.contains(&value)
                                    && sudoku.can_fill(cell, value)
                                {
                                    step.add(
                                        format!(
                                            "in {}, {} only appears in {}",
                                            house.name(),
                                            values_in_subset.iter().sorted().join(","),
                                            cell_union.to_string(sudoku),
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
    pub fn solve_naked_subset(&self, sudoku: &mut Sudoku) -> Option<Step> {
        let check = |cells: &NamedCellSet, size: usize| -> Option<Step> {
            let mut step = Step::new(StepKind::CandidateEliminated, StepRule::NakedSubset);

            for subset in cells
                .iter()
                .filter(|&cell| sudoku.get_candidates(cell).len() >= size)
                .combinations(size)
            {
                let value_union: HashSet<_> = subset
                    .iter()
                    .flat_map(|&cell| sudoku.get_candidates(cell).iter().copied())
                    .collect();
                let cells_in_subset = CellSet::from_cells(subset);

                if value_union.len() > size {
                    continue;
                }

                for cell in cells.iter() {
                    if cells_in_subset.has(cell) {
                        continue;
                    }
                    for &value in &value_union {
                        if sudoku.can_fill(cell, value) {
                            step.add(
                                format!(
                                    "in {}, {} only contains {}",
                                    cells.name(),
                                    cells_in_subset.to_string(sudoku),
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
            None
        };

        for house in self.all_constraints.iter() {
            for size in 2..=4 {
                if let Some(step) = check(house, size) {
                    return Some(step);
                }
            }
        }
        None
    }

    fn check_is_fish(
        &self,
        sudoku: &Sudoku,
        base_set: &[&NamedCellSet],
        cover_set: &[&NamedCellSet],
        value: CellValue,
        rule: StepRule,
    ) -> Option<Step> {
        let base_cells = CellSet::union_multiple(base_set.iter().map(|&s| &**s));
        let cover_cells = CellSet::union_multiple(cover_set.iter().map(|&s| &**s));
        let fins = &base_cells - &cover_cells;
        let mut eliminated_cells = &cover_cells - &base_cells;
        if eliminated_cells.is_empty() {
            return None;
        }

        let allow_fins = rule != StepRule::BasicFish;
        if !fins.is_empty() && !allow_fins {
            return None;
        }
        for fin in fins.iter() {
            eliminated_cells &= &self.house_union_of_cell[fin as usize];
        }
        if eliminated_cells.is_empty() {
            return None;
        }

        let mut step = Step::new(StepKind::CandidateEliminated, rule.clone());
        for cell in eliminated_cells.iter() {
            let reason = if rule == StepRule::FinnedFish {
                format!(
                    "for {}, {} is covered by {}",
                    value,
                    base_set.iter().map(|s| s.name()).join(","),
                    cover_set.iter().map(|s| s.name()).join(","),
                )
            } else {
                format!(
                    "for {}, {} is covered by {} with fins {}",
                    value,
                    base_set.iter().map(|s| s.name()).join(","),
                    cover_set.iter().map(|s| s.name()).join(","),
                    fins.to_string(sudoku),
                )
            };
            step.add(reason, cell, value);
        }
        return Some(step);
    }

    // 鱼需要选取一个数字和两个集合：base set 和 cover set。集合中的元素都是 House，且集合内部的 House 不相互重叠。
    // 要形成鱼，base set 和 cover set 的大小需要相同。且 candidate 在 base set 中的出现位置必须被 cover set 覆盖。
    // 而基本的鱼是指 House 不包含 Block 的鱼，因此基本的鱼由 n 个 Row 和 n 个 Column 组成，且基础集所覆盖的单元格数量正好等于 n。
    pub fn solve_fish(&self, sudoku: &mut Sudoku, rule: StepRule) -> Option<Step> {
        for size in 2..=4 {
            for value in 1..=9 {
                let possible_cells = sudoku.get_possible_cells(value);
                let rows = self
                    .cells_in_rows
                    .iter()
                    .map(|s| NamedCellSet::from_cellset(s.name().to_string(), s & possible_cells))
                    .filter(|s| !s.is_empty() && s.size() <= size)
                    .collect_vec();
                let cols = self
                    .cells_in_columns
                    .iter()
                    .map(|s| NamedCellSet::from_cellset(s.name().to_string(), s & possible_cells))
                    .filter(|s| !s.is_empty() && s.size() <= size)
                    .collect_vec();
                let blocks = self
                    .cells_in_blocks
                    .iter()
                    .map(|s| NamedCellSet::from_cellset(s.name().to_string(), s & possible_cells))
                    .filter(|s| !s.is_empty() && s.size() <= size)
                    .collect_vec();

                if rule != StepRule::ComplexFish {
                    for rol_set in rows.iter().combinations(size) {
                        for col_set in cols.iter().combinations(size) {
                            if let Some(step) =
                                self.check_is_fish(sudoku, &rol_set, &col_set, value, rule.clone())
                            {
                                return Some(step);
                            }
                            if let Some(step) =
                                self.check_is_fish(sudoku, &col_set, &rol_set, value, rule.clone())
                            {
                                return Some(step);
                            }
                        }
                    }
                } else {
                    for rol_set in (0..size).flat_map(|rol_size| rows.iter().combinations(rol_size))
                    {
                        for block_set in blocks
                            .iter()
                            .filter(|&b| rol_set.iter().all(|&r| (r & b).is_empty()))
                            .combinations(size - rol_set.len())
                        {
                            let rol_block_set = rol_set
                                .iter()
                                .chain(block_set.iter())
                                .cloned()
                                .collect_vec();
                            for col_set in cols.iter().combinations(size) {
                                if let Some(step) = self.check_is_fish(
                                    sudoku,
                                    &rol_block_set,
                                    &col_set,
                                    value,
                                    rule.clone(),
                                ) {
                                    return Some(step);
                                }
                                if let Some(step) = self.check_is_fish(
                                    sudoku,
                                    &col_set,
                                    &rol_block_set,
                                    value,
                                    rule.clone(),
                                ) {
                                    return Some(step);
                                }
                            }
                        }
                    }
                    for col_set in (0..size).flat_map(|col_size| cols.iter().combinations(col_size))
                    {
                        for block_set in blocks
                            .iter()
                            .filter(|&b| col_set.iter().all(|&c| (c & b).is_empty()))
                            .combinations(size - col_set.len())
                        {
                            let col_block_set = col_set
                                .iter()
                                .chain(block_set.iter())
                                .cloned()
                                .collect_vec();
                            for rol_set in rows.iter().combinations(size) {
                                if let Some(step) = self.check_is_fish(
                                    sudoku,
                                    &rol_set,
                                    &col_block_set,
                                    value,
                                    rule.clone(),
                                ) {
                                    return Some(step);
                                }
                                if let Some(step) = self.check_is_fish(
                                    sudoku,
                                    &col_block_set,
                                    &rol_set,
                                    value,
                                    rule.clone(),
                                ) {
                                    return Some(step);
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn solve_basic_fish(&self, sudoku: &mut Sudoku) -> Option<Step> {
        self.solve_fish(sudoku, StepRule::BasicFish)
    }

    pub fn solve_finned_fish(&self, sudoku: &mut Sudoku) -> Option<Step> {
        self.solve_fish(sudoku, StepRule::FinnedFish)
    }

    pub fn solve_complex_fish(&self, sudoku: &mut Sudoku) -> Option<Step> {
        self.solve_fish(sudoku, StepRule::ComplexFish)
    }

    pub fn solve_one_step(&self, sudoku: &mut Sudoku) -> Option<Step> {
        let solving_techniques = [
            SudokuSolver::solve_full_house,
            SudokuSolver::solve_naked_single,
            SudokuSolver::solve_hidden_single,
            SudokuSolver::solve_locked_candidates,
            SudokuSolver::solve_hidden_subset,
            SudokuSolver::solve_naked_subset,
            SudokuSolver::solve_basic_fish,
            SudokuSolver::solve_finned_fish,
            SudokuSolver::solve_complex_fish,
        ];
        for technique in solving_techniques.iter() {
            if let Some(step) = technique(self, sudoku) {
                return Some(step);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sudoku::Sudoku;

    fn solve_all(
        solving_techniques: &[fn(&SudokuSolver, &mut Sudoku) -> Option<Step>],
        sudoku: &mut Sudoku,
    ) -> Vec<Step> {
        let mut solver = SudokuSolver::new(&sudoku);

        let mut steps = vec![];
        let mut has_step = true;
        while has_step {
            has_step = false;
            for solve in solving_techniques.iter() {
                let step = solve(&solver, sudoku);
                if let Some(step) = step {
                    has_step = true;
                    solver.apply_step(sudoku, &step);
                    println!("{}", sudoku.to_candidate_string().trim());
                    println!("{}", step.to_string(sudoku));
                    steps.push(step);
                    break;
                }
            }
        }

        steps
    }

    #[test]
    fn test_solver() {
        let mut sudoku = Sudoku::from_str(
            "53__7____
             6__195___
             _98____6_
             8___6___3
             4__8_3__1
             7___2___6
             _6____28_
             ___419__5
             ____8__79",
        );
        let solver = SudokuSolver::new(&sudoku);
        solver.initialize_candidates(&mut sudoku);

        assert_eq!(
            sudoku.to_candidate_string().trim(),
            "
+-----------------+--------------+-----------------+
|   5    3    124 |   26  7 2468 |  1489 1249  248 |
|   6  247    247 |    1  9    5 |  3478  234 2478 |
|  12    9      8 |   23 34   24 | 13457    6  247 |
+-----------------+--------------+-----------------+
|   8  125   1259 |  579  6  147 |  4579 2459    3 |
|   4   25   2569 |    8  5    3 |   579  259    1 |
|   7   15   1359 |   59  2   14 |  4589  459    6 |
+-----------------+--------------+-----------------+
| 139    6 134579 |  357 35    7 |     2    8    4 |
|  23  278    237 |    4  1    9 |    36    3    5 |
| 123 1245  12345 | 2356  8   26 |  1346    7    9 |
+-----------------+--------------+-----------------+"
                .trim()
        );

        let steps = solve_all(
            &[
                SudokuSolver::solve_full_house,
                SudokuSolver::solve_naked_single,
                SudokuSolver::solve_hidden_single,
                SudokuSolver::solve_locked_candidates,
                SudokuSolver::solve_hidden_subset,
                SudokuSolver::solve_naked_subset,
            ],
            &mut sudoku,
        );

        assert_eq!(solver.get_invalid_positions(&sudoku), vec![]);

        assert_eq!(
            steps
                .iter()
                .map(|x| x.to_string(&sudoku).trim().to_string())
                .collect_vec(),
            [
                "[HiddenSingle] in b2, r1c6 is the only possible cell that can be 8 => r1c6=8",
                "[HiddenSingle] in b2, r1c4 is the only possible cell that can be 6 => r1c4=6",
                "[HiddenSingle] in b3, r3c7 is the only possible cell that can be 5 => r3c7=5",
                "[HiddenSingle] in b4, r6c3 is the only possible cell that can be 3 => r6c3=3",
                "[HiddenSingle] in b4, r5c3 is the only possible cell that can be 6 => r5c3=6",
                "[HiddenSingle] in b4, r4c3 is the only possible cell that can be 9 => r4c3=9",
                "[HiddenSingle] in b5, r6c4 is the only possible cell that can be 9 => r6c4=9",
                "[HiddenSingle] in b6, r6c7 is the only possible cell that can be 8 => r6c7=8",
                "[HiddenSingle] in b3, r2c9 is the only possible cell that can be 8 => r2c9=8",
                "[HiddenSingle] in b7, r8c2 is the only possible cell that can be 8 => r8c2=8",
                "[HiddenSingle] in b7, r7c1 is the only possible cell that can be 9 => r7c1=9",
                "[HiddenSingle] in b8, r9c6 is the only possible cell that can be 6 => r9c6=6",
                "[HiddenSingle] in b8, r9c4 is the only possible cell that can be 2 => r9c4=2",
                "[HiddenSingle] in b2, r3c6 is the only possible cell that can be 2 => r3c6=2",
                "[HiddenSingle] in b2, r3c5 is the only possible cell that can be 4 => r3c5=4",
                "[FullHouse] r3c4 is the only missing cell in b2 => r3c4=3",
                "[HiddenSingle] in b8, r7c5 is the only possible cell that can be 3 => r7c5=3",
                "[FullHouse] r5c5 is the only missing cell in c5 => r5c5=5",
                "[HiddenSingle] in b8, r7c4 is the only possible cell that can be 5 => r7c4=5",
                "[FullHouse] r7c6 is the only missing cell in b8 => r7c6=7",
                "[FullHouse] r4c4 is the only missing cell in c4 => r4c4=7",
                "[HiddenSingle] in b6, r5c7 is the only possible cell that can be 7 => r5c7=7",
                "[HiddenSingle] in b3, r3c9 is the only possible cell that can be 7 => r3c9=7",
                "[FullHouse] r3c1 is the only missing cell in r3 => r3c1=1",
                "[HiddenSingle] in b6, r5c8 is the only possible cell that can be 9 => r5c8=9",
                "[FullHouse] r5c2 is the only missing cell in r5 => r5c2=2",
                "[HiddenSingle] in b3, r1c7 is the only possible cell that can be 9 => r1c7=9",
                "[HiddenSingle] in b3, r1c8 is the only possible cell that can be 1 => r1c8=1",
                "[HiddenSingle] in b6, r4c8 is the only possible cell that can be 2 => r4c8=2",
                "[HiddenSingle] in b3, r1c9 is the only possible cell that can be 2 => r1c9=2",
                "[FullHouse] r1c3 is the only missing cell in r1 => r1c3=4",
                "[FullHouse] r7c9 is the only missing cell in c9 => r7c9=4",
                "[FullHouse] r7c3 is the only missing cell in r7 => r7c3=1",
                "[HiddenSingle] in b1, r2c3 is the only possible cell that can be 2 => r2c3=2",
                "[FullHouse] r2c2 is the only missing cell in b1 => r2c2=7",
                "[HiddenSingle] in b6, r6c8 is the only possible cell that can be 5 => r6c8=5",
                "[FullHouse] r4c7 is the only missing cell in b6 => r4c7=4",
                "[HiddenSingle] in b3, r2c8 is the only possible cell that can be 4 => r2c8=4",
                "[FullHouse] r2c7 is the only missing cell in b3 => r2c7=3",
                "[FullHouse] r8c8 is the only missing cell in c8 => r8c8=3",
                "[HiddenSingle] in b4, r4c2 is the only possible cell that can be 5 => r4c2=5",
                "[FullHouse] r6c2 is the only missing cell in b4 => r6c2=1",
                "[FullHouse] r4c6 is the only missing cell in r4 => r4c6=1",
                "[FullHouse] r6c6 is the only missing cell in b5 => r6c6=4",
                "[FullHouse] r9c2 is the only missing cell in c2 => r9c2=4",
                "[HiddenSingle] in b7, r8c1 is the only possible cell that can be 2 => r8c1=2",
                "[FullHouse] r9c1 is the only missing cell in c1 => r9c1=3",
                "[HiddenSingle] in b7, r9c3 is the only possible cell that can be 5 => r9c3=5",
                "[FullHouse] r8c3 is the only missing cell in b7 => r8c3=7",
                "[FullHouse] r8c7 is the only missing cell in r8 => r8c7=6",
                "[FullHouse] r9c7 is the only missing cell in b9 => r9c7=1"
            ]
            .iter()
            .map(|x| x.to_string())
            .collect_vec()
        );
    }

    #[test]
    fn test_fish_solver_1() {
        let mut sudoku = Sudoku::from_str(
            ".5..346..........8.3.879....15.....6...26..5.......92..4..27.13.73...........87..",
        );
        let solver = SudokuSolver::new(&sudoku);
        solver.initialize_candidates(&mut sudoku);

        println!("{}", sudoku.to_candidate_string().trim());
        assert_eq!(
            sudoku.to_candidate_string().trim(),
            "
+-------------------+------------------+-----------------+
|  12789   5  12789 |      1    3    4 |     6   79 1279 |
| 124679 269 124679 |    156   15 1256 | 12345 3479    8 |
|   1246   3   1246 |      8    7    9 |  1245    4 1245 |
+-------------------+------------------+-----------------+
| 234789   1      5 |   3479  489    3 |   348 3478    6 |
|  34789  89   4789 |      2    6   13 |  1348    5  147 |
|  34678  68   4678 |  13457 1458  135 |     9    2  147 |
+-------------------+------------------+-----------------+
|   5689   4    689 |    569    2    7 |    58    1    3 |
| 125689   7      3 |  14569 1459  156 |  2458 4689 2459 |
|  12569 269   1269 | 134569 1459    8 |     7  469 2459 |
+-------------------+------------------+-----------------+"
                .trim()
        );

        let steps = solve_all(
            &[
                SudokuSolver::solve_full_house,
                SudokuSolver::solve_naked_single,
                SudokuSolver::solve_hidden_single,
                SudokuSolver::solve_locked_candidates,
                SudokuSolver::solve_hidden_subset,
                SudokuSolver::solve_naked_subset,
                SudokuSolver::solve_basic_fish,
                SudokuSolver::solve_finned_fish,
                SudokuSolver::solve_complex_fish,
            ],
            &mut sudoku,
        );

        assert_eq!(solver.get_invalid_positions(&sudoku), vec![]);

        assert_eq!(
            steps
                .iter()
                .map(|x| x.to_string(&sudoku).trim().to_string())
                .collect_vec(),
            ["[HiddenSingle] in b2, r2c6 is the only possible cell that can be 2 => r2c6=2", "[HiddenSingle] in b2, r2c4 is the only possible cell that can be 6 => r2c4=6", "[HiddenSingle] in b2, r2c5 is the only possible cell that can be 5 => r2c5=5", "[FullHouse] r1c4 is the only missing cell in b2 => r1c4=1", "[HiddenSingle] in b4, r4c1 is the only possible cell that can be 2 => r4c1=2", "[HiddenSingle] in b8, r9c4 is the only possible cell that can be 3 => r9c4=3", "[HiddenSingle] in b8, r8c6 is the only possible cell that can be 6 => r8c6=6", "[HiddenSingle] in b9, r9c8 is the only possible cell that can be 6 => r9c8=6", "[HiddenSingle] in c2, r9c2 is the only possible cell that can be 2 => r9c2=2", "[HiddenSingle] in c2, r6c2 is the only possible cell that can be 6 => r6c2=6", "[HiddenSingle] in c2, r5c2 is the only possible cell that can be 8 => r5c2=8", "[FullHouse] r2c2 is the only missing cell in c2 => r2c2=9", "[HiddenSingle] in r6, r6c5 is the only possible cell that can be 8 => r6c5=8", "[HiddenSingle] in c6, r6c6 is the only possible cell that can be 5 => r6c6=5", "[HiddenSingle] in b5, r5c6 is the only possible cell that can be 1 => r5c6=1", "[FullHouse] r4c6 is the only missing cell in c6 => r4c6=3", "[HiddenSingle] in b6, r6c9 is the only possible cell that can be 1 => r6c9=1", "[HiddenSingle] in b6, r5c7 is the only possible cell that can be 3 => r5c7=3", "[HiddenSingle] in b3, r2c8 is the only possible cell that can be 3 => r2c8=3", "[HiddenSingle] in b4, r6c1 is the only possible cell that can be 3 => r6c1=3", "[LockedCandidates] in r2, 7 can only be in r2 & b1 => r1c1<>7\n[LockedCandidates] in r2, 7 can only be in r2 & b1 => r1c3<>7", "[HiddenSubset] in b1, 1,4,6,7 only appears in r2c1,r2c3,r3c1,r3c3 => r3c3<>2", "[HiddenSingle] in b1, r1c3 is the only possible cell that can be 2 => r1c3=2", "[HiddenSingle] in b1, r1c1 is the only possible cell that can be 8 => r1c1=8", "[HiddenSingle] in b7, r7c3 is the only possible cell that can be 8 => r7c3=8", "[HiddenSingle] in b7, r7c1 is the only possible cell that can be 6 => r7c1=6", "[HiddenSingle] in b1, r3c3 is the only possible cell that can be 6 => r3c3=6", "[HiddenSingle] in r7, r7c4 is the only possible cell that can be 9 => r7c4=9", "[FullHouse] r7c7 is the only missing cell in r7 => r7c7=5", "[HiddenSingle] in b3, r3c9 is the only possible cell that can be 5 => r3c9=5", "[HiddenSingle] in b3, r3c7 is the only possible cell that can be 2 => r3c7=2", "[HiddenSingle] in b3, r2c7 is the only possible cell that can be 1 => r2c7=1", "[HiddenSingle] in b1, r3c1 is the only possible cell that can be 1 => r3c1=1", "[FullHouse] r3c8 is the only missing cell in r3 => r3c8=4", "[HiddenSingle] in b5, r4c5 is the only possible cell that can be 9 => r4c5=9", "[HiddenSingle] in b7, r9c3 is the only possible cell that can be 1 => r9c3=1", "[HiddenSingle] in b8, r8c5 is the only possible cell that can be 1 => r8c5=1", "[FullHouse] r9c5 is the only missing cell in c5 => r9c5=4", "[FullHouse] r8c4 is the only missing cell in b8 => r8c4=5", "[HiddenSingle] in b7, r9c1 is the only possible cell that can be 5 => r9c1=5", "[FullHouse] r8c1 is the only missing cell in b7 => r8c1=9", "[FullHouse] r9c9 is the only missing cell in r9 => r9c9=9", "[HiddenSingle] in b3, r1c8 is the only possible cell that can be 9 => r1c8=9", "[FullHouse] r1c9 is the only missing cell in b3 => r1c9=7", "[HiddenSingle] in b4, r5c3 is the only possible cell that can be 9 => r5c3=9", "[HiddenSingle] in b6, r4c8 is the only possible cell that can be 7 => r4c8=7", "[FullHouse] r8c8 is the only missing cell in c8 => r8c8=8", "[HiddenSingle] in b5, r6c4 is the only possible cell that can be 7 => r6c4=7", "[FullHouse] r4c4 is the only missing cell in b5 => r4c4=4", "[FullHouse] r4c7 is the only missing cell in r4 => r4c7=8", "[FullHouse] r5c9 is the only missing cell in b6 => r5c9=4", "[FullHouse] r5c1 is the only missing cell in r5 => r5c1=7", "[FullHouse] r6c3 is the only missing cell in b4 => r6c3=4", "[FullHouse] r2c1 is the only missing cell in c1 => r2c1=4", "[FullHouse] r2c3 is the only missing cell in b1 => r2c3=7", "[FullHouse] r8c7 is the only missing cell in c7 => r8c7=4", "[FullHouse] r8c9 is the only missing cell in b9 => r8c9=2"]
            .iter()
            .map(|x| x.to_string())
            .collect_vec()
        );
        assert_eq!(
            sudoku.to_candidate_string().trim(),
            "
+-------+-------+-------+
| 8 5 2 | 1 3 4 | 6 9 7 |
| 4 9 7 | 6 5 2 | 1 3 8 |
| 1 3 6 | 8 7 9 | 2 4 5 |
+-------+-------+-------+
| 2 1 5 | 4 9 3 | 8 7 6 |
| 7 8 9 | 2 6 1 | 3 5 4 |
| 3 6 4 | 7 8 5 | 9 2 1 |
+-------+-------+-------+
| 6 4 8 | 9 2 7 | 5 1 3 |
| 9 7 3 | 5 1 6 | 4 8 2 |
| 5 2 1 | 3 4 8 | 7 6 9 |
+-------+-------+-------+"
                .trim()
        );
    }

    #[test]
    fn test_fish_solver_2() {
        let mut sudoku = Sudoku::from_str(
            "9.7..5...1..7..9..86..9.57..8...61.9316.59..72.91..65.....2..96.9...4..8...9..3.5",
        );
        let solver = SudokuSolver::new(&sudoku);
        solver.initialize_candidates(&mut sudoku);

        println!("{}", sudoku.to_candidate_string().trim());
        assert_eq!(
            sudoku.to_candidate_string().trim(),
            "
+----------------+------------------+-----------------+
|   9  234     7 | 23468 13468    5 | 248 123468 1234 |
|   1 2345  2345 |     7  3468  238 |   9  23468  234 |
|   8    6   234 |   234     9  123 |   5      7 1234 |
+----------------+------------------+-----------------+
| 457    8    45 |   234   347    6 |   1    234    9 |
|   3    1     6 |   248     5    9 | 248    248    7 |
|   2   47     9 |     1  3478  378 |   6      5   34 |
+----------------+------------------+-----------------+
| 457 3457 13458 |   358     2 1378 |  47      9    6 |
| 567    9  1235 |   356  1367    4 |  27     12    8 |
| 467  247  1248 |     9  1678  178 |   3    124    5 |
+----------------+------------------+-----------------+"
                .trim()
        );

        let steps = solve_all(
            &[
                SudokuSolver::solve_full_house,
                SudokuSolver::solve_naked_single,
                SudokuSolver::solve_hidden_single,
                SudokuSolver::solve_locked_candidates,
                SudokuSolver::solve_hidden_subset,
                SudokuSolver::solve_naked_subset,
                SudokuSolver::solve_basic_fish,
                SudokuSolver::solve_finned_fish,
                SudokuSolver::solve_complex_fish,
            ],
            &mut sudoku,
        );

        assert_eq!(solver.get_invalid_positions(&sudoku), vec![]);

        assert_eq!(
            steps
                .iter()
                .map(|x| x.to_string(&sudoku).trim().to_string())
                .collect_vec(),
                ["[LockedCandidates] in c6, 2 can only be in c6 & b2 => r1c4<>2\n[LockedCandidates] in c6, 2 can only be in c6 & b2 => r3c4<>2", "[LockedCandidates] in c9, 1 can only be in c9 & b3 => r1c8<>1", "[LockedCandidates] in c9, 2 can only be in c9 & b3 => r1c7<>2\n[LockedCandidates] in c9, 2 can only be in c9 & b3 => r1c8<>2\n[LockedCandidates] in c9, 2 can only be in c9 & b3 => r2c8<>2", "[LockedCandidates] in r6, 8 can only be in r6 & b5 => r5c4<>8", "[HiddenSubset] in c4, 5,6,8 only appears in r1c4,r7c4,r8c4 => r1c4<>3\n[HiddenSubset] in c4, 5,6,8 only appears in r1c4,r7c4,r8c4 => r1c4<>4\n[HiddenSubset] in c4, 5,6,8 only appears in r1c4,r7c4,r8c4 => r7c4<>3\n[HiddenSubset] in c4, 5,6,8 only appears in r1c4,r7c4,r8c4 => r8c4<>3", "[ComplexFish] for 3, c4,b6,b7,b8 is covered by r3,r4,r7,r8 with fins r6c9 => r3c9<>3"]
            .iter()
            .map(|x| x.to_string())
            .collect_vec()
        );
    }
}
