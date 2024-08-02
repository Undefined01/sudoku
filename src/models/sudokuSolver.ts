import { CellIndex, CellPosition, CellSet, Sudoku, SudokuState } from "./sudoku";

const stopSymbol = Symbol("stop");
type StopSymbol = typeof stopSymbol;

export class SudokuSolver {
    sudoku: Sudoku;

    cellsInBlocks: CellSet[]
    cellsInRows: CellSet[]
    cellsInColumns: CellSet[]

    // the block, the row, and the column that a cell belongs to
    constrainsOfCell: Map<CellIndex, CellSet[]> = new Map();

    // cell to values (possible values to fill the cell)
    candidatesForCell: Map<CellIndex, Set<number>> = new Map();
    // value to cells that can be filled with the value
    possibleCellsForCandidate = new Map<number, CellSet>()

    constructor(sudoku: Sudoku) {
        this.sudoku = sudoku;
        if (sudoku.cells.length !== 81) {
            throw new Error("Invalid sudoku size");
        }

        this.cellsInBlocks = [];
        for (const houseX of [0, 3, 6]) {
            for (const houseY of [0, 3, 6]) {
                const house = new CellSet();
                house.name = `Block ${houseX + houseY / 3 + 1}`;
                for (const x of [0, 1, 2]) {
                    for (const y of [0, 1, 2]) {
                        const pos = this.sudoku.getCellPosition(houseX + x, houseY + y);
                        house.add(pos);
                        if (!this.constrainsOfCell.has(pos.idx)) {
                            this.constrainsOfCell.set(pos.idx, []);
                        }
                        this.constrainsOfCell.get(pos.idx)!.push(house);
                    }
                }
                this.cellsInBlocks.push(house)
            }
        }
        this.cellsInRows = [];
        for (let row = 0; row < 9; row++) {
            const house = new CellSet();
            house.name = `Row ${row + 1}`;
            for (let column = 0; column < 9; column++) {
                const pos = this.sudoku.getCellPosition(row, column);
                house.add(pos);
                if (!this.constrainsOfCell.has(pos.idx)) {
                    this.constrainsOfCell.set(pos.idx, []);
                }
                this.constrainsOfCell.get(pos.idx)!.push(house);
            }
            this.cellsInRows.push(house);
        }
        this.cellsInColumns = [];
        for (let column = 0; column < 9; column++) {
            const house = new CellSet();
            house.name = `Column ${column + 1}`;
            for (let row = 0; row < 9; row++) {
                const pos = this.sudoku.getCellPosition(row, column);
                house.add(pos);
                if (!this.constrainsOfCell.has(pos.idx)) {
                    this.constrainsOfCell.set(pos.idx, []);
                }
                this.constrainsOfCell.get(pos.idx)!.push(house);
            }
            this.cellsInColumns.push(house);
        }
    }

    private foreachHouse(f: (cells: CellSet) => void | StopSymbol | undefined) {
        for (const cellsInHouse of this.cellsInBlocks) {
            const res = f(cellsInHouse);
            if (res === stopSymbol) {
                return
            }
        }
        for (const cellsInRow of this.cellsInRows) {
            const res = f(cellsInRow);
            if (res === stopSymbol) {
                return
            }
        }
        for (const cellsInColumn of this.cellsInColumns) {
            const res = f(cellsInColumn);
            if (res === stopSymbol) {
                return
            }
        }
    }

    checkValueIsValid(): boolean {
        let isValid = true;
        this.foreachHouse((cells) => {
            for (const cell1 of cells) {
                for (const cell2 of cells) {
                    if (cell1 === cell2) {
                        continue;
                    }
                    if (this.sudoku.getCell(cell1).value === undefined || this.sudoku.getCell(cell2).value === undefined) {
                        continue;
                    }
                    if (this.sudoku.getCell(cell1).value === this.sudoku.getCell(cell2).value) {
                        isValid = false;
                        return stopSymbol;
                    }
                }
            }
        })
        return isValid;
    }

