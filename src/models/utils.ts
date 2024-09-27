import { immerable } from "immer";
import { CellPosition } from "./sudoku";

export class CellSet {
  [immerable] = true;
  static bitsetForCell: Array<bigint> = Array.from(
    { length: 81 },
    (_, idx) => BigInt(1) << BigInt(idx),
  );
  static deleteMaskForCell: Array<bigint> = Array.from(
    { length: 81 },
    (_, idx) => ~CellSet.bitsetForCell[idx],
  );
  static bitint0: bigint = BigInt(0);
  static bitint32: bigint = BigInt(32);
  static bitint64: bigint = BigInt(64);
  static bitset32Mask: bigint = BigInt(0xffffffff);
  static cells: Array<CellPosition> = Array.from(
    { length: 81 },
    (_, idx) =>
      new CellPosition({
        row: Math.floor(idx / 9),
        column: idx % 9,
        idx,
      }),
  );

  cells: Array<CellPosition> | undefined = undefined;
  bitset: bigint;
  name: string = "";

  constructor(value: bigint = BigInt(0)) {
    this.bitset = value;
  }

  static fromPositions(positions: Array<CellPosition>): CellSet {
    const set = new CellSet();
    for (const cell of positions) {
      set.add(cell);
    }
    set.cells = positions;
    return set;
  }

  static popcnt32(n: number): number {
    n = n - ((n >> 1) & 0x55555555);
    n = ((n & 0x33333333) + (n >> 2)) & 0x33333333;
    n = (n + (n >> 4)) & 0x0f0f0f0f;
    return (n * 0x01010101) >> 24;
  }

  isEmpty() {
    return this.bitset === CellSet.bitint0;
  }

  get size() {
    let cnt = CellSet.popcnt32(Number(this.bitset & CellSet.bitset32Mask));
    cnt += CellSet.popcnt32(
      Number((this.bitset >> CellSet.bitint32) & CellSet.bitset32Mask),
    );
    cnt += CellSet.popcnt32(
      Number((this.bitset >> CellSet.bitint64) & CellSet.bitset32Mask),
    );
    return cnt;
  }

  add(cell: CellPosition) {
    this.cells = undefined;
    this.bitset |= CellSet.bitsetForCell[cell.idx];
  }

  has(cell: CellPosition) {
    return this.bitset & CellSet.bitsetForCell[cell.idx] ? true : false;
  }

  delete(cell: CellPosition) {
    this.cells = undefined;
    this.bitset &= CellSet.deleteMaskForCell[cell.idx];
  }

  clear() {
    this.cells = undefined;
    this.bitset = BigInt(0);
  }

  equals(other: CellSet): boolean {
    return this.bitset === other.bitset;
  }

  values(): Array<CellPosition> {
    if (this.cells) {
      return this.cells;
    }
    // this.cells = [];
    const cells: Array<CellPosition> = [];
    const valuesForNumber32 = (
      n: number,
      shift: number,
    ) => {
      for (let idx = 0; idx < 32; idx++) {
        if (n & (1 << idx)) {
          cells.push(CellSet.cells[idx + shift]);
        }
      }
    };
    valuesForNumber32(
      Number(this.bitset & CellSet.bitset32Mask),
      0,
    );
    valuesForNumber32(
      Number((this.bitset >> CellSet.bitint32) & CellSet.bitset32Mask),
      32,
    );
    valuesForNumber32(
      Number((this.bitset >> CellSet.bitint64) & CellSet.bitset32Mask),
      64,
    );
    return cells;
  }

  forEach(callback: (cell: CellPosition) => void) {
    this.values().forEach(callback);
  }

  isSubsetOf(other: CellSet): boolean {
    return (this.bitset & other.bitset) === this.bitset;
  }

  // this - other
  substract(other: CellSet): CellSet {
    return new CellSet(this.bitset & ~other.bitset);
  }

  union(other: CellSet): CellSet {
    return new CellSet(this.bitset | other.bitset);
  }

  intersection(other: CellSet): CellSet {
    return new CellSet(this.bitset & other.bitset);
  }

  static union(...sets: CellSet[]): CellSet {
    const union = new CellSet();
    sets.forEach((set) => {
      union.bitset |= set.bitset;
    });
    return union;
  }

  static intersection(...sets: CellSet[]): CellSet {
    if (sets.length === 0) {
      throw new Error("No sets to intersect");
    }
    const intersection = new CellSet(sets[0].bitset);
    sets.forEach((set) => {
      intersection.bitset &= set.bitset;
    });
    return intersection;
  }

  [Symbol.iterator]() {
    return this.values()[Symbol.iterator]();
  }

  toString() {
    return this.values()
      .map((p) => `r${p.row + 1}c${p.column + 1}`)
      .join(",");
  }
}
