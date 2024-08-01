<script setup lang="ts">
import { reactive, onMounted, provide, shallowReactive, shallowRef, watch } from 'vue'
import { freeze, immerable, produce } from "immer"

import SudokuGrid from './SudokuGrid.vue'
import SudokuCell from './SudokuCell.vue'
import SudokuCellClick from './SudokuCellClick.vue'
import SudokuHighlighter from './SudokuHighlighter.vue'

import { Sudoku, CellPosition, CellSet, SudokuState } from '@/models/sudoku'
import { defaultSettings } from '@/models/settings'

const sudoku = Sudoku.fromString(
  '.....6....637....22.....15.6..2.85....8...6....46.5..3.36.....11....328....1.....'
)

const settings = defaultSettings
provide('sudoku', sudoku)
provide('settings', settings)

const { metadata } = sudoku
const { rows, columns } = metadata
const sudokuMargin = settings.appearance.sudoku.sudokuSvgMargin

onMounted(() => {
  document.addEventListener('keydown', function (event) {
    const key = event.code;

    // console.log(event.key, event.code, event)

    if (sudoku.selectedCells.size === 0) {
      return;
    }

    switch (key) {
      case 'Backspace':
      case 'Delete':
        sudoku.updateState(true, state => {
          state.selectedCells.values().forEach(cell => {
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
        })
        break;
      case 'Digit0':
      case 'Digit1':
      case 'Digit2':
      case 'Digit3':
      case 'Digit4':
      case 'Digit5':
      case 'Digit6':
      case 'Digit7':
      case 'Digit8':
      case 'Digit9':
        if (event.repeat) {
          break;
        }
        const value = parseInt(key[5])
        sudoku.updateState(true, state => {
          if (!event.ctrlKey && !event.shiftKey && !event.altKey) {
            state.selectedCells.values().forEach(cell => {
              state.getCell(cell).setValue(value)
            })
          }
          if (event.ctrlKey && !event.shiftKey && !event.altKey) {
            state.selectedCells.values().forEach(cell => {
              state.getCell(cell).toggleCandidate(value)
            })
          }
          if (!event.ctrlKey && event.shiftKey && !event.altKey) {
            state.selectedCells.values().forEach(cell => {
              state.getCell(cell).togglePencilMark(value)
            })
          }
        })
        break;
      case 'KeyZ':
        if (event.ctrlKey && !event.shiftKey && !event.altKey) {
          sudoku.undo()
        } else if (event.ctrlKey && event.shiftKey && !event.altKey) {
          sudoku.redo()
        }
        break;
      case 'KeyY':
        if (event.ctrlKey && !event.shiftKey && !event.altKey) {
          sudoku.redo()
        }
        break;
    }
  });
})

</script>

<template>
  <svg class="sudoku-main" xmlns="http://www.w3.org/2000/svg" :width="columns * 100 + sudokuMargin * 2"
    :height="rows * 100 + sudokuMargin * 2"
    :viewBox="`${-sudokuMargin} ${-sudokuMargin} ${columns * 100 + sudokuMargin * 2} ${rows * 100 + sudokuMargin * 2}`"
    shape-rendering="geometricPrecision" pointer-events="none">

    <SudokuHighlighter :highlighted-cells="() => sudoku.selectedCells" />

    <SudokuGrid :metadata="sudoku.metadata" />

    <SudokuCell/>

    <SudokuCellClick/>

  </svg>
</template>

<style scoped></style>