    fillPencilMarks() {
        if (!this.checkValueIsValid()) {
            throw new Error("Invalid sudoku");
        }

        for (const cell of this.sudoku.state.cells) {
            if (cell.isGiven || cell.value !== undefined) {
                this.candidatesForCell.delete(cell.position.idx);
                continue;
            }
            this.candidatesForCell.set(cell.position.idx, new Set([1, 2, 3, 4, 5, 6, 7, 8, 9]));
        }

        this.foreachHouse((cells) => {
            const values = cells.values().map((cell) => this.sudoku.getCell(cell).value);
            for (const cell of cells) {
                for (const value of values) {
                    if (value !== undefined) {
                        this.candidatesForCell.get(cell.idx)?.delete(value);
                    }
                }
            }
        })

        this.sudoku.updateState(false, (state) => {
            for (const [cellIdx, candidates] of this.candidatesForCell) {
                const cell = state.cells[cellIdx];
                if (cell.isGiven || cell.value !== undefined) {
                    continue;
                }
                cell.clearPencilMarks();
                for (const candidate of candidates) {
                    cell.togglePencilMark(candidate);
                }
            }
        });

        for (const [cellIdx, candidates] of this.candidatesForCell) {
            const cell = this.sudoku.state.cells[cellIdx];
            for (const value of candidates) {
                if (!this.possibleCellsForCandidate.has(value)) {
                    this.possibleCellsForCandidate.set(value, new CellSet());
                }
                this.possibleCellsForCandidate.get(value)!.add(cell.position);
            }
        }
    }

    solveOneStep() {
        const solvers = [this.fullHouse, this.nakedSingle, this.solveHiddenSingle, this.LockedCandidatesType, this.hiddenSubset, this.nakedSubset, this.basicFish, this.finnedFish, this.complexFish];
        console.time("solveOneStep");
        for (const solver of solvers) {
            if (this.sudoku.updateState(true, solver.bind(this))) {
                if (!this.checkValueIsValid()) {
                    throw new Error("Invalid sudoku");
                }
                console.timeEnd("solveOneStep");
                return
            }
        }
        console.timeEnd("solveOneStep");
    }

    private setCellValue(state: SudokuState, cell: CellPosition, value: number) {
        for (const candidate of [...this.candidatesForCell.get(cell.idx) ?? []]) {
            this.deleteCandidate(state, cell, candidate);
        }
        this.candidatesForCell.delete(cell.idx);
        state.getCell(cell).setValue(value);
        for (const constraint of this.constrainsOfCell.get(cell.idx)!) {
            for (const otherCell of constraint) {
                if (otherCell === cell) {
                    continue;
                }
                this.deleteCandidate(state, otherCell, value);
            }
        }
    }

    private deleteCandidate(state: SudokuState, cell: CellPosition, candidate: number) {
        state.getCell(cell).pencilMarks.delete(candidate);
        this.possibleCellsForCandidate.get(candidate)?.delete(cell);
        this.candidatesForCell.get(cell.idx)?.delete(candidate);
    }

    fullHouse(state: SudokuState) {
        this.foreachHouse((cells) => {
            const unfilledCells = cells.values().filter((cell) => state.getCell(cell).value === undefined);
            if (unfilledCells.length === 1) {
                const cell = unfilledCells[0];
                const missingValue = 45 - cells.values().reduce((acc, cell) => acc + (state.getCell(cell).value || 0), 0);
                this.setCellValue(state, cell, missingValue);
                console.log(`Full house: in ${cells.name}, there is only one blank => r${cell.row + 1}c${cell.column + 1}=${missingValue}`);
                return stopSymbol;
            }
        })
    }

    nakedSingle(state: SudokuState) {
        this.foreachHouse((cells) => {
            for (const cell of cells) {
                if (state.getCell(cell).value !== undefined) {
                    continue;
                }
                const pencilMarks = state.getCell(cell).pencilMarks;
                if (pencilMarks.size === 1) {
                    const value = [...pencilMarks][0];
                    console.log(`Naked single: there is only one candidate for r${cell.row + 1}c${cell.column + 1} => r${cell.row + 1}c${cell.column + 1}=${value}`);
                    this.setCellValue(state, cell, value);
                    return stopSymbol;
                }
            }
        })
    }

    solveHiddenSingle(state: SudokuState) {
        this.foreachHouse((cells) => {
            for (let value = 1; value <= 9; value++) {
                const cellsWithPencilMark = this.possibleCellsForCandidate.get(value)?.intersection(cells);
                if (cellsWithPencilMark?.size === 1) {
                    const targetCell = state.getCell([...cellsWithPencilMark][0]).position;
                    console.log(`Hidden single: in ${cells.name}, there is only one possible cell that can fill ${value} => r${targetCell.row + 1}c${targetCell.column + 1}=${value}`);
                    this.setCellValue(state, targetCell, value);
                    return stopSymbol;
                }
            }
        })
    }

