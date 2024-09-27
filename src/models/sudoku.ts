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

  constructor(options: { row: number; column: number; idx: CellIndex }) {
    const { row, column, idx } = options;
    this.row = row;
    this.column = column;
    this.idx = idx;
  }

  toString() {
    return `r${this.row + 1}c${this.column + 1}`;
  }
}

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
      this.candidates.delete(candidate);
    } else {
      this.candidates.add(candidate);
    }
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
      this.pencilMarks.delete(pencilMark);
    } else {
      this.pencilMarks.add(pencilMark);
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

  // Import sudoku from a string representation like
  // +----------------+------------------+-----------------+
  // |   9  234     7 | 23468 13468    5 | 248 123468 1234 |
  // |   1 2345  2345 |     7  3468  238 |   9  23468  234 |
  // |   8    6   234 |   234     9  123 |   5      7 1234 |
  // +----------------+------------------+-----------------+
  // | 457    8    45 |   234   347    6 |   1    234    9 |
  // |   3    1     6 |   248     5    9 | 248    248    7 |
  // |   2   47     9 |     1  3478  378 |   6      5   34 |
  // +----------------+------------------+-----------------+
  // | 457 3457 13458 |   358     2 1378 |  47      9    6 |
  // | 567    9  1235 |   356  1367    4 |  27     12    8 |
  // | 467  247  1248 |     9  1678  178 |   3    124    5 |
  // +----------------+------------------+-----------------+
  fromCandidateString(code: string) {
    const candidatesRegex = /\d+/g;
    const allCandidates = [];
    while (true) {
      const candidates = candidatesRegex.exec(code);
      if (!candidates) {
        break;
      }
      allCandidates.push(candidates[0]);
    }
    if (allCandidates.length !== 81) {
      throw new Error("Invalid candidate string");
    }
    for (let i = 0; i < allCandidates.length; i++) {
      const cell = this.cells[i];
      const candidates = allCandidates[i].split("").map(Number);
      if (candidates.length === 1) {
        cell.isGiven = true;
        cell.value = candidates[0];
      } else {
        cell.pencilMarks.clear();
        for (const candidate of candidates) {
          cell.pencilMarks.add(candidate);
        }
      }
    }
  }

  fillFromCandidateString(candidateString: string) {
    const candidatesRegex = /\d+/g;
    const allCandidates = [];
    while (true) {
      const candidates = candidatesRegex.exec(candidateString);
      if (!candidates) {
        break;
      }
      allCandidates.push(candidates[0]);
    }
    if (allCandidates.length !== 81) {
      throw new Error("Invalid candidate string");
    }
    for (let i = 0; i < allCandidates.length; i++) {
      const cell = this.cells[i];
      const candidates = allCandidates[i].split("").map(Number);
      if (cell.isGiven) {
        continue;
      }
      if (cell.value !== undefined) {
        continue;
      }
      if (candidates.length === 1) {
        cell.setValue(candidates[0]);
      } else {
        cell.pencilMarks.clear();
        for (const candidate of candidates) {
          cell.pencilMarks.add(candidate);
        }
      }
    }
  }

  toCandidateString(): string {
    return this.cells
      .map((cell) => {
        if (cell.isGiven) {
          return cell.value;
        }
        return Array.from(cell.pencilMarks).join("");
      })
      .join(" ");
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

  updateState(
    _immediateRecord: boolean,
    f: (state: SudokuState) => void,
  ): boolean {
    const newState = produce(this.self.state, f);
    if (this.self.state === newState) {
      return false;
    }
    this.self.stateHistory = this.self.stateHistory.slice(
      0,
      this.self.currentStateIndex + 1,
    );
    this.self.stateHistory.push(newState);
    this.self.currentStateIndex += 1;
    this.self.state = newState;
    return true;
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

  toValueString() {
    return this.self.state.cells
      .map((cell) => (cell.value === undefined ? "." : cell.value))
      .join("");
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
