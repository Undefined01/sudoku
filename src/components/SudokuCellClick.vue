<script setup lang="ts">
import { inject } from 'vue';
import { Sudoku, SudokuMetadata, SudokuCell, CellPosition } from '@/models/sudoku'
import { CellSet } from '@/models/utils'
import { SudokuInjection } from '@/models/injection'
import { SudokuHandleMode } from '@/models/sudokuSelectionEventHandler';

const { sudoku } = inject<SudokuInjection>('sudoku')!
const { metadata, state, selectionEventHandler } = sudoku
const { rows, columns } = metadata
const { cells } = state


function getCellsWithSameNumber(cellPosition: CellPosition): CellSet | undefined {
  const selectedNumber = cells[cellPosition.idx].value
  if (selectedNumber === undefined) {
    return undefined
  }
  const cellsWithSameNumber = new CellSet()
  for (let cell of cells) {
    if (cell.value === selectedNumber) {
      cellsWithSameNumber.add(cell.position)
    }
  }
  return cellsWithSameNumber
}

function selectSameNumber(cellPosition: CellPosition, event: MouseEvent) {
  const cellsWithSameNumber = getCellsWithSameNumber(cellPosition)
  if (cellsWithSameNumber === undefined) {
    return
  }
  selectionEventHandler.setSelection({
    reference: cellPosition,
    cells: cellsWithSameNumber,
    clearPreviousSelection: !event.ctrlKey && !event.shiftKey
  })
}

let isMultiselecting = false
let multiselectedCells: CellSet = new CellSet()
let multiselectMode: SudokuHandleMode = 'set'
let longPressTimer: number | null = null

async function startMultiselect(cellPosition: CellPosition, event: MouseEvent) {
  if (event.buttons !== 1) {
    return
  }
  isMultiselecting = true
  multiselectedCells.clear()
  multiselectedCells.add(cellPosition)
  const selectionMode = selectionEventHandler.setSelection({
    reference: cellPosition,
    cells: new CellSet(cellPosition),
    clearPreviousSelection: !event.ctrlKey && !event.shiftKey
  })
  longPressTimer = setTimeout(() => {
    console.log('long press')
    const cellsWithSameNumber = getCellsWithSameNumber(cellPosition)
    if (cellsWithSameNumber !== undefined) {
      cellsWithSameNumber.values().forEach(cell => {
        multiselectedCells.add(cell)
      })
      selectionEventHandler.setSelection({
        reference: cellPosition,
        cells: new CellSet(cellPosition),
        clearPreviousSelection: !event.ctrlKey && !event.shiftKey,
        mode: selectionMode,
      })
    }
  }, 600)
  multiselectMode = selectionMode
}

function doMultiselect(cellPosition: CellPosition, event: MouseEvent) {
  if (!isMultiselecting) {
    return
  }
  if (event.buttons === 0) {
    endMultiselect()
    return
  }
  if (multiselectedCells.has(cellPosition)) {
    return
  }
  if (longPressTimer !== null) {
    clearTimeout(longPressTimer)
    longPressTimer = null
  }
  multiselectedCells.add(cellPosition)
  selectionEventHandler.setSelection({
    reference: cellPosition,
    cells: multiselectedCells,
    mode: multiselectMode,
  })
}

function endMultiselect() {
  isMultiselecting = false
  if (longPressTimer !== null) {
    clearTimeout(longPressTimer)
    longPressTimer = null
  }
}

</script>

<template>
  <g>
    <template v-for="(_, row) in rows">
      <template v-for="(_, column) in columns">
        <rect :x="column * 100" :y="row * 100" width="100" height="100" fill="transparent"
          pointer-events="visiblePainted"
          @dblclick="event => selectSameNumber({ row, column, idx: row * columns + column }, event)"
          @mousedown="event => startMultiselect({ row, column, idx: row * columns + column }, event)"
          @mousemove="event => doMultiselect({ row, column, idx: row * columns + column }, event)"
          @mouseup="endMultiselect" />
      </template>
    </template>
  </g>
</template>

<style scoped>
.sudoku-cell-text {
  user-select: none;
}
</style>