    // 当 House A 中的一个数字只出现在 House A & House B 的交集中时，这个数字不可能再出现在 House B 中的其他单元格中
    LockedCandidatesType(state: SudokuState) {
        const check = (houseA: CellSet, houseB: CellSet) => {
            const intersection = houseA.intersection(houseB);
            if (intersection.size === 0) {
                return false;
            }
            for (let value = 1; value <= 9; value++) {
                const possibleCellsInA = houseA.values().filter((cell) => state.getCell(cell).pencilMarks.has(value));
                if (possibleCellsInA.length === 0) {
                    continue;
                }
                if (possibleCellsInA.every((cell) => intersection.has(cell))) {
                    let candidateDeleted = false
                    for (const cell of houseB.values()) {
                        if (!intersection.has(cell) && state.getCell(cell).pencilMarks.has(value)) {
                            this.deleteCandidate(state, cell, value);
                            console.log(`Locked candidates: in ${houseA.name}, ${value} must be filled in one of ${intersection} => r${cell.row + 1}c${cell.column + 1}<>${value}`);
                            candidateDeleted = true
                        }
                    }
                    if (candidateDeleted) {
                        return true
                    }
                }
            }
            return false
        }
        for (const block of this.cellsInBlocks) {
            for (const row of this.cellsInRows) {
                if (check(block, row) || check(row, block)) {
                    return
                }
            }
            for (const column of this.cellsInColumns) {
                if (check(block, column) || check(column, block)) {
                    return
                }
            }
        }
    }

    static *permutations<T>(arr: T[], size: number): Generator<T[]> {
        if (size === 0) {
            yield [];
        } else {
            for (let i = 0; i < arr.length; i++) {
                const rest = arr.slice(i + 1);
                for (const restPermutation of SudokuSolver.permutations(rest, size - 1)) {
                    yield [arr[i], ...restPermutation];
                }
            }
        }
    }

    // 在一个 House 中，若任意 n 个数字只可能出现在相同 n 个（或更少）单元格中，则这 n 个单元格中不可能出现其他数字
    hiddenSubset(state: SudokuState) {
        this.foreachHouse((cells) => {
            const possibleHouseCellsForCandidate = new Map<number, CellSet>();
            for (let value = 1; value <= 9; value++) {
                const possibleCells = this.possibleCellsForCandidate.get(value)?.intersection(cells);
                if (possibleCells === undefined || possibleCells.size === 0) {
                    continue;
                }
                possibleHouseCellsForCandidate.set(value, possibleCells)
            }
            for (let size = 2; size <= 4; size++) {
                const possibleHouseCellsForCandidateInSize = new Array<[number, CellSet]>();
                for (let value = 1; value <= 9; value++) {
                    const possibleCells = possibleHouseCellsForCandidate.get(value);
                    if (possibleCells && possibleCells.size <= size) {
                        possibleHouseCellsForCandidateInSize.push([value, possibleHouseCellsForCandidate.get(value)!]);
                    }
                }
                if (possibleHouseCellsForCandidateInSize.length < size) {
                    continue;
                }
                // 对于任意 size 个数字，若这些数字只出现在 size 个单元格中，则这些单元格中不可能出现其他数字
                for (const subset of SudokuSolver.permutations(possibleHouseCellsForCandidateInSize, size)) {
                    const cellUnion = CellSet.union(...subset.map(([_, cells]) => cells));
                    const valuesInSubset = new Set(subset.map(([value, _]) => value));
                    if (cellUnion.size <= size) {
                        let candidateDeleted = false
                        for (const cell of cellUnion) {
                            for (let value = 1; value <= 9; value++) {
                                if (!valuesInSubset.has(value) && state.getCell(cell).pencilMarks.has(value)) {
                                    this.deleteCandidate(state, cell, value);
                                    candidateDeleted = true
                                    console.log(`Hidden subset: in ${cells.name}, candidates ${[...valuesInSubset]} only appears in ${cellUnion} => r${cell.row + 1}c${cell.column + 1}<>${value}`);
                                }
                            }
                        }
                        if (candidateDeleted) {
                            return stopSymbol;
                        }
                    }
                }
            }
        })
    }

