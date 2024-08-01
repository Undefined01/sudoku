<script setup lang="ts">
import { computed, inject } from "vue";
import { CellPosition, CellSet, Sudoku } from "@/models/sudoku";
import { defaultSettings, Settings } from "@/models/settings";

const { highlightedCells: highlightedCellsGetter } = defineProps<{
  highlightedCells: () => CellSet;
}>();

const sudoku = inject<Sudoku>("sudoku")!;
const { metadata } = sudoku;
const { rows, columns } = metadata;
const highlightedCells = computed(highlightedCellsGetter);

const settings = inject<Settings>("settings") ?? defaultSettings;
const borderWidth = settings.appearance.sudoku.selectedHighlight.width;
const cellSize = settings.appearance.sudoku.cellSize;

const getCellIdx = (row: number, column: number) => {
  return row * columns + column;
};
const outOfBound = (row: number, column: number) => {
  return row < 0 || row >= rows || column < 0 || column >= columns;
};

const regionBorderPath = computed(() => {
  const cellsMap = new Set<number>();
  const triedUp = new Set<number>();

  highlightedCells.value.forEach((cell) => {
    cellsMap.add(getCellIdx(cell.row, cell.column));
  });

  const cellsToRegionBorder = (startCell: CellPosition): string => {
    const directions = [
      { dr: -1, dc: 0 }, // Up
      { dr: 0, dc: 1 }, // Right
      { dr: 1, dc: 0 }, // Down
      { dr: 0, dc: -1 }, // Left
    ];
    const corner = [
      { dr: borderWidth * 0.5, dc: cellSize - borderWidth * 0.5 }, // Top right
      { dr: cellSize - borderWidth * 0.5, dc: cellSize - borderWidth * 0.5 }, // Bottom right
      { dr: cellSize - borderWidth * 0.5, dc: borderWidth * 0.5 }, // Bottom left
      { dr: borderWidth * 0.5, dc: borderWidth * 0.5 }, // Top left
    ];

    let path = "";

    // 贴墙走法，从最上的cell开始，保持左侧贴着墙、顺时针绕行
    let startRow = startCell.row;
    let startCol = startCell.column;
    let currentRow = startRow;
    let currentCol = startCol;
    let startDirection = 0;
    let directionIndex = startDirection;

    const startPoint = `M ${currentCol * cellSize + corner[3].dc} ${currentRow * cellSize + corner[3].dr}`;
    path += startPoint;

    do {
      let currentIdx = getCellIdx(currentRow, currentCol);
      if (directionIndex === 0) {
        triedUp.add(currentIdx);
      }
      let nextRow = currentRow + directions[directionIndex].dr;
      let nextCol = currentCol + directions[directionIndex].dc;

      if (
        outOfBound(nextRow, nextCol) ||
        !cellsMap.has(getCellIdx(nextRow, nextCol))
      ) {
        path += ` L ${currentCol * cellSize + corner[directionIndex].dc} ${currentRow * cellSize + corner[directionIndex].dr}`;
        directionIndex = (directionIndex + 1) % 4;
      } else {
        currentRow = nextRow;
        currentCol = nextCol;
        directionIndex = (directionIndex + 3) % 4;
        path += ` L ${currentCol * cellSize + corner[(directionIndex + 3) % 4].dc} ${currentRow * cellSize + corner[(directionIndex + 3) % 4].dr}`;
      }
    } while (
      currentRow !== startRow ||
      currentCol !== startCol ||
      directionIndex !== startDirection
    );

    path += " Z ";
    return path;
  };

  let finalPaths = "";

  // 找到一个位于极高点的cell，然后调用cellsToRegionBorder得到外围路径
  for (let cell of highlightedCells.value.values()) {
    if (
      (outOfBound(cell.row - 1, cell.column) ||
        !cellsMap.has(getCellIdx(cell.row - 1, cell.column))) &&
      !triedUp.has(getCellIdx(cell.row, cell.column))
    ) {
      finalPaths += cellsToRegionBorder(cell);
    }
  }

  return finalPaths;
});
</script>

<template>
  <g class="sudoku-highlighter">
    <path
      :d="regionBorderPath"
      fill="transparent"
      stroke="rgba(0, 126, 255, 0.7)"
      :stroke-width="borderWidth"
      stroke-linejoin="round"
    />
  </g>
</template>

<style scoped></style>
