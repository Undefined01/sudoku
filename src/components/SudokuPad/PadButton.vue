<script setup lang="ts">
import { inject, defineProps } from "vue";
import { Sudoku } from "@/models/sudoku";
import { PadMode } from "./Pad";
import { Settings } from "@/models/settings";

const sudoku = inject<Sudoku>("sudoku")!;
const settings = inject<Settings>("settings")!;

const { mode, idx } = defineProps<{ mode: PadMode; idx: number }>();
const { backgroundColorPalette } = settings.appearance.sudoku;

const toggleValueOfSelectedCells = (value: number) => {
  sudoku.updateState(true, (state) =>
    state.toggleValue(state.selectedCells.values(), value),
  );
};
const toggleCandidateOfSelectedCells = (value: number) => {
  sudoku.updateState(true, (state) =>
    state.toggleCandidate(state.selectedCells.values(), value),
  );
};
const togglePencilMarkOfSelectedCells = (value: number) => {
  sudoku.updateState(true, (state) =>
    state.togglePencilMark(state.selectedCells.values(), value),
  );
};
const toggleColorOfSelectedCells = (value: number) => {
  sudoku.updateState(true, (state) =>
    state.toggleColor(state.selectedCells.values(), value),
  );
};
</script>

<template>
  <v-btn
    v-if="mode === 'value'"
    class="pad-button"
    @click="() => toggleValueOfSelectedCells(idx)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
      <text
        font-size="16"
        font-weight="bold"
        x="12"
        y="12"
        text-anchor="middle"
        dominant-baseline="central"
      >
        {{ idx }}
      </text>
    </svg>
  </v-btn>
  <v-btn
    v-else-if="mode === 'candidate'"
    class="pad-button"
    @click="() => toggleCandidateOfSelectedCells(idx)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
      <text
        font-size="16"
        x="12"
        y="12"
        text-anchor="middle"
        dominant-baseline="central"
      >
        {{ idx }}
      </text>
    </svg>
  </v-btn>
  <v-btn
    v-else-if="mode === 'pencilMark'"
    class="pad-button"
    @click="() => togglePencilMarkOfSelectedCells(idx)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
      <text
        font-size="12"
        :x="12 + [-6, 0, 6][(idx - 1) % 3]"
        :y="12 + [-6, 0, 6][Math.floor((idx - 1) / 3)]"
        text-anchor="middle"
        dominant-baseline="central"
      >
        {{ idx }}
      </text>
    </svg>
  </v-btn>
  <v-btn
    v-else-if="mode === 'color'"
    class="pad-button"
    @click="() => toggleColorOfSelectedCells(idx - 1)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
      <rect
        x="0"
        y="0"
        width="24"
        height="24"
        :fill="backgroundColorPalette[idx - 1]"
      />
    </svg>
  </v-btn>
</template>

<style scoped>
.pad-button {
  width: 100%;
  height: auto !important;
  aspect-ratio: 1 / 1;
  min-width: auto;
}

.pad-button svg {
  width: 100%;
  height: 100%;
}

.pad-button i {
  display: block;
  width: 50%;
  height: 50%;
}
</style>