    // 当一个 House 中的 n 个单元格只可能包含相同的 n 个（或更少）数字时，这 n 个数字不可能出现在这个 House 中的其他单元格中
    nakedSubset(state: SudokuState) {
        const check = (cells: CellSet, size: number) => {
            const possibleValues = new Array<[CellPosition, Set<number>]>();
            for (const cell of cells) {
                const candidates = this.candidatesForCell.get(cell.idx);
                if (candidates && candidates.size <= size) {
                    possibleValues.push([cell, candidates]);
                }
            }
            for (const subset of SudokuSolver.permutations(possibleValues, size)) {
                const valueUnion = subset.reduce((acc, [_, candidates]) => { candidates.forEach(n => acc.add(n)); return acc }, new Set<number>());
                const cellsInSubset = new CellSet(...subset.map(([cell, _]) => cell));
                const cellIdsInSubset = new Set(subset.map(([cell, _]) => cell.idx));
                if (valueUnion.size <= size) {
                    let changed = false;

                    // 在当前 House 的其他单元格中删除这些数字
                    for (const cell of cells) {
                        if (cellIdsInSubset.has(cell.idx)) {
                            continue
                        }
                        for (const value of valueUnion) {
                            if (this.candidatesForCell.get(cell.idx)?.has(value)) {
                                this.deleteCandidate(state, cell, value);
                                console.log(`Naked subset: in ${cells.name}, ${cellsInSubset} only contains ${[...valueUnion]} => r${cell.row + 1}c${cell.column + 1}<>${value}`);
                                changed = true;
                            }
                        }
                    }

                    // 在所有其他同样包含了这些单元格的 House 中删除这些数字
                    this.foreachHouse((otherCells) => {
                        if (!cellsInSubset.isSubsetOf(otherCells)) {
                            return
                        }
                        for (const cell of otherCells) {
                            if (cellIdsInSubset.has(cell.idx)) {
                                continue
                            }
                            for (const value of valueUnion) {
                                if (this.candidatesForCell.get(cell.idx)?.has(value)) {
                                    this.deleteCandidate(state, cell, value);
                                    console.log(`Locked subset: in ${cells.name}, cell ${cellsInSubset} only contains ${[...valueUnion]} => r${cell.row + 1}c${cell.column + 1}<>${value}`);
                                    changed = true;
                                }
                            }
                        }
                    })

                    if (changed) {
                        return stopSymbol;
                    }
                }
            }
            return false;
        }
        this.foreachHouse((cells) => {
            for (let size = 2; size <= 4; size++) {
                if (check(cells, size)) {
                    return stopSymbol;
                }
            }
        })
    }

    // 鱼需要选取一个数字和两个集合：base set 和 cover set。集合中的元素都是 House，且集合内部的 House 不相互重叠。
    // 要形成鱼，base set 和 cover set 的大小需要相同。且 candidate 在 base set 中的出现位置必须被 cover set 覆盖。
    // 而基本的鱼是指 House 不包含 Block 的鱼，因此基本的鱼由 n 个 Row 和 n 个 Column 组成，且基础集所覆盖的单元格数量正好等于 n。
    basicFish(state: SudokuState) {
        const fishNames: { [keys: number]: string } = { 2: "X-Wing", 3: "Swordfish", 4: "Jellyfish" };

        for (let size = 2; size <= 4; size++) {
            for (let value = 1; value <= 9; value++) {
                const possibleCellsInRows = new Array<CellSet>();
                const possibleCellsInColumns = new Array<CellSet>();
                for (let i = 0; i < 9; i++) {
                    const possibleCellsInRow = this.possibleCellsForCandidate.get(value)?.intersection(this.cellsInRows[i]);
                    if (possibleCellsInRow && possibleCellsInRow.size > 0 && possibleCellsInRow.size <= size) {
                        possibleCellsInRow.name = this.cellsInRows[i].name;
                        possibleCellsInRows.push(possibleCellsInRow);
                    }
                    const possibleCellsInColumn = this.possibleCellsForCandidate.get(value)?.intersection(this.cellsInColumns[i]);
                    if (possibleCellsInColumn && possibleCellsInColumn.size > 0 && possibleCellsInColumn.size <= size) {
                        possibleCellsInColumn.name = this.cellsInColumns[i].name;
                        possibleCellsInColumns.push(possibleCellsInColumn);
                    }
                }

                const check = (baseSet: CellSet[], coverSet: CellSet[]) => {
                    const baseCells = CellSet.union(...baseSet);
                    const coverCells = CellSet.union(...coverSet);
                    if (!baseCells.isSubsetOf(coverCells)) {
                        return false;
                    }
                    let changed = false;
                    for (const cell of coverCells) {
                        if (baseCells.has(cell)) {
                            continue;
                        }
                        this.deleteCandidate(state, cell, value);
                        console.log(`${fishNames[size]}: for ${value}, ${baseSet.map(s => s.name)} is covered by ${coverSet.map(s => s.name)} => ${cell}<>${value}`);
                        changed = true;
                    }
                    return changed;
                }

                for (const colSet of SudokuSolver.permutations(possibleCellsInColumns, size)) {
                    for (const rowSet of SudokuSolver.permutations(possibleCellsInRows, size)) {
                        if (check(colSet, rowSet) || check(rowSet, colSet)) {
                            return;
                        }
                    }
                }
            }
        }
    }

