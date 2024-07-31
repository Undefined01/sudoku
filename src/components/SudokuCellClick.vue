<script setup lang="ts">
import { SudokuMetadata, SudokuCell, CellPosition,CellSet } from '@/models/sudoku'

const { metadata, cells } = defineProps<{ metadata: SudokuMetadata, cells: SudokuCell[] }>()
const { rows, columns } = metadata
const emit = defineEmits<{
  (e: 'set-selection', cells: CellSet, resolve: (ret: boolean) => void): void
  (e: 'toggle-selection', reference: CellPosition, cells: CellSet, resolve: (ret: boolean) => void, isSet: boolean | undefined): void
}>()

function emitToggleSelection(event: MouseEvent, reference: CellPosition, cells: CellSet, isSet: boolean | undefined = undefined): Promise<boolean> {
  return new Promise((resolve) => {
    if (!event.ctrlKey && !event.shiftKey) {
      return emit('set-selection', cells, resolve)
    } else {
      return emit('toggle-selection', reference, cells, resolve, isSet)
    }
  })
}

// function toggleSelection(cellPosition: CellPosition, event: MouseEvent) {
//   emitToggleSelection(event, cellPosition, new CellSet(cellPosition))
// }

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
  emitToggleSelection(event, cellPosition, cellsWithSameNumber)
}

let isMultiselecting = false
let multiselectedCells: CellSet = new CellSet()
let multiselectMode: 'add' | 'remove' = 'add'
let longPressTimer: number | null = null

async function startMultiselect(cellPosition: CellPosition, event: MouseEvent) {
  if (event.buttons !== 1) {
    return
  }
  isMultiselecting = true
  multiselectedCells.clear()
  multiselectedCells.add(cellPosition)
  const emitRet = await emitToggleSelection(event, cellPosition, new CellSet(cellPosition))
  longPressTimer = setTimeout(() => {
    console.log('long press')
    const cellsWithSameNumber = getCellsWithSameNumber(cellPosition)
    if (cellsWithSameNumber !== undefined) {
      cellsWithSameNumber.values().forEach(cell => {
        multiselectedCells.add(cell)
      })
      emitToggleSelection(event, cellPosition, multiselectedCells, emitRet)
    }
  }, 600)
  multiselectMode = emitRet ? 'add' : 'remove'
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
  if (multiselectMode === 'add') {
    emitToggleSelection(event, cellPosition, multiselectedCells, true)
  } else {
    emitToggleSelection(event, cellPosition, multiselectedCells, false)
  }
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
          @mouseup="endMultiselect"
           />
      </template>
    </template>
  </g>
</template>

<style scoped>
.sudoku-cell-text {
  user-select: none;
}
</style>
