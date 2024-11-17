<script setup lang="ts">
import { inject, onMounted } from "vue";
import { Sudoku } from "@/models/sudoku";
import { defaultSettings, Settings } from "@/models/settings";

const sudoku = inject<Sudoku>("sudoku")!;

const settings = inject<Settings>("settings") ?? defaultSettings;

onMounted(() => {
  sudoku.updateState(true, (state) => {
    state.cells[0].colors.add(0);
    state.cells[2].colors.add(2);
    state.cells[2].colors.add(3);
  });
});

const { cellSize, backgroundColorPalette } = settings.appearance.sudoku;

function getIntersectionPointOnBorder(angle: number): [number, number] {
  angle = angle % (Math.PI * 2);
  if (angle < Math.PI / 4 || angle > (Math.PI * 7) / 4) {
    const x = cellSize / 2 + (cellSize / 2) * Math.tan(angle);
    const y = 0;
    return [x, y];
  } else if (angle < (Math.PI * 3) / 4) {
    const x = cellSize;
    const y = cellSize / 2 + (cellSize / 2) * Math.tan(angle - Math.PI / 2);
    return [x, y];
  } else if (angle < (Math.PI * 5) / 4) {
    const x = cellSize / 2 - (cellSize / 2) * Math.tan(angle - Math.PI);
    const y = cellSize;
    return [x, y];
  } else {
    const x = 0;
    const y =
      cellSize / 2 - (cellSize / 2) * Math.tan(angle - (Math.PI * 3) / 2);
    return [x, y];
  }
}
function generateBackgroundFillPath(totalCount: number, idx: number): string {
  if (totalCount === 1) {
    return `M 0 0 L ${cellSize} 0 L ${cellSize} ${cellSize} L 0 ${cellSize} Z`;
  }

  const angleOffset = Math.PI / 6;
  const angle = (Math.PI * 2) / totalCount;
  const startAngle = angle * idx + angleOffset;
  const endAngle = angle * (idx + 1) + angleOffset;
  let path = `M ${cellSize / 2} ${cellSize / 2}`;
  path += ` L ${getIntersectionPointOnBorder(startAngle).join(" ")}`;
  for (const vertex of [
    Math.PI / 4,
    (Math.PI * 3) / 4,
    (Math.PI * 5) / 4,
    (Math.PI * 7) / 4,
  ]) {
    if (startAngle < vertex && vertex < endAngle) {
      path += ` L ${getIntersectionPointOnBorder(vertex).join(" ")}`;
    }
  }
  path += ` L ${getIntersectionPointOnBorder(endAngle).join(" ")}`;
  path += " Z";
  return path;
}
const backgroundFillPaths: { [key: number]: { [key: number]: string } } = {};
function getBackgroundFillPath(totalCount: number, idx: number): string {
  if (!backgroundFillPaths[totalCount]) {
    backgroundFillPaths[totalCount] = {};
  }
  if (!backgroundFillPaths[totalCount][idx]) {
    backgroundFillPaths[totalCount][idx] = generateBackgroundFillPath(
      totalCount,
      idx,
    );
  }
  return backgroundFillPaths[totalCount][idx];
}

function sortColors(colors: Set<number>): number[] {
  return [...colors].sort();
}
</script>
<template>
  <g class="sudoku-cell-background">
    <template v-for="cell in sudoku.cells" :key="cell.position.idx">
      <path
        :transform="`translate(${cell.position.column * cellSize} ${cell.position.row * cellSize})`"
        v-for="(color, i) in sortColors(cell.colors)"
        :key="i"
        :d="getBackgroundFillPath(cell.colors.size, i)"
        :fill="backgroundColorPalette[color]"
      />
    </template>
  </g>
</template>
