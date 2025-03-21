<script setup lang="ts">
import { onMounted, inject } from "vue";

import SudokuGrid from "./SudokuGrid.vue";
import SudokuCell from "./SudokuCell.vue";
import SudokuCellClick from "./SudokuCellClick.vue";
import SudokuHighlighter from "./SudokuHighlighter.vue";

import { Sudoku } from "@/models/sudoku";
import { defaultSettings, Settings } from "@/models/settings";
import SudokuCellBackground from "./SudokuCellBackground.vue";

const sudoku = inject<Sudoku>("sudoku")!;
const { metadata } = sudoku;
const { rows, columns } = metadata;

const settings = inject<Settings>("settings") ?? defaultSettings;
const sudokuMargin = settings.appearance.sudoku.sudokuSvgMargin;

onMounted(() => {
  document.addEventListener("keydown", function (event) {
    const key = event.code;

    if (sudoku.selectedCells.size === 0) {
      return;
    }

    switch (key) {
      case "Backspace":
      case "Delete":
        sudoku.updateState(true, (state) => {
          state.selectedCells.values().forEach((cell) => {
            const noModifier =
              !event.ctrlKey && !event.shiftKey && !event.altKey;
            if (noModifier) {
              state.getCell(cell).setValue(undefined);
            }
            if (noModifier || event.ctrlKey) {
              state.getCell(cell).candidates.clear();
            }
            if (noModifier || event.shiftKey) {
              state.getCell(cell).pencilMarks.clear();
            }
          });
        });
        break;
      case "Digit0":
      case "Digit1":
      case "Digit2":
      case "Digit3":
      case "Digit4":
      case "Digit5":
      case "Digit6":
      case "Digit7":
      case "Digit8":
      case "Digit9":
      case "Numpad0":
      case "Numpad1":
      case "Numpad2":
      case "Numpad3":
      case "Numpad4":
      case "Numpad5":
      case "Numpad6":
      case "Numpad7":
      case "Numpad8":
      case "Numpad9": {
        if (event.repeat) {
          break;
        }
        const value = parseInt(key.charAt(key.length - 1));
        sudoku.updateState(true, (state) => {
          if (!event.ctrlKey && !event.shiftKey && !event.altKey) {
            state.selectedCells.values().forEach((cell) => {
              state.getCell(cell).setValue(value);
            });
          }
          if (event.ctrlKey && !event.shiftKey && !event.altKey) {
            state.toggleCandidate(state.selectedCells.values(), value);
          }
          if (!event.ctrlKey && event.shiftKey && !event.altKey) {
            state.togglePencilMark(state.selectedCells.values(), value);
          }
        });
        break;
      }
      case "KeyZ":
        if (event.ctrlKey && !event.shiftKey && !event.altKey) {
          sudoku.undo();
        } else if (event.ctrlKey && event.shiftKey && !event.altKey) {
          sudoku.redo();
        }
        break;
      case "KeyY":
        if (event.ctrlKey && !event.shiftKey && !event.altKey) {
          sudoku.redo();
        }
        break;
      default:
        return;
    }

    event.preventDefault();
  });
});
</script>

<template>
  <svg
    class="sudoku-main"
    xmlns="http://www.w3.org/2000/svg"
    :viewBox="`${-sudokuMargin} ${-sudokuMargin} ${columns * 100 + sudokuMargin * 2} ${rows * 100 + sudokuMargin * 2}`"
    shape-rendering="geometricPrecision"
    pointer-events="none"
  >
    <SudokuCellBackground />

    <SudokuHighlighter :highlighted-cells="() => sudoku.selectedCells" />

    <SudokuGrid :metadata="sudoku.metadata" />

    <SudokuCell />

    <SudokuCellClick />
  </svg>
</template>

<style scoped></style>
