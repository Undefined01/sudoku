<script setup lang="ts">
import { inject } from "vue";
import { Sudoku, CellPosition } from "@/models/sudoku";
import { CellSet } from "@/models/utils";
import { SudokuHandleMode } from "@/models/sudokuSelectionEventHandler";

const sudoku = inject<Sudoku>("sudoku")!;
const { metadata, selectionEventHandler } = sudoku;
const { rows, columns } = metadata;

function getCellsWithSameNumber(
  cellPosition: CellPosition,
): CellSet | undefined {
  const selectedNumber = sudoku.getCell(cellPosition).value;
  if (selectedNumber === undefined) {
    return undefined;
  }
  const cellsWithSameNumber = new CellSet();
  for (let cell of sudoku.cells) {
    if (cell.value === selectedNumber) {
      cellsWithSameNumber.add(cell.position);
    }
  }
  return cellsWithSameNumber;
}

function selectSameNumber(cellPosition: CellPosition, event: MouseEvent) {
  const cellsWithSameNumber = getCellsWithSameNumber(cellPosition);
  if (cellsWithSameNumber === undefined) {
    return;
  }
  selectionEventHandler.setSelection({
    reference: cellPosition,
    cells: cellsWithSameNumber,
    clearPreviousSelection: !event.ctrlKey && !event.shiftKey,
  });
}

let isMultiselecting = false;
let multiselectedCells: CellSet = new CellSet();
let multiselectMode: SudokuHandleMode = "set";
let longPressTimer: number | null = null;

async function startMultiselect(
  cellPosition: CellPosition,
  event: MouseEvent | TouchEvent,
) {
  const mouseEvent = event as MouseEvent;
  if (mouseEvent.buttons !== undefined && mouseEvent.buttons !== 1) {
    return;
  }
  isMultiselecting = true;
  multiselectedCells.clear();
  multiselectedCells.add(cellPosition);
  const selectionMode = selectionEventHandler.setSelection({
    reference: cellPosition,
    cells: CellSet.fromPositions([cellPosition]),
    clearPreviousSelection: !event.ctrlKey && !event.shiftKey,
  });
  longPressTimer = setTimeout(() => {
    const cellsWithSameNumber = getCellsWithSameNumber(cellPosition);
    if (cellsWithSameNumber !== undefined) {
      cellsWithSameNumber.values().forEach((cell) => {
        multiselectedCells.add(cell);
      });
      selectionEventHandler.setSelection({
        reference: cellPosition,
        cells: CellSet.fromPositions([cellPosition]),
        clearPreviousSelection: !event.ctrlKey && !event.shiftKey,
        mode: selectionMode,
      });
    }
  }, 600);
  multiselectMode = selectionMode;
}

function doMultiselect(
  cellPosition: CellPosition,
  event: MouseEvent | TouchEvent,
) {
  if (!isMultiselecting) {
    return;
  }
  const mouseEvent = event as MouseEvent;
  if (mouseEvent.buttons === 0) {
    endMultiselect();
    return;
  }
  if (multiselectedCells.has(cellPosition)) {
    return;
  }
  if (longPressTimer !== null) {
    clearTimeout(longPressTimer);
    longPressTimer = null;
  }
  multiselectedCells.add(cellPosition);
  selectionEventHandler.setSelection({
    reference: cellPosition,
    cells: multiselectedCells,
    mode: multiselectMode,
  });
}

function endMultiselect() {
  isMultiselecting = false;
  if (longPressTimer !== null) {
    clearTimeout(longPressTimer);
    longPressTimer = null;
  }
}

function handleTouchMove(event: TouchEvent) {
  if (event.touches.length !== 1) {
    endMultiselect();
    return;
  }
  const touch = event.touches[0];
  const element = document.elementFromPoint(touch.clientX, touch.clientY);
  if (element === null || !element.classList.contains("sudoku-cell")) {
    return;
  }
  const row = parseInt(element.getAttribute("y")!) / 100;
  const column = parseInt(element.getAttribute("x")!) / 100;
  console.log("touchmove", row, column);
  doMultiselect({ row, column, idx: row * columns + column }, event);
}
</script>

<template>
  <g>
    <template v-for="(_, row) in rows" :key="row">
      <template v-for="(_, column) in columns" :key="column">
        <rect
          class="sudoku-cell"
          :x="column * 100"
          :y="row * 100"
          width="100"
          height="100"
          fill="transparent"
          pointer-events="visiblePainted"
          @dblclick="
            (event) =>
              selectSameNumber(
                { row, column, idx: row * columns + column },
                event,
              )
          "
          @mousedown="
            (event) =>
              startMultiselect(
                { row, column, idx: row * columns + column },
                event,
              )
          "
          @mousemove="
            (event) =>
              doMultiselect({ row, column, idx: row * columns + column }, event)
          "
          @mouseup="endMultiselect"
          @touchstart="
            (event) =>
              startMultiselect(
                { row, column, idx: row * columns + column },
                event,
              )
          "
          @touchmove="(event) => handleTouchMove(event)"
          @touchend="
            (event) => {
              event.preventDefault();
              endMultiselect();
            }
          "
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
