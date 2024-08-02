import { immerable } from "immer";
import { CellIndex, CellPosition } from "./sudoku";

export class CellSet {
  [immerable] = true;
  cells: Map<CellIndex, CellPosition> = new Map();
  name: string = "";

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

  isSubsetOf(other: CellSet): boolean {
    for (const cell of this.cells.values()) {
      if (!other.has(cell)) {
        return false;
      }
    }
    return true;
  }

  // this - other
  substract(other: CellSet): CellSet {
    const difference = new CellSet();
    for (const cell of this.cells.values()) {
      if (!other.has(cell)) {
        difference.add(cell);
      }
    }
    return difference;
  }

  union(other: CellSet): CellSet {
    const union = new CellSet();
    for (const cell of this.cells.values()) {
      union.add(cell);
    }
    for (const cell of other.cells.values()) {
      union.add(cell);
    }
    return union;
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

  static union(...sets: CellSet[]): CellSet {
    const union = new CellSet();
    sets.forEach((set) => {
      set.forEach((cell) => union.add(cell));
    });
    return union;
  }

  static intersection(...sets: CellSet[]): CellSet {
    const intersection = new CellSet();
    const [first, ...rest] = sets;
    first.forEach((cell) => {
      if (rest.every((set) => set.has(cell))) {
        intersection.add(cell);
      }
    });
    return intersection;
  }

  [Symbol.iterator]() {
    return this.cells.values();
  }

  toString() {
    return this.values().map(p => `r${p.row + 1}c${p.column + 1}`).join(',')
  }
}