    // 带鳍的鱼是指 cover set 并不完全覆盖 base set，这些没有被覆盖的单元格称为鳍。
    // 如果在数独的解中所有的鳍都不填入鱼的数字，即开了上帝视角的情况下可以假装已经消除了所有的鳍，那么可以按照鱼的逻辑删除 cover set 中的数字。
    // 如果在数独的解中有鳍填入了鱼的数字，那么按照数独的规则，可以删除鱼鳍所在行、列、块中的其他单元格中的这个数字。
    // 以上两种情况取交集，可以得出 cover set 中和所有鳍共在一个 House 中的数字无论在哪种情况都可以删除。
    finnedFish(state: SudokuState, searchComplexFish = false) {
        const fishNames: { [keys: number]: string } = { 2: "X-Wing", 3: "Swordfish", 4: "Jellyfish" };

        for (let size = 2; size <= 4; size++) {
            for (let value = 1; value <= 9; value++) {
                const possibleCellsInRows = new Array<CellSet>();
                const possibleCellsInColumns = new Array<CellSet>();
                const possibleCellsInBlocks = new Array<CellSet>();
                for (let i = 0; i < 9; i++) {
                    const possibleCellsInRow = this.possibleCellsForCandidate.get(value)?.intersection(this.cellsInRows[i]);
                    if (possibleCellsInRow && possibleCellsInRow.size > 0) {
                        possibleCellsInRow.name = this.cellsInRows[i].name;
                        possibleCellsInRows.push(possibleCellsInRow);
                    }
                    const possibleCellsInColumn = this.possibleCellsForCandidate.get(value)?.intersection(this.cellsInColumns[i]);
                    if (possibleCellsInColumn && possibleCellsInColumn.size > 0) {
                        possibleCellsInColumn.name = this.cellsInColumns[i].name;
                        possibleCellsInColumns.push(possibleCellsInColumn);
                    }
                    const possibleCellsInBlock = this.possibleCellsForCandidate.get(value)?.intersection(this.cellsInBlocks[i]);
                    if (possibleCellsInBlock && possibleCellsInBlock.size > 0) {
                        possibleCellsInBlock.name = this.cellsInBlocks[i].name;
                        possibleCellsInBlocks.push(possibleCellsInBlock);
                    }
                }

                const check = (baseSet: CellSet[], coverSet: CellSet[]) => {
                    const baseCells = CellSet.union(...baseSet);
                    const coverCells = CellSet.union(...coverSet);
                    const fins = baseCells.substract(coverCells);

                    const finCoverCells = fins.values().map(fin => CellSet.union(...this.constrainsOfCell.get(fin.idx)!));
                    const eliminatedCells = CellSet.intersection(...finCoverCells, coverCells)

                    let changed = false;
                    for (const cell of eliminatedCells) {
                        if (baseCells.has(cell)) {
                            continue;
                        }
                        this.deleteCandidate(state, cell, value);
                        const hasFin = fins.size !== 0
                        console.log(`${hasFin ? "Finned" : "Complex"} ${fishNames[size]}: for ${value}, ${baseSet.map(s => s.name)} is covered by ${coverSet.map(s => s.name)}${hasFin ? ` with fin ${fins}` : ""} => ${cell}<>${value}`);
                        changed = true;
                    }
                    return changed;
                }

                if (!searchComplexFish) {
                    for (const colSet of SudokuSolver.permutations(possibleCellsInColumns, size)) {
                        for (const rowSet of SudokuSolver.permutations(possibleCellsInRows, size)) {
                            if (check(colSet, rowSet) || check(rowSet, colSet)) {
                                return;
                            }
                        }
                    }
                } else {
                    for (const colSet of SudokuSolver.permutations([...possibleCellsInColumns, ...possibleCellsInBlocks], size)) {
                        // 两两之间互不相交
                        if ([...SudokuSolver.permutations(colSet, 2)].some(set => CellSet.intersection(...set).size !== 0)) {
                            continue;
                        }
                        for (const rowSet of SudokuSolver.permutations(possibleCellsInRows, size)) {
                            if (check(colSet, rowSet) || check(rowSet, colSet)) {
                                return;
                            }
                        }
                    }
                    for (const rowSet of SudokuSolver.permutations([...possibleCellsInRows, ...possibleCellsInBlocks], size)) {
                        // 两两之间互不相交
                        if ([...SudokuSolver.permutations(rowSet, 2)].some(set => CellSet.intersection(...set).size !== 0)) {
                            continue;
                        }
                        for (const colSet of SudokuSolver.permutations(possibleCellsInColumns, size)) {
                            if (check(colSet, rowSet) || check(rowSet, colSet)) {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    complexFish(state: SudokuState) {
        this.finnedFish(state, true);
    }
}