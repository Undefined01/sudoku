export type CellIndex = number
export type CellPosition = {
    row: number
    column: number
    idx: CellIndex
}

export class CellSet {
    cells: Map<CellIndex, CellPosition> = new Map()

    constructor(...cells: CellPosition[]) {
        cells.forEach(cell => this.cells.set(cell.idx, cell))
    }

    get size() {
        return this.cells.size
    }

    add(cell: CellPosition) {
        this.cells.set(cell.idx, cell)
    }

    has(cell: CellPosition) {
        return this.cells.has(cell.idx)
    }

    delete(cell: CellPosition) {
        this.cells.delete(cell.idx)
    }

    clear() {
        this.cells.clear()
    }

    equals(other: CellSet): boolean {
        if (this.size !== other.size) {
            return false
        }
        for (const cell of this.cells.values()) {
            if (!other.has(cell)) {
                return false
            }
        }
        return true
    }

    values(): Array<CellPosition> {
        return Array.from(this.cells.values())
    }
}

export class SudokuCell {
    position: CellPosition
    isGiven: boolean
    value?: number
    candidates: number[] = []
    pencilMarks: number[] = []

    constructor(options: { position: CellPosition, isGiven: boolean, value?: number }) {
        const { position, isGiven, value } = options
        this.position = position
        this.isGiven = isGiven
        this.value = value
    }

    setValue(value: number | undefined) {
        if (this.isGiven) {
            return
        }
        this.value = value
        this.candidates = []
        this.pencilMarks = []
    }

    toggleCandidate(candidate: number) {
        if (this.isGiven) {
            return
        }
        this.value = undefined
        const idx = this.candidates.indexOf(candidate)
        if (idx >= 0) {
            this.candidates.splice(idx, 1)
        } else {
            this.candidates.push(candidate)
        }
        console.log(this.candidates)
    }

    togglePencilMark(pencilMark: number) {
        if (this.isGiven) {
            return
        }
        this.value = undefined
        const idx = this.pencilMarks.indexOf(pencilMark)
        if (idx >= 0) {
            this.pencilMarks.splice(idx, 1)
        } else {
            this.pencilMarks.push(pencilMark)
        }
    }
}

export type SudokuMetadata = {
    title: string
    description: string

    rows: number
    columns: number

    decorations: SudokuDecorations
}

export class Sudoku {
    metadata: SudokuMetadata
    cells: SudokuCell[]

    constructor(rows: number, columns: number) {
        this.metadata = {
            title: 'Untitled',
            description: '',
            rows,
            columns,
            decorations: new SudokuDecorations(),
        }
        this.cells = Array.from({ length: rows * columns }, (_, idx) => new SudokuCell({
            position: {
                row: Math.floor(idx / columns),
                column: idx % columns,
                idx,
            },
            isGiven: false,
            value: undefined,
        }))
    }

    getCell(position: CellPosition) {
        return this.cells[position.idx]
    }

    getCellByRC(row: number, column: number) {
        return this.cells[row * this.metadata.columns + column]
    }
}

export class SudokuDecorations {
    boldRows: Array<number> = [0, 3, 6, 9]
    boldColumns: Array<number> = [0, 3, 6, 9]
}

export namespace Sudoku {
    // Import a Sudoku from a string representation like
    // .....6....637....22.....15.6..2.85....8...6....46.5..3.36.....11....328....1.....
    export function fromString(code: string, options: { rows?: number, columns?: number } = {}) {
        const rows = options.rows ?? 9
        const columns = options.columns ?? 9
        const sudoku = new Sudoku(rows, columns)
        for (let i = 0; i < code.length; i++) {
            const char = code[i]
            const cell = sudoku.cells[i]
            if (char !== '.') {
                cell.isGiven = true
                cell.value = Number(char)
            }
        }
        return sudoku
    }
}