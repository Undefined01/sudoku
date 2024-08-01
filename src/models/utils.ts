import { immerable } from "immer";
import { CellIndex, CellPosition } from "./sudoku";

export class CellSet {
  [immerable] = true;
  cells: Map<CellIndex, CellPosition> = new Map();

  constructor(...cells: CellPosition[]) {
    cells.forEach((cell) => this.cells.set(cell.idx, cell));
  }

  get size() {
    return this.cells.size;
  }

  add(cell: CellPosition) {
    this.cells.set(cell.idx, cell);
  }

  has(cell: CellPosition) {
    return this.cells.has(cell.idx);
  }

  delete(cell: CellPosition) {
    this.cells.delete(cell.idx);
  }

  clear() {
    this.cells.clear();
  }

  equals(other: CellSet): boolean {
    if (this.size !== other.size) {
      return false;
    }
    for (const cell of this.cells.values()) {
      if (!other.has(cell)) {
        return false;
      }
    }
    return true;
  }

  values(): Array<CellPosition> {
    return Array.from(this.cells.values());
  }

  forEach(callback: (cell: CellPosition) => void) {
    this.cells.forEach((cell) => callback(cell));
  }

  intersection(other: CellSet): CellSet {
    const intersection = new CellSet();
    for (const cell of this.cells.values()) {
      if (other.has(cell)) {
        intersection.add(cell);
      }
    }
    return intersection;
  }

  [Symbol.iterator]() {
    return this.cells.values();
  }
}
