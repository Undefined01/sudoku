import { CellPosition, Sudoku, SudokuState } from "./sudoku";

const stopSymbol = Symbol("stop");
type StopSymbol = typeof stopSymbol;

export class SudokuSolver {
    sudoku: Sudoku;

    cellsInHouses: CellPosition[][]
    cellsInRows: CellPosition[][]
    cellsInColumns: CellPosition[][]

    constrainsOfCell: Map<number, CellPosition[][]> = new Map();

    constructor(sudoku: Sudoku) {
        this.sudoku = sudoku;
        if (sudoku.cells.length !== 81) {
            throw new Error("Invalid sudoku size");
        }

        this.cellsInHouses = [];
        for (let houseX of [0, 3, 6]) {
            for (let houseY of [0, 3, 6]) {
                let house: CellPosition[] = []
                for (let x of [0, 1, 2]) {
                    for (let y of [0, 1, 2]) {
                        const pos = this.sudoku.getCellPosition(houseX + x, houseY + y);
                        house.push(pos);
                        if (!this.constrainsOfCell.has(pos.idx)) {
                            this.constrainsOfCell.set(pos.idx, []);
                        }
                        this.constrainsOfCell.get(pos.idx)!.push(house);
                    }
                }
                this.cellsInHouses.push(house)
            }
        }
        this.cellsInRows = [];
        for (let row = 0; row < 9; row++) {
            let cells: CellPosition[] = []
            for (let column = 0; column < 9; column++) {
                const pos = this.sudoku.getCellPosition(row, column);
                cells.push(pos);
                if (!this.constrainsOfCell.has(pos.idx)) {
                    this.constrainsOfCell.set(pos.idx, []);
                }
                this.constrainsOfCell.get(pos.idx)!.push(cells);
            }
            this.cellsInRows.push(cells);
        }
        this.cellsInColumns = [];
        for (let column = 0; column < 9; column++) {
            let cells: CellPosition[] = []
            for (let row = 0; row < 9; row++) {
                const pos = this.sudoku.getCellPosition(row, column);
                cells.push(pos);
                if (!this.constrainsOfCell.has(pos.idx)) {
                    this.constrainsOfCell.set(pos.idx, []);
                }
                this.constrainsOfCell.get(pos.idx)!.push(cells);
            }
            this.cellsInColumns.push(cells);
        }
    }

    private foreachConstraint(f: (cells: CellPosition[]) => void | StopSymbol | undefined) {
        for (let cellsInHouse of this.cellsInHouses) {
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
        this.foreachConstraint((cells) => {
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
                cell.pencilMarks = [1, 2, 3, 4, 5, 6, 7, 8, 9];
            }

            this.foreachConstraint((cells) => {
                let values = cells.map((cell) => state.getCell(cell).value);
                for (let cell of cells) {
                    let pencilMarks = state.getCell(cell).pencilMarks;
                    if (state.getCell(cell).value !== undefined) {
                        continue;
                    }
                    state.getCell(cell).pencilMarks = pencilMarks.filter((value) => !values.includes(value));
                }
            })
        });
    }
    
    solveOneStep() {
        const solvers = [this.nakedSingle, this.solveHiddenSingle.bind(this)];
        for (let solver of solvers) {
            if (solver()) {
                return true;
            }
        }
        return false
    }

    private setCellValue(state: SudokuState, cell: CellPosition, value: number) {
        state.getCell(cell).setValue(value);
        for (let constraint of this.constrainsOfCell.get(cell.idx)!) {
            for (let otherCell of constraint) {
                if (otherCell === cell) {
                    continue;
                }
                state.getCell(otherCell).pencilMarks = state.getCell(otherCell).pencilMarks.filter((v) => v !== value);
            }
        }
    }

    nakedSingle(): boolean {
        return this.sudoku.updateState(false, (state) => {
            this.foreachConstraint((cells) => {
                for (let cell of cells) {
                    if (state.getCell(cell).value !== undefined) {
                        continue;
                    }
                    let pencilMarks = state.getCell(cell).pencilMarks;
                    if (pencilMarks.length === 1) {
                        const value = pencilMarks[0];
                        console.log(`Naked single: r${cell.row + 1}c${cell.column + 1}=${value}`);
                        this.setCellValue(state, cell, value);
                        return stopSymbol;
                    }
                }
            })
        })
    }

    solveHiddenSingle(): boolean {
        return this.sudoku.updateState(false, (state) => {
            this.foreachConstraint((cells) => {
                for (let value = 1; value <= 9; value++) {
                    let cellsWithPencilMark = cells.filter((cell) => state.getCell(cell).pencilMarks.includes(value));
                    if (cellsWithPencilMark.length === 1) {
                        const targetCell = state.getCell(cellsWithPencilMark[0]).position;
                        console.log(`Hidden single: r${targetCell.row + 1}c${targetCell.column + 1}=${value}`);
                        this.setCellValue(state, targetCell, value);
                        return stopSymbol;
                    }
                }
            })
        })
    }
}