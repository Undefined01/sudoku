import { immerable, produce } from "immer";
import {
  SelectionEventHandlerForSelectedCells,
  SudokuSelectionEventHandler,
} from "./sudokuSelectionEventHandler";
import { CellSet } from "./utils";
import { shallowReactive, ShallowReactive } from "vue";
export { CellSet };

export type CellIndex = number;
export class CellPosition {
  row: number;
  column: number;
  idx: CellIndex;

  constructor(options: {row: number, column: number, idx: CellIndex}) {
    const { row, column, idx } = options;
    this.row = row;
    this.column = column;
    this.idx = idx;
  }

  toString() {
    return `r${this.row + 1}c${this.column + 1}`;
  }
};

export class SudokuCell {
  [immerable] = true;
  readonly position: CellPosition;
  isGiven: boolean;
  value?: number;
  candidates = new Set<number>();
  pencilMarks = new Set<number>();

  constructor(options: {
    position: CellPosition;
    isGiven: boolean;
    value?: number;
  }) {
    const { position, isGiven, value } = options;
    this.position = position;
    this.isGiven = isGiven;
    this.value = value;
  }

  setValue(value: number | undefined) {
    if (this.isGiven) {
      return;
    }
    this.value = value;
    this.candidates = new Set();
    this.pencilMarks = new Set();
  }

  toggleCandidate(candidate: number) {
    if (this.isGiven) {
      return;
    }
    this.value = undefined;
    if (this.candidates.has(candidate)) {
      this.candidates.delete(candidate)
    } else {
      this.candidates.add(candidate)
    }
    console.log(this.candidates);
  }

  clearCandidates() {
    this.candidates.clear();
  }

  togglePencilMark(pencilMark: number) {
    if (this.isGiven) {
      return;
    }
    this.value = undefined;
    if (this.pencilMarks.has(pencilMark)) {
      this.pencilMarks.delete(pencilMark)
    } else {
      this.pencilMarks.add(pencilMark)
    }
  }

  clearPencilMarks() {
    this.pencilMarks.clear();
  }
}

export type SudokuMetadata = {
  title: string;
  description: string;

  rows: number;
  columns: number;

  decorations: SudokuDecorations;
};

export class SudokuState {
  [immerable] = true;
  cells: SudokuCell[];
  selectedCells: CellSet = new CellSet();

  constructor(cells: SudokuCell[]) {
    this.cells = cells;
  }

  getCells(): SudokuCell[] {
    return this.cells;
  }

  getCell(position: CellPosition) {
    return this.cells[position.idx];
  }

  getCellByIdx(idx: number) {
    return this.cells[idx];
  }
}

export class Sudoku {
  self: ShallowReactive<Sudoku>;
  metadata: SudokuMetadata;
  state: SudokuState;
  currentStateIndex: number = 0;
  stateHistory: Array<SudokuState>;
  selectionEventHandler: SudokuSelectionEventHandler;

  constructor(rows: number, columns: number) {
    this.metadata = {
      title: "Untitled",
      description: "",
      rows,
      columns,
      decorations: new SudokuDecorations(),
    };
    const cells = Array.from(
      { length: rows * columns },
      (_, idx) =>
        new SudokuCell({
          position: new CellPosition({
            row: Math.floor(idx / columns),
            column: idx % columns,
            idx,
          }),
          isGiven: false,
          value: undefined,
        }),
    );
    this.state = new SudokuState(cells);
    this.stateHistory = [this.state];
    this.selectionEventHandler = new SelectionEventHandlerForSelectedCells(
      this,
    );

    this.self = shallowReactive(this);
  }

  get cells() {
    return this.self.state.cells;
  }

  get selectedCells() {
    return this.self.state.selectedCells;
  }

  getCell(position: CellPosition) {
    return this.self.state.getCell(position);
  }

  getCellByIdx(idx: number) {
    return this.self.state.getCellByIdx(idx);
  }

  getCellPosition(row: number, column: number): CellPosition {
    return this.self.state.cells[row * this.self.metadata.columns + column]
      .position;
  }

  updateState(_immediateRecord: boolean, f: (state: SudokuState) => void): boolean {
    const newState = produce(this.self.state, f);
    if (this.self.state === newState) {
        return false
    }
    this.self.stateHistory = this.self.stateHistory.slice(
      0,
      this.self.currentStateIndex + 1,
    );
    this.self.stateHistory.push(newState);
    this.self.currentStateIndex += 1;
    this.self.state = newState;
    return true
  }

  undo(count: number = 1) {
    if (this.self.currentStateIndex - count < 0) {
      return;
    }
    this.self.currentStateIndex -= count;
    this.self.state = this.self.stateHistory[this.self.currentStateIndex];
  }

  redo(count: number = 1) {
    if (this.self.currentStateIndex + count >= this.self.stateHistory.length) {
      return;
    }
    this.self.currentStateIndex += count;
    this.self.state = this.self.stateHistory[this.self.currentStateIndex];
  }

  // Import a Sudoku from a string representation like
  // .....6....637....22.....15.6..2.85....8...6....46.5..3.36.....11....328....1.....
  static fromString(
    code: string,
    options: { rows?: number; columns?: number } = {},
  ) {
    const rows = options.rows ?? 9;
    const columns = options.columns ?? 9;
    const sudoku = new Sudoku(rows, columns);
    for (let i = 0; i < code.length; i++) {
      const char = code[i];
      if (char !== ".") {
        const cellPosition = sudoku.getCellPosition(
          Math.floor(i / columns),
          i % columns,
        );
        const cell = sudoku.state.getCell(cellPosition);
        cell.isGiven = true;
        cell.value = Number(char);
      }
    }
    return sudoku.self;
  }
}

export class SudokuDecorations {
  boldRows: Array<number> = [0, 3, 6, 9];
  boldColumns: Array<number> = [0, 3, 6, 9];
}
