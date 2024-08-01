<script setup lang="ts">
import { reactive, onMounted, provide } from 'vue'
import SudokuGrid from './SudokuGrid.vue'
import SudokuCell from './SudokuCell.vue'
import SudokuCellClick from './SudokuCellClick.vue'
import SudokuHighlighter from './SudokuHighlighter.vue'

import { Sudoku, CellPosition, CellSet } from '@/models/sudoku'

const sudoku = reactive(Sudoku.fromString(
  '.....6....637....22.....15.6..2.85....8...6....46.5..3.36.....11....328....1.....'
))

provide('sudoku', { sudoku })

const { rows, columns } = sudoku.metadata
const { state, } = sudoku
const { selectedCells } = state

const sudokuMargin = 10

function setSelection(cells: CellSet, resolve: (ret: boolean) => void) {
  if (cells.equals(selectedCells)) {
    selectedCells.clear()
    resolve(false)
  } else {
    selectedCells.clear()
    cells.values().forEach(cell => {
      selectedCells.add(cell)
    })
    resolve(true)
  }
}

function toggleSelection(reference: CellPosition, cells: CellSet, resolve: (ret: boolean) => void, isSet: boolean | undefined = undefined) {
  if (isSet !== true && selectedCells.has(reference)) {
    cells.values().forEach(cell => {
      selectedCells.delete(cell)
    })
    resolve(false)
  } else {
    cells.values().forEach(cell => {
      selectedCells.add(cell)
    })
    resolve(true)
  }
}

onMounted(() => {
  document.addEventListener('keydown', function (event) {
    const keysToPrevent = [
      'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12',
      'Escape',
      'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
      'Backspace', 'Tab', 'Enter', 'Space',
      'PageUp', 'PageDown', 'End', 'Home', 'Insert', 'Delete'
    ];

    const key = event.keyCode;

    if (event.ctrlKey || event.altKey || event.shiftKey) {
      event.preventDefault();
    }
    if (keysToPrevent.includes(event.key)) {
      event.preventDefault();
    }


    if (selectedCells.size === 0) {
      return;
    }

    if (key === 46) { // Delete
      selectedCells.values().forEach(cell => {
        const noModifier = !event.ctrlKey && !event.shiftKey && !event.altKey;
        if (noModifier) {
          state.getCell(cell).setValue(undefined)
        }
        if (noModifier || event.ctrlKey) {
          state.getCell(cell).candidates = []
        }
        if (noModifier || event.shiftKey) {
          state.getCell(cell).pencilMarks = []
        }
      })
    }

    if (key >= 48 && key <= 57) { // 0-9
      const value = key - 48;
      if (!event.ctrlKey && !event.shiftKey && !event.altKey) {
        selectedCells.values().forEach(cell => {
          state.getCell(cell).setValue(value)
        })
      }
      if (event.ctrlKey && !event.shiftKey && !event.altKey) {
        selectedCells.values().forEach(cell => {
          state.getCell(cell).toggleCandidate(value)
        })
      }
      if (!event.ctrlKey && event.shiftKey && !event.altKey) {
        selectedCells.values().forEach(cell => {
          state.getCell(cell).togglePencilMark(value)
        })
      }
    }
  });
})

</script>

<template>
  <svg class="sudoku-main" xmlns="http://www.w3.org/2000/svg" :width="columns * 100 + sudokuMargin * 2"
    :height="rows * 100 + sudokuMargin * 2"
    :viewBox="`${-sudokuMargin} ${-sudokuMargin} ${columns * 100 + sudokuMargin * 2} ${rows * 100 + sudokuMargin * 2}`"
    shape-rendering="geometricPrecision" pointer-events="none">

    <SudokuHighlighter :highlighted-cells="selectedCells" />

    <SudokuGrid :metadata="sudoku.metadata" />

    <SudokuCell :cells="sudoku.state.cells" />

    <SudokuCellClick :metadata="sudoku.metadata" :cells="sudoku.state.cells" @set-selection="setSelection"
      @toggle-selection="toggleSelection" />

  </svg>
</template>

<style scoped></style>
