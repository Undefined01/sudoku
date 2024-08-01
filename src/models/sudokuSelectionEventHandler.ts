import { CellPosition, CellSet } from "./sudoku"

export type SudokuHandleMode = 'set' | 'clear'

export type SudokuSelectionEvent = {
    reference?: CellPosition
    cells: CellSet
    clearPreviousSelection?: boolean
    mode?: SudokuHandleMode
}

export interface SudokuSelectionEventHandler {
    setSelection: (event: SudokuSelectionEvent) => SudokuHandleMode
}

export class SelectionEventHandlerForSelectedCells {
    selectedCells: CellSet
    constructor(selectedCells: CellSet) {
        this.selectedCells = selectedCells
    }

    setSelection(event: SudokuSelectionEvent): SudokuHandleMode {
        const { reference, cells, clearPreviousSelection } = event
        let { mode } = event
        if (reference !== undefined && this.selectedCells.has(reference)) {
            mode = 'clear'
        }
        if (clearPreviousSelection) {
            this.selectedCells.clear()
        }
        if (mode === undefined) {
            mode = 'set'
        }
        if (mode === 'set') {
            cells.forEach(cell => this.selectedCells.add(cell))
        } else {
            cells.forEach(cell => this.selectedCells.delete(cell))
        }
        return mode
    }
}
