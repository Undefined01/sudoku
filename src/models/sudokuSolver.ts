import { CellPosition, CellSet, Sudoku, SudokuState } from "./sudoku";

const stopSymbol = Symbol("stop");
type StopSymbol = typeof stopSymbol;

export class SudokuSolver {
    sudoku: Sudoku;

    cellsInBlocks: CellSet[]
    cellsInRows: CellSet[]
    cellsInColumns: CellSet[]

    constrainsOfCell: Map<number, CellSet[]> = new Map();

    constructor(sudoku: Sudoku) {
        this.sudoku = sudoku;
        if (sudoku.cells.length !== 81) {
            throw new Error("Invalid sudoku size");
        }

        this.cellsInBlocks = [];
        for (let houseX of [0, 3, 6]) {
            for (let houseY of [0, 3, 6]) {
                let house = new CellSet();
                for (let x of [0, 1, 2]) {
                    for (let y of [0, 1, 2]) {
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
            let house = new CellSet();
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
            let house = new CellSet();
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
        for (let cellsInHouse of this.cellsInBlocks) {
            const res = f(cellsInHouse);
            if (res === stopSymbol) {
                return
            }
        }
        for (let cellsInRow of this.cellsInRows) {
            const res = f(cellsInRow);
            if (res === stopSymbol) {
                return
            }
        }
        for (let cellsInColumn of this.cellsInColumns) {
            const res = f(cellsInColumn);
            if (res === stopSymbol) {
                return
            }
        }
    }

    checkValueIsValid(): boolean {
        let isValid = true;
        this.foreachHouse((cells) => {
            for (let cell1 of cells) {
                for (let cell2 of cells) {
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
        this.sudoku.updateState(false, (state) => {
            for (let cell of state.cells) {
                if (cell.isGiven || cell.value !== undefined) {
                    continue;
                }
                cell.pencilMarks = new Set([1, 2, 3, 4, 5, 6, 7, 8, 9]);
            }

            this.foreachHouse((cells) => {
                let values = cells.values().map((cell) => state.getCell(cell).value);
                for (let cell of cells) {
                    for (let value of values) {
                        if (value !== undefined) {
                            state.getCell(cell).pencilMarks.delete(value);
                        }
                    }
                }
            })
        });
    }

    solveOneStep() {
        const solvers = [this.fullHouse, this.nakedSingle, this.solveHiddenSingle, this.LockedCandidatesType];
        for (let solver of solvers) {
            if (this.sudoku.updateState(true, solver.bind(this))) {
                return
            }
        }
    }

    private setCellValue(state: SudokuState, cell: CellPosition, value: number) {
        state.getCell(cell).setValue(value);
        for (let constraint of this.constrainsOfCell.get(cell.idx)!) {
            for (let otherCell of constraint) {
                if (otherCell === cell) {
                    continue;
                }
                state.getCell(otherCell).pencilMarks.delete(value);
            }
        }
    }

    fullHouse(state: SudokuState) {
        this.foreachHouse((cells) => {
            const unfilledCells = cells.values().filter((cell) => state.getCell(cell).value === undefined);
            if (unfilledCells.length === 1) {
                const cell = unfilledCells[0];
                const missingValue = 45 - cells.values().reduce((acc, cell) => acc + (state.getCell(cell).value || 0), 0);
                this.setCellValue(state, cell, missingValue);
                console.log(`Full house: r${cell.row + 1}c${cell.column + 1}=${missingValue}`);
                return stopSymbol;
            }
        })
    }

    nakedSingle(state: SudokuState) {
        this.foreachHouse((cells) => {
            for (let cell of cells) {
                if (state.getCell(cell).value !== undefined) {
                    continue;
                }
                let pencilMarks = state.getCell(cell).pencilMarks;
                if (pencilMarks.size === 1) {
                    const value = [...pencilMarks][0];
                    console.log(`Naked single: r${cell.row + 1}c${cell.column + 1}=${value}`);
                    this.setCellValue(state, cell, value);
                    return stopSymbol;
                }
            }
        })
    }

    solveHiddenSingle(state: SudokuState) {
        this.foreachHouse((cells) => {
            for (let value = 1; value <= 9; value++) {
                let cellsWithPencilMark = cells.values().filter((cell) => state.getCell(cell).pencilMarks.has(value));
                if (cellsWithPencilMark.length === 1) {
                    const targetCell = state.getCell(cellsWithPencilMark[0]).position;
                    console.log(`Hidden single: r${targetCell.row + 1}c${targetCell.column + 1}=${value}`);
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
                    for (let cell of houseB.values()) {
                        if (!intersection.has(cell) && state.getCell(cell).pencilMarks.has(value)) {
                            state.getCell(cell).pencilMarks.delete(value);
                            return true
                        }
                    }
                }
            }
            return false
        }
        for (let block of this.cellsInBlocks) {
            for (let row of this.cellsInRows) {
                if (check(block, row) || check(row, block)) {
                    return
                }
            }
            for (let column of this.cellsInColumns) {
                if (check(block, column) || check(column, block)) {
                    return
                }
            }
        }
    }
}