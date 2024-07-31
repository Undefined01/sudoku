import { CellPosition, CellSet } from './sudoku';

export type ToggleSelectionEvent = {
    reference: CellPosition,
    cells: CellSet,
}