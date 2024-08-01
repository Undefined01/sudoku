import { CellPosition, CellSet, Sudoku } from "./sudoku";

export type SudokuHandleMode = "set" | "clear";

export type SudokuSelectionEvent = {
  reference?: CellPosition;
  cells: CellSet;
  clearPreviousSelection?: boolean;
  mode?: SudokuHandleMode;
};

export interface SudokuSelectionEventHandler {
  setSelection: (event: SudokuSelectionEvent) => SudokuHandleMode;
}

export class SelectionEventHandlerForSelectedCells {
  sudoku: Sudoku;
  constructor(sudoku: Sudoku) {
    this.sudoku = sudoku;
  }

  setSelection(event: SudokuSelectionEvent): SudokuHandleMode {
    const { reference, cells, clearPreviousSelection } = event;
    let { mode } = event;
    this.sudoku.updateState(false, (state) => {
      if (reference !== undefined && state.selectedCells.has(reference)) {
        mode = "clear";
      }
      if (clearPreviousSelection) {
        state.selectedCells.clear();
      }
      if (mode === undefined) {
        mode = "set";
      }
      if (mode === "set") {
        cells.forEach((cell) => state.selectedCells.add(cell));
      } else {
        cells.forEach((cell) => state.selectedCells.delete(cell));
      }
    });
    return mode!;
  }
}
