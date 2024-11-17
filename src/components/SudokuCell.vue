<script setup lang="ts">
import { inject } from "vue";
import { Sudoku } from "@/models/sudoku";
import { defaultSettings, Settings } from "@/models/settings";

const sudoku = inject<Sudoku>("sudoku")!;

const settings = inject<Settings>("settings") ?? defaultSettings;

const {
  cellSize,
  valueFontSize,
  pencilMarkOffset,
  pencilMarkOffsetWhenHavingCandidates,
  pencilMarkFontSize,
  candidateFontSize,
} = settings.appearance.sudoku;
const pencilMarkPosition = [
  {
    x: cellSize / 2 - pencilMarkOffsetWhenHavingCandidates.x,
    y: cellSize / 2 - pencilMarkOffsetWhenHavingCandidates.y,
  },

  {
    x: cellSize / 2 - pencilMarkOffset.x,
    y: cellSize / 2 - pencilMarkOffset.y,
  },
  { x: cellSize / 2, y: cellSize / 2 - pencilMarkOffset.y },
  {
    x: cellSize / 2 + pencilMarkOffset.x,
    y: cellSize / 2 - pencilMarkOffset.y,
  },

  { x: cellSize / 2 - pencilMarkOffset.x, y: cellSize / 2 },
  { x: cellSize / 2, y: cellSize / 2 },
  { x: cellSize / 2 + pencilMarkOffset.x, y: cellSize / 2 },

  {
    x: cellSize / 2 - pencilMarkOffset.x,
    y: cellSize / 2 + pencilMarkOffset.y,
  },
  { x: cellSize / 2, y: cellSize / 2 + pencilMarkOffset.y },
  {
    x: cellSize / 2 + pencilMarkOffset.x,
    y: cellSize / 2 + pencilMarkOffset.y,
  },
];
const fivePositionOnHavingCandidates = {
  x: cellSize / 2 + pencilMarkOffsetWhenHavingCandidates.x,
  y: cellSize / 2 - pencilMarkOffsetWhenHavingCandidates.y,
};
</script>

<template>
  <g class="sudoku-cell">
    <template v-for="cell in sudoku.cells" :key="cell.position.idx">
      <text
        v-if="cell.value !== undefined"
        :x="(cell.position.column + 0.5) * cellSize"
        :y="(cell.position.row + 0.5) * cellSize"
        class="sudoku-cell-text"
        text-anchor="middle"
        dominant-baseline="central"
        :font-size="valueFontSize"
        :fill="cell.isGiven ? 'black' : 'blue'"
      >
        {{ cell.value }}
      </text>
      <text
        v-if="cell.candidates.size > 0"
        :x="(cell.position.column + 0.5) * cellSize"
        :y="(cell.position.row + 0.5) * cellSize"
        class="sudoku-cell-text"
        text-anchor="middle"
        dominant-baseline="central"
        :font-size="candidateFontSize"
        fill="blue"
        stroke-width="3px"
        stroke="#fff"
        paint-order="stroke"
      >
        {{ [...cell.candidates].sort().join("") }}
      </text>
      <template v-for="pencilMark in cell.pencilMarks" :key="pencilMark">
        <text
          v-if="!(pencilMark === 5 && cell.candidates.size > 0)"
          :x="
            cell.position.column * cellSize + pencilMarkPosition[pencilMark].x
          "
          :y="cell.position.row * cellSize + pencilMarkPosition[pencilMark].y"
          class="sudoku-cell-text"
          text-anchor="middle"
          dominant-baseline="central"
          :font-size="pencilMarkFontSize"
          fill="gray"
          stroke-width="2px"
          stroke="#fff"
          paint-order="stroke"
        >
          {{ pencilMark }}
        </text>
        <text
          v-else
          :x="
            cell.position.column * cellSize + fivePositionOnHavingCandidates.x
          "
          :y="cell.position.row * cellSize + fivePositionOnHavingCandidates.y"
          class="sudoku-cell-text"
          text-anchor="middle"
          dominant-baseline="central"
          :font-size="pencilMarkFontSize"
          fill="gray"
          stroke-width="2px"
          stroke="#fff"
          paint-order="stroke"
        >
          5
        </text>
      </template>
    </template>
  </g>
</template>

<style scoped>
.sudoku-cell-text {
  user-select: none;
}
</style>
